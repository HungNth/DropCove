function Assert-MsixVersion {
    param(
        [Parameter(Mandatory = $true)][string]$Value,
        [switch]$StoreSubmission
    )

    if ($Value -notmatch '^\d+\.\d+\.\d+\.\d+$') {
        throw "MSIX version must use four numeric components, for example 1.2.3.0: $Value"
    }

    $components = @($Value.Split('.') | ForEach-Object { [int]$_ })
    foreach ($component in $components) {
        if ($component -lt 0 -or $component -gt 65535) {
            throw "MSIX version components must be between 0 and 65535: $Value"
        }
    }

    if ($StoreSubmission -and ($components[0] -eq 0 -or $components[3] -ne 0)) {
        throw "Microsoft Store package versions require a nonzero major component and a zero revision component: $Value"
    }
}
function Write-JsonFile {
    param(
        [Parameter(Mandatory = $true)]$Value,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $parent = Split-Path -Parent $Path
    if (-not [string]::IsNullOrWhiteSpace($parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }

    $json = $Value | ConvertTo-Json -Depth 8
    [IO.File]::WriteAllText($Path, $json, [Text.UTF8Encoding]::new($false))
}


function Resolve-SdkTool {
    param([Parameter(Mandatory = $true)][string]$Name)

    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($null -ne $command) {
        return $command.Path
    }

    $nugetRoot = Join-Path $env:USERPROFILE '.nuget\packages\microsoft.windows.sdk.buildtools'
    if (Test-Path -LiteralPath $nugetRoot) {
        $candidate = Get-ChildItem -LiteralPath $nugetRoot -Filter $Name -File -Recurse |
            Where-Object { $_.FullName -match '\\x64\\' } |
            Sort-Object FullName -Descending |
            Select-Object -First 1
        if ($null -ne $candidate) {
            return $candidate.FullName
        }
    }

    throw "Windows SDK tool was not found: $Name. Restore Microsoft.Windows.SDK.BuildTools before running this pipeline."
}

function Invoke-External {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code $LASTEXITCODE`: $FilePath $($Arguments -join ' ')"
    }
}

function Assert-SelfContainedPackageLayout {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [switch]$RequireVisualAssets
    )

    $requiredFiles = @(
        'DropCove.exe',
        'DropCove.runtimeconfig.json',
        'DropCove.deps.json',
        'coreclr.dll',
        'hostfxr.dll',
        'System.Private.CoreLib.dll',
        'Microsoft.WindowsAppRuntime.Bootstrap.Net.dll'
    )
    if ($RequireVisualAssets) {
        $requiredFiles += @(
            'Assets\DropCove-44.png',
            'Assets\DropCove-150.png'
        )
    }

    foreach ($relativePath in $requiredFiles) {
        $path = Join-Path $Root $relativePath
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "MSIX package is missing required self-contained/runtime file: $relativePath"
        }
    }
}

function Resolve-AppCertificationKit {
    param([string]$OverridePath)

    if (-not [string]::IsNullOrWhiteSpace($OverridePath)) {
        $resolvedOverride = [IO.Path]::GetFullPath($OverridePath)
        if (-not (Test-Path -LiteralPath $resolvedOverride -PathType Leaf)) {
            throw "Windows App Certification Kit executable was not found: $resolvedOverride"
        }
        return $resolvedOverride
    }

    $command = Get-Command appcert.exe -ErrorAction SilentlyContinue
    if ($null -ne $command) {
        return $command.Path
    }

    foreach ($candidate in @(
        'C:\Program Files (x86)\Windows Kits\10\App Certification Kit\appcert.exe',
        'C:\Program Files\Windows Kits\10\App Certification Kit\appcert.exe'
    )) {
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            return $candidate
        }
    }

    throw 'Windows App Certification Kit was not found. Install the Windows SDK App Certification Kit component or pass -CertificationKitPath to the production MSIX build.'
}

function Assert-AppCertificationReportPassed {
    param([Parameter(Mandatory = $true)][string]$ReportPath)

    $document = New-Object System.Xml.XmlDocument
    $document.Load($ReportPath)
    $resultNames = @(
        'result',
        'status',
        'outcome',
        'state',
        'passfail',
        'certificationresult',
        'overallresult'
    )
    $values = New-Object System.Collections.Generic.List[string]
    foreach ($element in $document.SelectNodes('//*')) {
        foreach ($attribute in $element.Attributes) {
            if ($resultNames -contains $attribute.Name.ToLowerInvariant()) {
                $values.Add($attribute.Value.Trim()) | Out-Null
            }
        }
        if ($resultNames -contains $element.LocalName.ToLowerInvariant()) {
            $values.Add($element.InnerText.Trim()) | Out-Null
        }
    }

    $failedValues = @($values.ToArray() | Where-Object { $_ -match '^(fail|failed|failure|error|notcertified|not certified)$' })
    if ($failedValues.Count -gt 0) {
        throw "Windows App Certification Kit report contains failed results: $($failedValues -join ', ')"
    }

    $passedValues = @($values.ToArray() | Where-Object { $_ -match '^(pass|passed|success|successful|certified)$' })
    if ($passedValues.Count -eq 0) {
        throw "Windows App Certification Kit report did not contain an explicit passing result: $ReportPath"
    }

    [ordered]@{
        Report = [IO.Path]::GetFileName($ReportPath)
        Status = 'Passed'
    }
}

function Invoke-AppCertificationKit {
    param(
        [Parameter(Mandatory = $true)][string]$AppCertificationKitPath,
        [Parameter(Mandatory = $true)][string]$PackagePath,
        [Parameter(Mandatory = $true)][string]$ReportPath
    )

    Invoke-External -FilePath $AppCertificationKitPath -Arguments @('reset')
    Invoke-External -FilePath $AppCertificationKitPath -Arguments @(
        'test',
        '-appxpackagepath', $PackagePath,
        '-reportoutputpath', $ReportPath
    )
    if (-not (Test-Path -LiteralPath $ReportPath -PathType Leaf)) {
        throw "Windows App Certification Kit completed without producing its report: $ReportPath"
    }
    Assert-AppCertificationReportPassed -ReportPath $ReportPath
}
