[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$PackagePath,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedIdentityName,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedPublisher,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+\.\d+$')]
    [string]$ExpectedVersion,

    [switch]$StoreSubmission,

    [string]$OutputReportPath

)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$commonScriptPath = Join-Path $PSScriptRoot 'msix-common.ps1'
if (-not (Test-Path -LiteralPath $commonScriptPath -PathType Leaf)) {
    throw "Required MSIX helper script was not found: $commonScriptPath"
}
. $commonScriptPath




function Read-PackageManifest {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [Parameter(Mandatory = $true)][string]$PackageFile
    )

    $manifestPath = Join-Path $Root 'AppxManifest.xml'
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Package does not contain AppxManifest.xml: $PackageFile"
    }

    $document = New-Object System.Xml.XmlDocument
    $document.PreserveWhitespace = $true
    $document.Load($manifestPath)

    $namespaces = [System.Xml.XmlNamespaceManager]::new($document.NameTable)
    $namespaces.AddNamespace('f', 'http://schemas.microsoft.com/appx/manifest/foundation/windows10')
    $namespaces.AddNamespace('uap', 'http://schemas.microsoft.com/appx/manifest/uap/windows10')
    $namespaces.AddNamespace('uap10', 'http://schemas.microsoft.com/appx/manifest/uap/windows10/10')
    $namespaces.AddNamespace('rescap', 'http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities')

    $identity = $document.SelectSingleNode('/f:Package/f:Identity', $namespaces)
    $application = $document.SelectSingleNode('/f:Package/f:Applications/f:Application', $namespaces)
    $deviceFamily = $document.SelectSingleNode('/f:Package/f:Dependencies/f:TargetDeviceFamily', $namespaces)
    $visualElements = $document.SelectSingleNode('/f:Package/f:Applications/f:Application/uap:VisualElements', $namespaces)
    $fullTrust = $document.SelectSingleNode('/f:Package/f:Capabilities/rescap:Capability[@Name="runFullTrust"]', $namespaces)

    if ($null -eq $identity -or $null -eq $application -or $null -eq $deviceFamily -or $null -eq $visualElements) {
        throw "Package manifest is missing required identity, application, device-family, or visual elements: $PackageFile"
    }

    if ($null -eq $fullTrust) {
        throw "Package manifest does not declare the required full-trust capability: $PackageFile"
    }

    if ($identity.GetAttribute('Name') -ne $ExpectedIdentityName) {
        throw "Unexpected package identity name in ${PackageFile}: $($identity.GetAttribute('Name'))"
    }
    if ($identity.GetAttribute('Publisher') -ne $ExpectedPublisher) {
        throw "Unexpected package publisher in ${PackageFile}: $($identity.GetAttribute('Publisher'))"
    }
    if ($identity.GetAttribute('Version') -ne $ExpectedVersion) {
        throw "Unexpected package version in ${PackageFile}: $($identity.GetAttribute('Version'))"
    }
    if ($identity.GetAttribute('ProcessorArchitecture') -ne 'x64') {
        throw "Package is not x64: $PackageFile"
    }
    if ($application.GetAttribute('Executable') -ne 'DropCove.exe') {
        throw "Package application does not launch DropCove.exe: $PackageFile"
    }
    if ($application.GetAttribute('uap10:RuntimeBehavior') -ne 'packagedClassicApp' -or
        $application.GetAttribute('uap10:TrustLevel') -ne 'mediumIL') {
        throw "Package application is not configured as a medium-integrity packaged classic app: $PackageFile"
    }
    if ($deviceFamily.GetAttribute('Name') -ne 'Windows.Desktop') {
        throw "Package targets an unexpected device family: $PackageFile"
    }

    [ordered]@{
        Package = $PackageFile
        IdentityName = $identity.GetAttribute('Name')
        Publisher = $identity.GetAttribute('Publisher')
        Version = $identity.GetAttribute('Version')
        Architecture = $identity.GetAttribute('ProcessorArchitecture')
        Executable = $application.GetAttribute('Executable')
        RuntimeBehavior = $application.GetAttribute('uap10:RuntimeBehavior')
        TrustLevel = $application.GetAttribute('uap10:TrustLevel')
        TargetDeviceFamily = $deviceFamily.GetAttribute('Name')
        SelfContainedRuntimeFiles = $true
    }
}

$resolvedPackagePath = [IO.Path]::GetFullPath($PackagePath)
if (-not (Test-Path -LiteralPath $resolvedPackagePath -PathType Leaf)) {
    throw "MSIX artifact was not found: $resolvedPackagePath"
}
Assert-MsixVersion -Value $ExpectedVersion -StoreSubmission:$StoreSubmission

$makeAppx = Resolve-SdkTool 'makeappx.exe'
$temporaryRoot = Join-Path ([IO.Path]::GetTempPath()) ("DropCove-MsixValidation-{0}" -f [Guid]::NewGuid().ToString('N'))
$packageRoot = Join-Path $temporaryRoot 'package'
$unpackedRoot = Join-Path $temporaryRoot 'unpacked'
$bundlePackageRoot = Join-Path $temporaryRoot 'bundle-packages'
$reports = New-Object System.Collections.Generic.List[object]

try {
    New-Item -ItemType Directory -Path $packageRoot, $unpackedRoot, $bundlePackageRoot -Force | Out-Null

    $extension = [IO.Path]::GetExtension($resolvedPackagePath).ToLowerInvariant()
    if ($extension -eq '.msixbundle') {
        Invoke-External -FilePath $makeAppx -Arguments @('unbundle', '/o', '/p', $resolvedPackagePath, '/d', $bundlePackageRoot)
        $innerPackages = @(Get-ChildItem -LiteralPath $bundlePackageRoot -Filter '*.msix' -File -Recurse)
        if ($innerPackages.Count -eq 0) {
            throw "MSIXBundle contains no MSIX package: $resolvedPackagePath"
        }

        foreach ($innerPackage in $innerPackages) {
            $innerUnpackedRoot = Join-Path $unpackedRoot $innerPackage.BaseName
            New-Item -ItemType Directory -Path $innerUnpackedRoot -Force | Out-Null
            Invoke-External -FilePath $makeAppx -Arguments @('unpack', '/o', '/p', $innerPackage.FullName, '/d', $innerUnpackedRoot)
            Assert-SelfContainedPackageLayout -Root $innerUnpackedRoot -RequireVisualAssets
            $reports.Add((Read-PackageManifest -Root $innerUnpackedRoot -PackageFile $innerPackage.Name))
        }
    }
    elseif ($extension -eq '.msix') {
        Invoke-External -FilePath $makeAppx -Arguments @('unpack', '/o', '/p', $resolvedPackagePath, '/d', $packageRoot)
        Assert-SelfContainedPackageLayout -Root $packageRoot -RequireVisualAssets
        $reports.Add((Read-PackageManifest -Root $packageRoot -PackageFile ([IO.Path]::GetFileName($resolvedPackagePath))))
    }
    else {
        throw "Expected an .msix or .msixbundle artifact: $resolvedPackagePath"
    }

    $privateKeyFiles = @(
        @($packageRoot, $unpackedRoot, $bundlePackageRoot) |
            ForEach-Object {
                if (Test-Path -LiteralPath $_) {
                    Get-ChildItem -LiteralPath $_ -File -Recurse -ErrorAction SilentlyContinue
                }
            } |
            Where-Object { $_.Extension.ToLowerInvariant() -in @('.pfx', '.p12', '.pem', '.key', '.snk') }
    )
    if ($privateKeyFiles.Count -gt 0) {
        throw "MSIX artifact contains private-key material: $($privateKeyFiles.FullName -join ', ')"
    }

    $report = [ordered]@{
        Artifact = $resolvedPackagePath
        IdentityName = $ExpectedIdentityName
        Publisher = $ExpectedPublisher
        Version = $ExpectedVersion
        Architecture = 'x64'
        PackageCount = $reports.Count
        SelfContained = $true
        StoreSubmissionIdentityChecked = $true
        Packages = @($reports.ToArray())
    }

    if (-not [string]::IsNullOrWhiteSpace($OutputReportPath)) {
        Write-JsonFile -Value $report -Path $OutputReportPath
    }
    $report | ConvertTo-Json -Depth 8
}
finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
