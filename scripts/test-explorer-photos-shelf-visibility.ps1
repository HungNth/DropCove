[CmdletBinding()]
param(
    [string]$ExecutablePath = ""
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($ExecutablePath)) {
    $ExecutablePath = Join-Path $PSScriptRoot "..\src\DropCove.App\bin\x64\Debug\net10.0-windows10.0.22000.0\win-x64\DropCove.exe"
}

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class ExplorerPhotosShelfTestNative
{
    [StructLayout(LayoutKind.Sequential)]
    public struct Rect
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out Rect rect);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(System.Drawing.Point point);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extraInfo);
    [DllImport("user32.dll")] public static extern void keybd_event(byte virtualKey, byte scanCode, uint flags, UIntPtr extraInfo);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint message, IntPtr wParam, IntPtr lParam);

    public const uint WmClose = 0x0010;
    public const uint MouseLeftDown = 0x0002;
    public const uint MouseLeftUp = 0x0004;
    public const byte VkMenu = 0x12;
    public const byte VkF4 = 0x73;
    public const uint KeyUp = 0x0002;
}
'@ -ReferencedAssemblies System.Drawing.Primitives

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

function Wait-ForWindowVisibility {
    param(
        [IntPtr]$WindowHandle,
        [bool]$Visible,
        [int]$TimeoutSeconds = 10
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        if ([ExplorerPhotosShelfTestNative]::IsWindowVisible($WindowHandle) -eq $Visible) {
            return $true
        }

        Start-Sleep -Milliseconds 100
    } while ((Get-Date) -lt $deadline)

    return $false
}

function Close-ViewerWindow {
    param([IntPtr]$WindowHandle)

    [ExplorerPhotosShelfTestNative]::SetForegroundWindow($WindowHandle) | Out-Null
    Start-Sleep -Milliseconds 100
    if ([ExplorerPhotosShelfTestNative]::GetForegroundWindow() -ne $WindowHandle) {
        throw "Windows Photos could not be foregrounded for Alt+F4."
    }

    [ExplorerPhotosShelfTestNative]::keybd_event([ExplorerPhotosShelfTestNative]::VkMenu, 0, 0, [UIntPtr]::Zero)
    [ExplorerPhotosShelfTestNative]::keybd_event([ExplorerPhotosShelfTestNative]::VkF4, 0, 0, [UIntPtr]::Zero)
    [ExplorerPhotosShelfTestNative]::keybd_event([ExplorerPhotosShelfTestNative]::VkF4, 0, [ExplorerPhotosShelfTestNative]::KeyUp, [UIntPtr]::Zero)
    [ExplorerPhotosShelfTestNative]::keybd_event([ExplorerPhotosShelfTestNative]::VkMenu, 0, [ExplorerPhotosShelfTestNative]::KeyUp, [UIntPtr]::Zero)
}


$resolvedExecutable = Resolve-Path $ExecutablePath -ErrorAction SilentlyContinue
if ($null -eq $resolvedExecutable) {
    throw "DropCove executable not found at: $ExecutablePath. Build the application first."
}

$python = Get-Command python.exe -ErrorAction SilentlyContinue
if ($null -eq $python) {
    $python = Get-Command py.exe -ErrorAction SilentlyContinue
}
if ($null -eq $python) {
    throw "Python 3 is required for the disposable SQLite profile."
}

$runId = [Guid]::NewGuid().ToString('N')
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) "DropCove-ExplorerPhotos-$runId"
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
& $python.Source $seedScript $dbPath $fixturesRoot | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Failed to seed the disposable DropCove profile."
}

$minimizeScript = @"
import sqlite3

conn = sqlite3.connect(r'''$dbPath''')
conn.execute('PRAGMA foreign_keys = ON')
batch_id = conn.execute("SELECT batch_id FROM shelf_items WHERE name = 'Preview.png'").fetchone()[0]
conn.execute('DELETE FROM shelf_batches WHERE id <> ?', (batch_id,))
conn.execute('UPDATE shelf_batches SET position = 0 WHERE id = ?', (batch_id,))
conn.commit()
conn.close()
"@
& $python.Source -c $minimizeScript
if ($LASTEXITCODE -ne 0) {
    throw "Failed to minimise the disposable DropCove profile."
}

$dropCove = $null
$explorerWindowHandle = [IntPtr]::Zero
$viewerWindowHandle = [IntPtr]::Zero
try {
    $desktop = [System.Windows.Automation.AutomationElement]::RootElement
    $existingPhotos = $desktop.FindAll(
        [System.Windows.Automation.TreeScope]::Children,
        [System.Windows.Automation.Condition]::TrueCondition) | Where-Object {
            $process = Get-Process -Id $_.Current.ProcessId -ErrorAction SilentlyContinue
            $null -ne $process -and
            [ExplorerPhotosShelfTestNative]::IsWindowVisible([IntPtr]::new($_.Current.NativeWindowHandle)) -and
            ($process.ProcessName -match 'Photos|ApplicationFrameHost' -or $_.Current.Name -match 'Photos')
        }
    if ($existingPhotos.Count -gt 0) {
        throw "Close existing Windows Photos windows before running this destructive UI smoke."
    }

    $dropCove = Start-Process -FilePath $resolvedExecutable -ArgumentList "--test-profile `"$profileRoot`"" -PassThru
    $processCondition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ProcessIdProperty,
        $dropCove.Id
    )

    if ($null -eq (Wait-ForElementByName -Root $desktop -ProcessCondition $processCondition -Name 'Preview.png')) {
        throw "Preview.png did not appear in the Drop Shelf."
    }
    if ($null -eq (Wait-ForElementByName -Root $desktop -ProcessCondition $processCondition -Name 'Hide DropCove')) {
        throw "The visible Drop Shelf marker did not appear."
    }

    $dropCove.Refresh()
    $shelfWindowHandle = $dropCove.MainWindowHandle
    if ($shelfWindowHandle -eq [IntPtr]::Zero) {
        throw "The Drop Shelf window handle is unavailable."
    }


    Start-Process explorer.exe -ArgumentList "/new,`"$fixturesRoot`""

    $explorerImageElement = $null
    $deadline = (Get-Date).AddSeconds(15)
    do {
        $windows = $desktop.FindAll(
            [System.Windows.Automation.TreeScope]::Children,
            [System.Windows.Automation.Condition]::TrueCondition)
        foreach ($window in $windows) {
            if ($window.Current.ClassName -ne 'CabinetWClass') {
                continue
            }

            $items = $window.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                [System.Windows.Automation.Condition]::TrueCondition)
            foreach ($item in $items) {
                if ($item.Current.Name -eq 'Preview.png' -or $item.Current.Name -eq 'Preview') {
                    $explorerWindowHandle = [IntPtr]::new($window.Current.NativeWindowHandle)
                    $explorerImageElement = $item
                    break
                }
            }
            if ($null -ne $explorerImageElement) {
                break
            }
        }

        if ($null -eq $explorerImageElement) {
            Start-Sleep -Milliseconds 100
        }
    } while ($null -eq $explorerImageElement -and (Get-Date) -lt $deadline)

    if ($null -eq $explorerImageElement) {
        throw "Explorer did not display Preview.png from the isolated fixture folder."
    }


    $imageBounds = $explorerImageElement.Current.BoundingRectangle
    if ($imageBounds.IsEmpty -or $imageBounds.Width -le 0 -or $imageBounds.Height -le 0) {
        throw "Explorer's Preview.png item has no clickable bounds."
    }

    [ExplorerPhotosShelfTestNative]::SetCursorPos(
        [int]($imageBounds.Left + ($imageBounds.Width / 2)),
        [int]($imageBounds.Top + ($imageBounds.Height / 2))) | Out-Null
    [ExplorerPhotosShelfTestNative]::mouse_event([ExplorerPhotosShelfTestNative]::MouseLeftDown, 0, 0, 0, [UIntPtr]::Zero)
    [ExplorerPhotosShelfTestNative]::mouse_event([ExplorerPhotosShelfTestNative]::MouseLeftUp, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 100
    if ([ExplorerPhotosShelfTestNative]::GetForegroundWindow() -ne $explorerWindowHandle) {
        throw "Clicking Preview.png did not activate Explorer; the feedback loop is invalid."
    }
    [ExplorerPhotosShelfTestNative]::mouse_event([ExplorerPhotosShelfTestNative]::MouseLeftDown, 0, 0, 0, [UIntPtr]::Zero)
    [ExplorerPhotosShelfTestNative]::mouse_event([ExplorerPhotosShelfTestNative]::MouseLeftUp, 0, 0, 0, [UIntPtr]::Zero)

    $viewerProcess = $null
    $deadline = (Get-Date).AddSeconds(15)
    do {
        $foregroundHandle = [ExplorerPhotosShelfTestNative]::GetForegroundWindow()
        if ($foregroundHandle -ne [IntPtr]::Zero -and $foregroundHandle -ne $explorerWindowHandle) {
            $foregroundProcessId = [uint32]0
            [ExplorerPhotosShelfTestNative]::GetWindowThreadProcessId($foregroundHandle, [ref]$foregroundProcessId) | Out-Null
            $foregroundProcess = Get-Process -Id $foregroundProcessId -ErrorAction SilentlyContinue
            $foregroundElement = [System.Windows.Automation.AutomationElement]::FromHandle($foregroundHandle)
            if ($null -ne $foregroundProcess -and
                ($foregroundProcess.ProcessName -match 'Photos|ApplicationFrameHost' -or $foregroundElement.Current.Name -match 'Photos')) {
                $viewerWindowHandle = $foregroundHandle
                $viewerProcess = $foregroundProcess
                break
            }
        }

        Start-Sleep -Milliseconds 100
    } while ($viewerWindowHandle -eq [IntPtr]::Zero -and (Get-Date) -lt $deadline)

    if ($viewerWindowHandle -eq [IntPtr]::Zero) {
        throw "Double-clicking Preview.png in Explorer did not activate Windows Photos."
    }

    Close-ViewerWindow -WindowHandle $viewerWindowHandle
    if (-not (Wait-ForWindowVisibility -WindowHandle $viewerWindowHandle -Visible $false)) {
        throw "Windows Photos did not close within 10 seconds."
    }

    $deadline = (Get-Date).AddSeconds(5)
    while ([ExplorerPhotosShelfTestNative]::GetForegroundWindow() -ne $explorerWindowHandle -and (Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 100
    }
    if ([ExplorerPhotosShelfTestNative]::GetForegroundWindow() -ne $explorerWindowHandle) {
        throw "Explorer did not regain foreground after Windows Photos closed; the feedback loop is invalid."
    }

    Start-Sleep -Seconds 1

    $shelfBounds = New-Object ExplorerPhotosShelfTestNative+Rect
    if (-not [ExplorerPhotosShelfTestNative]::GetWindowRect($shelfWindowHandle, [ref]$shelfBounds)) {
        throw "Could not read the Drop Shelf bounds."
    }
    $shelfCenter = [System.Drawing.Point]::new(
        [int](($shelfBounds.Left + $shelfBounds.Right) / 2),
        [int](($shelfBounds.Top + $shelfBounds.Bottom) / 2))
    $coveringWindow = [ExplorerPhotosShelfTestNative]::WindowFromPoint($shelfCenter)
    $coveringProcessId = [uint32]0
    [ExplorerPhotosShelfTestNative]::GetWindowThreadProcessId($coveringWindow, [ref]$coveringProcessId) | Out-Null

    $shelfMarker = Find-ElementByName -Root $desktop -ProcessCondition $processCondition -Name 'Hide DropCove'
    $shelfVisible = [ExplorerPhotosShelfTestNative]::IsWindowVisible($shelfWindowHandle)
    $shelfMarkerExists = $null -ne $shelfMarker
    $shelfMarkerOffscreen = $shelfMarkerExists -and $shelfMarker.Current.IsOffscreen
    if (-not $shelfVisible -or -not $shelfMarkerExists -or $shelfMarkerOffscreen -or $coveringProcessId -ne $dropCove.Id) {
        throw "FAIL: Drop Shelf was covered after Windows Photos closed via Alt+F4 (visible=$shelfVisible; markerExists=$shelfMarkerExists; markerOffscreen=$shelfMarkerOffscreen; coveringHwnd=$coveringWindow; coveringPid=$coveringProcessId)."
    }

    Write-Output "PASS: Drop Shelf remained visible over Explorer after Windows Photos closed via Alt+F4 (viewer: $($viewerProcess.ProcessName))."
}
finally {
    if ($viewerWindowHandle -ne [IntPtr]::Zero -and [ExplorerPhotosShelfTestNative]::IsWindowVisible($viewerWindowHandle)) {
        [ExplorerPhotosShelfTestNative]::PostMessage(
            $viewerWindowHandle,
            [ExplorerPhotosShelfTestNative]::WmClose,
            [IntPtr]::Zero,
            [IntPtr]::Zero) | Out-Null
    }
    if ($explorerWindowHandle -ne [IntPtr]::Zero -and [ExplorerPhotosShelfTestNative]::IsWindowVisible($explorerWindowHandle)) {
        [ExplorerPhotosShelfTestNative]::PostMessage(
            $explorerWindowHandle,
            [ExplorerPhotosShelfTestNative]::WmClose,
            [IntPtr]::Zero,
            [IntPtr]::Zero) | Out-Null
    }
    if ($null -ne $dropCove -and -not $dropCove.HasExited) {
        Stop-Process -Id $dropCove.Id -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -Path $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
