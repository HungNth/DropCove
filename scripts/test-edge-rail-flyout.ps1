[CmdletBinding()]
param(
    [string]$ExecutablePath = ""
)

if ([string]::IsNullOrWhiteSpace($ExecutablePath)) {
    $ExecutablePath = Join-Path $PSScriptRoot "..\src\DropCove.App\bin\Debug\net10.0-windows10.0.22000.0\win-x64\DropCove.exe"
}

$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class EdgeRailTestNative
{
    [DllImport("user32.dll")]
    public static extern IntPtr SendMessage(IntPtr windowHandle, uint message, IntPtr wParam, IntPtr lParam);
}
'@

$resolvedExe = Resolve-Path $ExecutablePath -ErrorAction SilentlyContinue
if ($null -eq $resolvedExe) {
    throw "DropCove executable not found at: $ExecutablePath. Build the application first."
}

$runId = [Guid]::NewGuid().ToString('N')
$tempDir = Join-Path ([IO.Path]::GetTempPath()) "DropCove-EdgeRailTest-$runId"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
$dbPath = Join-Path $tempDir 'shelf.db'

$python = Get-Command python.exe -ErrorAction SilentlyContinue
if ($null -eq $python) {
    $python = Get-Command py.exe -ErrorAction SilentlyContinue
}
if ($null -eq $python) {
    throw "Python 3 is required for disposable SQLite seed."
}

$seedScript = @"
import sqlite3
import uuid
from datetime import datetime, timezone

def dotnet_roundtrip_utc():
    now = datetime.now(timezone.utc)
    return now.strftime('%Y-%m-%dT%H:%M:%S.') + f'{now.microsecond:06d}0+00:00'

conn = sqlite3.connect(r'$dbPath')
conn.execute('PRAGMA foreign_keys = ON')
conn.execute('PRAGMA journal_mode = WAL')
conn.executescript('''
    CREATE TABLE IF NOT EXISTS shelf_batches (
        id TEXT PRIMARY KEY,
        created_at TEXT NOT NULL,
        position INTEGER NOT NULL UNIQUE
    );
    CREATE TABLE IF NOT EXISTS shelf_items (
        id TEXT PRIMARY KEY,
        batch_id TEXT NOT NULL REFERENCES shelf_batches(id) ON DELETE CASCADE,
        position INTEGER NOT NULL,
        path TEXT NOT NULL,
        name TEXT NOT NULL,
        is_folder INTEGER NOT NULL CHECK (is_folder IN (0, 1)),
        is_pinned INTEGER NOT NULL CHECK (is_pinned IN (0, 1)),
        UNIQUE (batch_id, position)
    );
''')
batch_id = str(uuid.uuid4())
conn.execute('INSERT INTO shelf_batches VALUES (?, ?, 0)', (batch_id, dotnet_roundtrip_utc()))
conn.execute('INSERT INTO shelf_items VALUES (?, ?, 0, ?, ?, 0, 0)', (str(uuid.uuid4()), batch_id, r'C:\test\Item1.txt', 'Item1.txt'))
conn.execute('INSERT INTO shelf_items VALUES (?, ?, 1, ?, ?, 0, 0)', (str(uuid.uuid4()), batch_id, r'C:\test\Item2.txt', 'Item2.txt'))
conn.commit()
conn.close()
"@

& $python.Source -c $seedScript

$process = Start-Process -FilePath $resolvedExe -ArgumentList "--test-profile `"$tempDir`"" -PassThru

try {
    Start-Sleep -Seconds 3

    $root = [System.Windows.Automation.AutomationElement]::RootElement
    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ProcessIdProperty,
        $process.Id
    )

    # 1. Hide DropCove to show Edge Rail
    $hideButton = $null
    $elements = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
    foreach ($el in $elements) {
        if ($el.Current.Name -eq "Hide DropCove") {
            $hideButton = $el
            break
        }
    }

    if ($null -eq $hideButton) {
        throw "Could not find 'Hide DropCove' button"
    }

    $pattern = $null
    if ($hideButton.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$pattern)) {
        ([System.Windows.Automation.InvokePattern]$pattern).Invoke()
    } else {
        throw "'Hide DropCove' does not support InvokePattern"
    }

    Start-Sleep -Seconds 1

    # 2. Find and invoke 'Show items in 2 items' flyout button
    $flyoutButton = $null
    $elements = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
    foreach ($el in $elements) {
        if ($el.Current.Name -like "Show items in*") {
            $flyoutButton = $el
            break
        }
    }

    if ($null -eq $flyoutButton) {
        throw "Could not find 'Show items in...' button"
    }

    $flyoutPattern = $null
    if ($flyoutButton.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$flyoutPattern)) {
        ([System.Windows.Automation.InvokePattern]$flyoutPattern).Invoke()
    } else {
        $flyoutButton.SetFocus()
        [System.Windows.Forms.SendKeys]::SendWait(" ")
    }

    Start-Sleep -Seconds 1

    # 3. Assert items rendered inside flyout
    $allElements = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
    $foundItems = @()
    foreach ($el in $allElements) {
        if ($el.Current.Name -like "Drag Item*") {
            $foundItems += $el.Current.Name
        }
    }

    if ($foundItems.Count -ne 2) {
        throw "Expected 2 items in flyout, but found $($foundItems.Count): $($foundItems -join ', ')"
    }

    Write-Output "PASS: Edge Rail flyout expanded and displayed 2 batch items ($($foundItems -join ', '))."
}
finally {
    if ($process -and -not $process.HasExited) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}
