[CmdletBinding()]
param(
    [ValidateSet('Development', 'Production')]
    [string]$Configuration = 'Development',

    [string]$Version = '0.1.0.0',

    [string]$IdentityName,

    [string]$Publisher,

    [string]$PublisherDisplayName = 'DropCove',

    [string]$OutputRoot,

    [string]$CertificationKitPath

)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$commonScriptPath = Join-Path $PSScriptRoot 'msix-common.ps1'
if (-not (Test-Path -LiteralPath $commonScriptPath -PathType Leaf)) {
    throw "Required MSIX helper script was not found: $commonScriptPath"
}
. $commonScriptPath

function Write-Utf8NoBom {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Content
    )

    [IO.File]::WriteAllText($Path, $Content, [Text.UTF8Encoding]::new($false))
}




function Assert-IdentityName {
    param([Parameter(Mandatory = $true)][string]$Value)

    if ($Value -notmatch '^[A-Za-z0-9.-]+$') {
        throw "MSIX identity name contains unsupported characters: $Value"
    }
    if ($Value.Length -gt 50) {
        throw "MSIX identity name is longer than the package limit: $Value"
    }
}

function Escape-XmlAttribute {
    param([Parameter(Mandatory = $true)][string]$Value)

    return [Security.SecurityElement]::Escape($Value)
}

function New-PackageLogos {
    param(
        [Parameter(Mandatory = $true)][string]$IconPath,
        [Parameter(Mandatory = $true)][string]$AssetDirectory
    )

    if (-not (Test-Path -LiteralPath $IconPath -PathType Leaf)) {
        throw "Application icon was not found: $IconPath"
    }

    try {
        Add-Type -AssemblyName System.Drawing
    }
    catch {
        throw "System.Drawing is required to create deterministic MSIX PNG logos from AppIcon.ico: $($_.Exception.Message)"
    }

    $icon = New-Object System.Drawing.Icon($IconPath)
    $sourceBitmap = $icon.ToBitmap()
    try {
        foreach ($size in @(44, 150)) {
            $targetPath = Join-Path $AssetDirectory ("DropCove-{0}.png" -f $size)
            $bitmap = New-Object System.Drawing.Bitmap($size, $size)
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.Clear([System.Drawing.Color]::Transparent)
                $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
                $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
                $graphics.DrawImage($sourceBitmap, 0, 0, $size, $size)
                $bitmap.Save($targetPath, [System.Drawing.Imaging.ImageFormat]::Png)
            }
            finally {
                $graphics.Dispose()
                $bitmap.Dispose()
            }
        }
    }
    finally {
        $sourceBitmap.Dispose()
        $icon.Dispose()
    }
}

function New-DevelopmentCertificate {
    param(
        [Parameter(Mandatory = $true)][string]$Subject,
        [Parameter(Mandatory = $true)][string]$PfxPath,
        [Parameter(Mandatory = $true)][string]$PublicCertificatePath
    )

    $command = Get-Command New-SelfSignedCertificate -ErrorAction SilentlyContinue
    if ($null -eq $command) {
        throw 'New-SelfSignedCertificate is required for Development MSIX builds.'
    }

    $certificate = New-SelfSignedCertificate `
        -Type Custom `
        -KeyAlgorithm RSA `
        -KeyLength 2048 `
        -HashAlgorithm SHA256 `
        -KeyExportPolicy Exportable `
        -KeyUsage DigitalSignature `
        -Subject $Subject `
        -TextExtension @(
            '2.5.29.37={text}1.3.6.1.5.5.7.3.3',
            '2.5.29.19={text}'
        ) `
        -CertStoreLocation 'Cert:\CurrentUser\My'

    if ($certificate.Subject -ne $Subject) {
        throw "Development certificate subject does not match the manifest publisher. Expected '$Subject', got '$($certificate.Subject)'."
    }

    $passwordText = [Guid]::NewGuid().ToString('N')
    $password = ConvertTo-SecureString -String $passwordText -AsPlainText -Force
    Export-PfxCertificate -Cert $certificate -FilePath $PfxPath -Password $password -Force | Out-Null
    Export-Certificate -Cert $certificate -FilePath $PublicCertificatePath -Force | Out-Null

    [ordered]@{
        Certificate = $certificate
        PfxPassword = $passwordText
        Thumbprint = $certificate.Thumbprint
        StorePath = "Cert:\CurrentUser\My\$($certificate.Thumbprint)"
    }
}

function Sign-DevelopmentArtifact {
    param(
        [Parameter(Mandatory = $true)][string]$SignToolPath,
        [Parameter(Mandatory = $true)][string]$PfxPath,
        [Parameter(Mandatory = $true)][string]$PfxPassword,
        [Parameter(Mandatory = $true)][string]$ArtifactPath
    )

    Invoke-External -FilePath $SignToolPath -Arguments @(
        'sign',
        '/fd', 'SHA256',
        '/a',
        '/f', $PfxPath,
        '/p', $PfxPassword,
        $ArtifactPath
    )
}

function Invoke-MsixValidator {
    param(
        [Parameter(Mandatory = $true)][string]$PackagePath,
        [Parameter(Mandatory = $true)][string]$ReportPath,
        [Parameter(Mandatory = $true)][string]$ValidatorPath,
        [Parameter(Mandatory = $true)][string]$ExpectedIdentityName,
        [Parameter(Mandatory = $true)][string]$ExpectedPublisher,
        [Parameter(Mandatory = $true)][string]$ExpectedVersion,
        [switch]$StoreSubmission
    )

    & $ValidatorPath `
        -PackagePath $PackagePath `
        -ExpectedIdentityName $ExpectedIdentityName `
        -ExpectedPublisher $ExpectedPublisher `
        -ExpectedVersion $ExpectedVersion `
        -OutputReportPath $ReportPath `
        -StoreSubmission:$StoreSubmission
    if ($LASTEXITCODE -ne 0) {
        throw "MSIX validation failed for $PackagePath with exit code $LASTEXITCODE."
    }
}

$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$appProject = Join-Path $repoRoot 'src\DropCove.App\DropCove.csproj'
$manifestTemplatePath = Join-Path $repoRoot 'packaging\msix\AppxManifest.template.xml'
$validatorPath = Join-Path $repoRoot 'scripts\validate-msix.ps1'
$iconPath = Join-Path $repoRoot 'src\DropCove.App\Assets\AppIcon.ico'

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $repoRoot 'artifacts\msix'
}
elseif (-not [IO.Path]::IsPathRooted($OutputRoot)) {
    $OutputRoot = Join-Path (Get-Location) $OutputRoot
}
$OutputRoot = [IO.Path]::GetFullPath($OutputRoot)

Assert-MsixVersion -Value $Version -StoreSubmission:($Configuration -eq 'Production')

if ($Configuration -eq 'Production') {
    if ([string]::IsNullOrWhiteSpace($IdentityName) -or [string]::IsNullOrWhiteSpace($Publisher)) {
        throw 'Production MSIX generation is disabled until both Partner Center IdentityName and Publisher are supplied with -IdentityName and -Publisher.'
    }
    if ($IdentityName -eq 'DropCove.Development' -or $Publisher -eq 'CN=DropCove Development') {
        throw 'Production MSIX generation cannot use the development package identity or publisher.'
    }
}
else {
    if ([string]::IsNullOrWhiteSpace($IdentityName)) {
        $IdentityName = 'DropCove.Development'
    }
    if ([string]::IsNullOrWhiteSpace($Publisher)) {
        $Publisher = 'CN=DropCove Development'
    }
}

Assert-IdentityName -Value $IdentityName
if ([string]::IsNullOrWhiteSpace($Publisher)) {
    throw 'MSIX Publisher cannot be empty.'
}
if ([string]::IsNullOrWhiteSpace($PublisherDisplayName)) {
    throw 'MSIX PublisherDisplayName cannot be empty.'
}

foreach ($requiredPath in @($appProject, $manifestTemplatePath, $validatorPath, $iconPath, $commonScriptPath)) {
    if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
        throw "Required MSIX pipeline input was not found: $requiredPath"
    }
}

$configurationRoot = Join-Path $OutputRoot $Configuration.ToLowerInvariant()
$versionRoot = Join-Path $configurationRoot $Version
$packageRoot = Join-Path $versionRoot 'package-layout'
$bundleInputRoot = Join-Path $versionRoot 'bundle-input'
$packagePath = Join-Path $versionRoot ("DropCove-{0}-x64.msix" -f $Version)
$bundlePath = Join-Path $versionRoot ("DropCove-{0}-x64.msixbundle" -f $Version)
$packageReportPath = Join-Path $versionRoot 'validation-msix.json'
$bundleReportPath = Join-Path $versionRoot 'validation-msixbundle.json'
$wackReportPath = Join-Path $versionRoot 'wack-msix-report.xml'
$wackBundleReportPath = Join-Path $versionRoot 'wack-msixbundle-report.xml'
$metadataPath = Join-Path $versionRoot 'build-metadata.json'
$publicCertificatePath = Join-Path $versionRoot ("DropCove-{0}-development.cer" -f $Version)
$signingRoot = Join-Path ([IO.Path]::GetTempPath()) ("DropCove-MsixSigning-{0}" -f [Guid]::NewGuid().ToString('N'))
$developmentPfxPath = Join-Path $signingRoot 'development-signing.pfx'
$certificateStorePath = $null

if (Test-Path -LiteralPath $versionRoot) {
    Remove-Item -LiteralPath $versionRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $packageRoot, $bundleInputRoot -Force | Out-Null
if ($Configuration -eq 'Development') {
    New-Item -ItemType Directory -Path $signingRoot -Force | Out-Null
}

$makeAppx = $null
$makePri = $null
$signtool = $null
$appCertificationKit = $null

$devCertificate = $null
$certificationResults = @()
try {
    $dotnet = (Get-Command dotnet.exe).Path
    $restoreArguments = @(
        'restore',
        $appProject,
        '-r', 'win-x64',
        '-p:Platform=x64'
    )
    Invoke-External -FilePath $dotnet -Arguments $restoreArguments

    $makeAppx = Resolve-SdkTool 'makeappx.exe'
    $makePri = Resolve-SdkTool 'makepri.exe'
    if ($Configuration -eq 'Development') {
        $signtool = Resolve-SdkTool 'signtool.exe'
    }
    if ($Configuration -eq 'Production') {
        $appCertificationKit = Resolve-AppCertificationKit -OverridePath $CertificationKitPath
    }

    $publishArguments = @(
        'publish',
        $appProject,
        '-c', 'Release',
        '-r', 'win-x64',
        '--self-contained', 'true',
        '--no-restore',
        "-p:Version=$Version",
        '-o', $packageRoot
    )
    Invoke-External -FilePath $dotnet -Arguments $publishArguments
    Assert-SelfContainedPackageLayout -Root $packageRoot

    $assetsDirectory = Join-Path $packageRoot 'Assets'
    New-Item -ItemType Directory -Path $assetsDirectory -Force | Out-Null
    New-PackageLogos -IconPath $iconPath -AssetDirectory $assetsDirectory

    $manifest = Get-Content -LiteralPath $manifestTemplatePath -Raw
    $manifest = $manifest.Replace('__IDENTITY_NAME__', (Escape-XmlAttribute -Value $IdentityName))
    $manifest = $manifest.Replace('__PUBLISHER__', (Escape-XmlAttribute -Value $Publisher))
    $manifest = $manifest.Replace('__VERSION__', (Escape-XmlAttribute -Value $Version))
    $manifest = $manifest.Replace('__DISPLAY_NAME__', (Escape-XmlAttribute -Value 'DropCove'))
    $manifest = $manifest.Replace('__PUBLISHER_DISPLAY_NAME__', (Escape-XmlAttribute -Value $PublisherDisplayName))
    $manifest = $manifest.Replace('__DESCRIPTION__', (Escape-XmlAttribute -Value 'A temporary drag-and-drop workspace for Windows.'))
    Write-Utf8NoBom -Path (Join-Path $packageRoot 'AppxManifest.xml') -Content $manifest

    $packArguments = @(
        'pack',
        '/o',
        '/v',
        '/pri', $makePri,
        '/d', $packageRoot,
        '/p', $packagePath
    )
    Invoke-External -FilePath $makeAppx -Arguments $packArguments

    if ($Configuration -eq 'Development') {
        $devCertificate = New-DevelopmentCertificate `
            -Subject $Publisher `
            -PfxPath $developmentPfxPath `
            -PublicCertificatePath $publicCertificatePath
        $certificateStorePath = $devCertificate.StorePath

        Sign-DevelopmentArtifact `
            -SignToolPath $signtool `
            -PfxPath $developmentPfxPath `
            -PfxPassword $devCertificate.PfxPassword `
            -ArtifactPath $packagePath
    }

    Copy-Item -LiteralPath $packagePath -Destination (Join-Path $bundleInputRoot ([IO.Path]::GetFileName($packagePath))) -Force
    $bundleArguments = @(
        'bundle',
        '/o',
        '/v',
        '/bv', $Version,
        '/d', $bundleInputRoot,
        '/p', $bundlePath
    )
    Invoke-External -FilePath $makeAppx -Arguments $bundleArguments

    if ($Configuration -eq 'Development') {
        Sign-DevelopmentArtifact `
            -SignToolPath $signtool `
            -PfxPath $developmentPfxPath `
            -PfxPassword $devCertificate.PfxPassword `
            -ArtifactPath $bundlePath
    }

    if (Test-Path -LiteralPath $developmentPfxPath) {
        Remove-Item -LiteralPath $developmentPfxPath -Force
    }
    if (Test-Path -LiteralPath $signingRoot) {
        Remove-Item -LiteralPath $signingRoot -Recurse -Force
    }

    foreach ($artifactPath in @($packagePath, $bundlePath)) {
        $signature = Get-AuthenticodeSignature -LiteralPath $artifactPath
        if ($Configuration -eq 'Production') {
            if ($signature.Status.ToString() -ne 'NotSigned') {
                throw "Production Store submission artifact must remain unsigned until Microsoft Store signs it: $artifactPath"
            }
        }
        elseif ($null -eq $signature.SignerCertificate -or $signature.SignerCertificate.Subject -ne $Publisher) {
            throw "Development artifact is not signed by the manifest publisher '$Publisher': $artifactPath"
        }
    }

    Invoke-MsixValidator `
        -PackagePath $packagePath `
        -ReportPath $packageReportPath `
        -ValidatorPath $validatorPath `
        -ExpectedIdentityName $IdentityName `
        -ExpectedPublisher $Publisher `
        -ExpectedVersion $Version `
        -StoreSubmission:($Configuration -eq 'Production')
    Invoke-MsixValidator `
        -PackagePath $bundlePath `
        -ReportPath $bundleReportPath `
        -ValidatorPath $validatorPath `
        -ExpectedIdentityName $IdentityName `
        -ExpectedPublisher $Publisher `
        -ExpectedVersion $Version `
        -StoreSubmission:($Configuration -eq 'Production')

    if ($Configuration -eq 'Production') {
        $certificationResults = @(
            Invoke-AppCertificationKit `
                -AppCertificationKitPath $appCertificationKit `
                -PackagePath $packagePath `
                -ReportPath $wackReportPath
            Invoke-AppCertificationKit `
                -AppCertificationKitPath $appCertificationKit `
                -PackagePath $bundlePath `
                -ReportPath $wackBundleReportPath
        )
    }

    $artifacts = @(
        [ordered]@{
            Path = [IO.Path]::GetFileName($packagePath)
            Format = 'MSIX'
            Architecture = 'x64'
            Sha256 = (Get-FileHash -LiteralPath $packagePath -Algorithm SHA256).Hash
        },
        [ordered]@{
            Path = [IO.Path]::GetFileName($bundlePath)
            Format = 'MSIXBundle'
            Architecture = 'x64-only'
            Sha256 = (Get-FileHash -LiteralPath $bundlePath -Algorithm SHA256).Hash
        }
    )

    $metadata = [ordered]@{
        Product = 'DropCove'
        Configuration = $Configuration
        Version = $Version
        IdentityName = $IdentityName
        Publisher = $Publisher
        PublisherDisplayName = $PublisherDisplayName
        Architecture = 'x64'
        SelfContained = $true
        Signed = ($Configuration -eq 'Development')
        SignatureState = if ($Configuration -eq 'Production') { 'UnsignedStoreSubmission' } else { 'SelfSignedDevelopment' }
        StoreSigning = if ($Configuration -eq 'Production') { 'Microsoft Store signs the package during submission.' } else { 'Self-signed development certificate; never publicly trusted.' }
        Capabilities = @('runFullTrust')
        Artifacts = $artifacts
        ValidationReports = @('validation-msix.json', 'validation-msixbundle.json')
        PrivateKeysStoredInRepository = $false
        PublicArtifacts = @('MSIX', 'MSIXBundle')
        CertificationKit = if ($Configuration -eq 'Production') { 'Windows App Certification Kit' } else { 'NotRunDevelopment' }
        CertificationStatus = if ($Configuration -eq 'Production') { 'Passed' } else { 'NotRunDevelopment' }
        CertificationResults = $certificationResults
    }
    if ($Configuration -eq 'Development') {
        $metadata['DevelopmentCertificate'] = [IO.Path]::GetFileName($publicCertificatePath)
    }
    Write-JsonFile -Value $metadata -Path $metadataPath

    Remove-Item -LiteralPath $packageRoot, $bundleInputRoot -Recurse -Force
    Write-Output "MSIX artifacts created and validated: $versionRoot"
}
finally {
    if ($null -ne $certificateStorePath -and (Test-Path -LiteralPath $certificateStorePath)) {
        Remove-Item -LiteralPath $certificateStorePath -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $developmentPfxPath) {
        Remove-Item -LiteralPath $developmentPfxPath -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $signingRoot) {
        Remove-Item -LiteralPath $signingRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
