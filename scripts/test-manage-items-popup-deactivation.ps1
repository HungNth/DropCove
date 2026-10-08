[CmdletBinding()]
param(
    [string]$ExecutablePath = "",
    [ValidateRange(2, 4)]
    [int]$ItemCount = 2
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($ExecutablePath)) {
    $ExecutablePath = Join-Path $PSScriptRoot "..\src\DropCove.App\bin\x64\Debug\net10.0-windows10.0.22000.0\win-x64\DropCove.exe"
}

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class ManageItemsPopupTestNative
{
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    [DllImport("user32.dll")] public static extern IntPtr GetShellWindow();
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extraInfo);

    public const uint MouseLeftDown = 0x0002;
    public const uint MouseLeftUp = 0x0004;
}
'@

function Find-ElementByName {
    param(
        [System.Windows.Automation.AutomationElement]$Root,
        [System.Windows.Automation.Condition]$ProcessCondition,
        [string]$Name
    )

    $elements = $Root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $ProcessCondition)
    foreach ($element in $elements) {
        if ($element.Current.Name -eq $Name) {
            return $element
        }
    }

    return $null
}

function Wait-ForElementByName {
    param(
        [System.Windows.Automation.AutomationElement]$Root,
        [System.Windows.Automation.Condition]$ProcessCondition,
        [string]$Name,
        [int]$TimeoutSeconds = 10
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        $element = Find-ElementByName -Root $Root -ProcessCondition $ProcessCondition -Name $Name
        if ($null -ne $element) {
            return $element
        }

        Start-Sleep -Milliseconds 100
    } while ((Get-Date) -lt $deadline)

    return $null
}

$resolvedExecutable = Resolve-Path $ExecutablePath -ErrorAction SilentlyContinue
if ($null -eq $resolvedExecutable) {
    throw "DropCove executable not found at: $ExecutablePath. Build the application first."
}

$runId = [Guid]::NewGuid().ToString('N')
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) "DropCove-PopupDeactivation-$runId"
$profileRoot = Join-Path $tempRoot 'profile'
$fixturesRoot = Join-Path $tempRoot 'fixtures'
$dbPath = Join-Path $profileRoot 'shelf.db'
$settingsPath = Join-Path $profileRoot 'settings.json'
New-Item -ItemType Directory -Path $profileRoot -Force | Out-Null
New-Item -ItemType Directory -Path $fixturesRoot -Force | Out-Null

@{
    StartWithWindows = $false
    ShakeEnabled = $false
    ShakeSensitivity = 1
    RailMonitorId = ""
    RailAlignment = 1
    HotKey = @{
        Control = $true
        Alt = $false
        Shift = $true
        Windows = $false
        VirtualKey = 32
    }
} | ConvertTo-Json | Set-Content $settingsPath

$seedScript = Join-Path $PSScriptRoot '..\artifacts\seed-qualification-profile.py'
& python $seedScript $dbPath $fixturesRoot | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Failed to seed the disposable DropCove profile."
}

$minimizeScript = @"
import sqlite3

conn = sqlite3.connect(r'''$dbPath''')
conn.execute('PRAGMA foreign_keys = ON')
expected_item_count = $ItemCount
batch_id = conn.execute('SELECT id FROM shelf_batches WHERE position = 2').fetchone()[0]
conn.execute('DELETE FROM shelf_items WHERE batch_id = ? AND position >= ?', (batch_id, expected_item_count))
conn.execute('DELETE FROM shelf_batches WHERE id <> ?', (batch_id,))
conn.execute('UPDATE shelf_batches SET position = 0 WHERE id = ?', (batch_id,))
conn.commit()
batch_count = conn.execute('SELECT COUNT(*) FROM shelf_batches').fetchone()[0]
item_count = conn.execute('SELECT COUNT(*) FROM shelf_items').fetchone()[0]
if batch_count != 1 or item_count != expected_item_count:
    raise RuntimeError(f'Expected one batch with {expected_item_count} items, found {batch_count} batches and {item_count} items')
conn.close()
"@
& python -c $minimizeScript
if ($LASTEXITCODE -ne 0) {
    throw "Failed to minimize the disposable DropCove profile."
}

$dropCove = $null
try {
    $anchorWindowHandle = [ManageItemsPopupTestNative]::GetShellWindow()
    if ($anchorWindowHandle -eq [IntPtr]::Zero) {
        throw "Windows shell window is unavailable."
    }
    $shellProcessId = [uint32]0
    [ManageItemsPopupTestNative]::GetWindowThreadProcessId($anchorWindowHandle, [ref]$shellProcessId) | Out-Null

    $dropCove = Start-Process -FilePath $resolvedExecutable -ArgumentList "--test-profile `"$profileRoot`"" -PassThru
    $desktop = [System.Windows.Automation.AutomationElement]::RootElement
    $processCondition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ProcessIdProperty,
        $dropCove.Id
    )

    $manageButton = Wait-ForElementByName -Root $desktop -ProcessCondition $processCondition -Name "Manage $ItemCount items"
    if ($null -eq $manageButton) {
        throw "Manage Items button did not appear."
    }

    $bounds = $manageButton.Current.BoundingRectangle
    [ManageItemsPopupTestNative]::SetCursorPos(
        [int]($bounds.Left + ($bounds.Width / 2)),
        [int]($bounds.Top + ($bounds.Height / 2))) | Out-Null
    [ManageItemsPopupTestNative]::mouse_event([ManageItemsPopupTestNative]::MouseLeftDown, 0, 0, 0, [UIntPtr]::Zero)
    [ManageItemsPopupTestNative]::mouse_event([ManageItemsPopupTestNative]::MouseLeftUp, 0, 0, 0, [UIntPtr]::Zero)

    $popupMarker = Wait-ForElementByName -Root $desktop -ProcessCondition $processCondition -Name 'Remove multi1.txt'
    if ($null -eq $popupMarker) {
        throw "Manage Items popup did not open."
    }

    [ManageItemsPopupTestNative]::SetForegroundWindow($anchorWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 750

    $foregroundProcessId = [uint32]0
    [ManageItemsPopupTestNative]::GetWindowThreadProcessId(
        [ManageItemsPopupTestNative]::GetForegroundWindow(),
        [ref]$foregroundProcessId) | Out-Null
    if ($foregroundProcessId -ne $shellProcessId) {
        throw "Could not activate the external application; the feedback loop is invalid."
    }

    $popupMarker = Find-ElementByName -Root $desktop -ProcessCondition $processCondition -Name 'Remove multi1.txt'
    if ($null -eq $popupMarker) {
        throw "FAIL: Manage Items popup closed when another application became active."
    }

    Write-Output "PASS: Manage Items popup remained open after another application became active ($ItemCount items)."
}
finally {
    if ($null -ne $dropCove -and -not $dropCove.HasExited) {
        Stop-Process -Id $dropCove.Id -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -Path $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
