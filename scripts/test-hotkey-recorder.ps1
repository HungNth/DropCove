[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ExecutablePath,
    [string]$ArtifactPath = '',
    [switch]$Basic
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Threading;
public static class RecorderInput {
    [StructLayout(LayoutKind.Sequential)] public struct Mouse { public int x,y; public uint data,flags,time; public UIntPtr extra; }
    [StructLayout(LayoutKind.Sequential)] public struct Keyboard { public ushort key,scan; public uint flags,time; public UIntPtr extra; }
    [StructLayout(LayoutKind.Explicit)] public struct Union { [FieldOffset(0)] public Mouse mouse; [FieldOffset(0)] public Keyboard keyboard; }
    [StructLayout(LayoutKind.Sequential)] public struct Input { public uint type; public Union data; }
    [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr window,uint message,IntPtr w,IntPtr l);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
    [DllImport("user32.dll")] public static extern IntPtr GetKeyboardLayout(uint thread);
    [DllImport("user32.dll")] public static extern uint MapVirtualKeyEx(uint key,uint map,IntPtr layout);
    [DllImport("user32.dll", SetLastError=true)] public static extern bool RegisterHotKey(IntPtr window,int id,uint modifiers,uint key);
    [DllImport("user32.dll")] public static extern bool UnregisterHotKey(IntPtr window,int id);
    public static void Key(ushort key,bool up) {
        bool extended = key==0xA3 || key==0xA5 || key==0x5B || key==0x5C || key==0x6F || (key>=0x21 && key<=0x2E);
        var input = new Input { type=1 };
        input.data.keyboard.key=key;
        input.data.keyboard.flags=(up?2u:0u)|(extended?1u:0u);
        if(SendInput(1,new[]{input},Marshal.SizeOf<Input>())!=1) throw new Win32Exception(Marshal.GetLastWin32Error());
        Thread.Sleep(45);
    }
    public static void Chord(ushort[] modifiers,ushort key) {
        try { foreach(var m in modifiers) Key(m,false); Key(key,false); Key(key,true); }
        finally { for(int i=modifiers.Length-1;i>=0;i--) Key(modifiers[i],true); }
    }
    public sealed class Reservation : IDisposable {
        readonly ManualResetEventSlim ready=new(false),stop=new(false);
        readonly Thread thread;
        public Reservation(uint modifiers,uint key) {
            int error=0;
            thread=new Thread(()=>{
                bool ok=RegisterHotKey(IntPtr.Zero,992,modifiers|0x4000,key);
                if(!ok) error=Marshal.GetLastWin32Error();
                ready.Set();
                if(ok) { stop.Wait(); UnregisterHotKey(IntPtr.Zero,992); }
            });
            thread.IsBackground=true; thread.Start();
            if(!ready.Wait(5000)) throw new TimeoutException("Hotkey reservation thread did not start.");
            if(error!=0) { Dispose(); throw new Win32Exception(error); }
        }
        public void Dispose() { stop.Set(); if(!thread.Join(5000)) throw new TimeoutException("Hotkey reservation did not end."); ready.Dispose();stop.Dispose(); }
    }
}
'@

$exe = (Resolve-Path $ExecutablePath).Path
$profile = Join-Path ([IO.Path]::GetTempPath()) ('DropCove-Recorder-' + [guid]::NewGuid().ToString('N'))
$settingsPath = Join-Path $profile 'settings.json'
$desktop = [Windows.Automation.AutomationElement]::RootElement
$script:app = $null
$script:main = $null
$script:settings = $null
$script:activeKey = 0x79
$script:activeMods = [ushort[]]@(0x11,0x10)
$script:activeLabel = 'Ctrl + Shift + F10'
$reservation = $null
$lock = $null
$report = [ordered]@{
    Executable = $exe
    ExecutableSHA256 = (Get-FileHash $exe -Algorithm SHA256).Hash
    ApplicationSHA256 = (Get-FileHash (Join-Path (Split-Path $exe) 'DropCove.dll') -Algorithm SHA256).Hash
    WindowsBuild = [Environment]::OSVersion.Version.ToString()
    KeyboardLayout = [RecorderInput]::GetKeyboardLayout(0).ToInt64().ToString('X')
    Mode = $(if($Basic){'Basic'}else{'Full'})
    Started = (Get-Date).ToString('o')
    Scenarios = [Collections.Generic.List[object]]::new()
    Unobserved = @('AltGr on a non-US layout','High Contrast: no unsafe host theme mutation','Non-100-percent DPI')
    Passed = $false
    Cleanup = $null
}

function Check($condition,[string]$message) { if(-not $condition) { throw $message } }
function Wait([scriptblock]$condition,[string]$description) {
    $deadline=(Get-Date).AddSeconds(8)
    do { $value=& $condition; if($value){return $value}; Start-Sleep -Milliseconds 100 } while((Get-Date)-lt $deadline)
    throw "Timed out: $description"
}
function Note([string]$name,$details) {
    $report.Scenarios.Add([ordered]@{Name=$name;Passed=$true;Details=$details})
    Write-Host "PASS: $name"
}
function ById($root,[string]$id) {
    $root.FindFirst([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::AutomationIdProperty,$id))
}
function ByName($root,[string]$name) {
    $root.FindFirst([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,$name))
}
function Invoke($element) { Check ($null-ne $element) 'UI action unavailable'; $element.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke(); Start-Sleep -Milliseconds 180 }
function Foreground($window) {
    [RecorderInput]::SetForegroundWindow($window)|Out-Null
    Wait { [RecorderInput]::GetForegroundWindow()-eq $window } 'owned foreground' | Out-Null
}
function Press([ushort[]]$mods,[ushort]$key) { [RecorderInput]::Chord($mods,$key); Start-Sleep -Milliseconds 180 }
function Recorder { ById $script:settings 'GlobalHotkeyRecorder' }
function SaveButton { ById $script:settings 'SaveSettings' }
function Selected([string]$label) {
    $r=Recorder
    Check ($r.Current.ItemStatus-eq 'Shortcut selected') "Recorder still listening: $($r.Current.Name)"
    Check ($r.Current.Name-eq "Global hotkey, $label") "Expected $label; got $($r.Current.Name)"
}
function Start-Capture {
    Foreground ([IntPtr]$script:settings.Current.NativeWindowHandle)
    (Recorder).SetFocus()
    Press @() 0x0D
    Check ((Recorder).Current.ItemStatus-eq 'Listening') 'Enter activation did not start listening'
    Check (-not (SaveButton).Current.IsEnabled) 'Save enabled during listening'
}
function Record([ushort[]]$mods,[ushort]$key,[string]$label) {
    Start-Capture
    Press $mods $key
    Selected $label
    Check ((SaveButton).Current.IsEnabled) 'Save disabled after capture'
}
function StartApp {
    $script:app=Start-Process $exe -ArgumentList "--test-profile `"$profile`"" -PassThru
    $script:main=Wait {
        $condition=[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ProcessIdProperty,$script:app.Id)
        $windows=$desktop.FindAll([Windows.Automation.TreeScope]::Children,$condition)
        foreach($window in $windows) { if($window.Current.Name-eq 'DropCove — Test profile'){return $window} }
    } 'test-profile main window'
}
function StopApp {
    if($script:app-and -not $script:app.HasExited) {
        [RecorderInput]::PostMessage([IntPtr]$script:main.Current.NativeWindowHandle,0x8002,[IntPtr]::Zero,[IntPtr]::Zero)|Out-Null
        Check ($script:app.WaitForExit(5000)) 'Resident did not shut down cleanly'
    }
}
function Toggle {
    $hwnd=[IntPtr]$script:main.Current.NativeWindowHandle
    $before=[RecorderInput]::IsWindowVisible($hwnd)
    Press $script:activeMods $script:activeKey
    Wait { [RecorderInput]::IsWindowVisible($hwnd)-ne $before } 'saved shortcut toggles Drop Shelf'|Out-Null
}
function OpenSettings {
    param([switch]$FromTray)
    $hwnd=[IntPtr]$script:main.Current.NativeWindowHandle
    if ($FromTray) {
        Invoke (ByName $script:main 'Hide DropCove')
        # Deliver the existing Shell tray callback, then exercise the real native menu.
        [RecorderInput]::PostMessage($hwnd,0x8001,[IntPtr]::Zero,[IntPtr]0x0205)|Out-Null
        $menuItem=Wait {
            $condition=[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ProcessIdProperty,$script:app.Id)
            $items=$desktop.FindAll([Windows.Automation.TreeScope]::Descendants,$condition)
            foreach($item in $items){if($item.Current.ControlType-eq [Windows.Automation.ControlType]::MenuItem -and $item.Current.Name-eq 'Settings'){return $item}}
        } 'native tray Settings menu item'
        Invoke $menuItem
    } else {
        if(-not [RecorderInput]::IsWindowVisible($hwnd)){Toggle}
        Invoke (ByName $script:main 'Settings')
    }
    $script:settings=Wait {
        $condition=[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ProcessIdProperty,$script:app.Id)
        $windows=$desktop.FindAll([Windows.Automation.TreeScope]::Children,$condition)
        foreach($window in $windows){if($window.Current.Name-eq 'DropCove Settings'){return $window}}
    } 'Settings window'
    Foreground ([IntPtr]$script:settings.Current.NativeWindowHandle)
}
function SaveDraft([ushort[]]$mods,[ushort]$key,[string]$label) {
    $hwnd=[IntPtr]$script:settings.Current.NativeWindowHandle
    Invoke (SaveButton)
    Wait { -not [RecorderInput]::IsWindowVisible($hwnd) } 'successful Save closes Settings'|Out-Null
    $persisted=Get-Content $settingsPath -Raw|ConvertFrom-Json
    Check ($persisted.HotKey.VirtualKey-eq $key) 'Wrong durable virtual key'
    Check ($persisted.HotKey.Control-eq ($mods-contains 0x11) -and $persisted.HotKey.Alt-eq ($mods-contains 0x12) -and $persisted.HotKey.Shift-eq ($mods-contains 0x10) -and $persisted.HotKey.Windows-eq ($mods-contains 0x5B)) 'Wrong durable modifiers'
    $script:activeKey=$key; $script:activeMods=$mods; $script:activeLabel=$label
    Toggle
    StopApp
    StartApp
    Toggle
    OpenSettings
    Selected $label
}

try {
    New-Item $profile -ItemType Directory|Out-Null
    @{ HotKey=@{Control=$true;Alt=$false;Shift=$true;Windows=$false;VirtualKey=0x79};StartWithWindows=$false;ShakeEnabled=$false;ShakeSensitivity='Normal';RailEdge='Right';RailMonitorId='' }|ConvertTo-Json|Set-Content $settingsPath -Encoding utf8
    StartApp
    OpenSettings
    Selected $script:activeLabel
    Check ((Recorder).Current.HelpText-like '*Enter*Space*') 'Recorder help unavailable'
    Check ((Recorder).Current.IsKeyboardFocusable) 'Recorder not focusable'
    Note 'Initial accessible recorder' (Recorder).Current.Name

    $previous=ById $script:settings 'ShakeSensitivityComboBox'
    $previous.SetFocus()
    Press @() 0x09
    Check ((Recorder).Current.HasKeyboardFocus) 'Tab did not reach recorder'
    Selected $script:activeLabel
    Start-Capture
    Start-Sleep -Seconds 2
    Check ((Recorder).Current.ItemStatus-eq 'Listening') 'Listening timed out'
    Check ((Recorder).Current.Name-notlike '*Enter') 'Activation key was recorded'
    [RecorderInput]::Key(0x11,$false)
    Check ((Recorder).Current.Name-like '*Ctrl +*') 'Held Ctrl not displayed'
    [RecorderInput]::Key(0x11,$true)
    Press @() 0x1B
    Selected $script:activeLabel
    Note 'Tab, Enter activation exclusion, modifier display and Escape' 'Listening then restored saved shortcut'

    (Recorder).SetFocus(); Press @() 0x20
    Check ((Recorder).Current.ItemStatus-eq 'Listening') 'Space activation did not listen'
    Press @() 0x4B
    Check ((Recorder).Current.ItemStatus-eq 'Listening') 'Bare key captured'
    Check ($null-ne (ByName $script:settings 'Use at least one modifier and a supported key.')) 'Inline validation missing'
    Press @() 0x09
    Selected $script:activeLabel
    Check (-not (Recorder).Current.HasKeyboardFocus) 'Tab focus trapped'
    Note 'Space activation, invalid input and bare-Tab cancellation' 'Draft retained; focus moved'

    # Real pointer activation and pointer focus-loss cancellation.
    (Recorder).SetFocus()
    $bounds=(Recorder).Current.BoundingRectangle
    [RecorderInput]::SetCursorPos([int]($bounds.Left+$bounds.Width/2),[int]($bounds.Top+$bounds.Height/2))|Out-Null
    [RecorderInput]::mouse_event(2,0,0,0,[UIntPtr]::Zero); [RecorderInput]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
    Wait { (Recorder).Current.ItemStatus-eq 'Listening' } 'pointer starts capture'|Out-Null
    $cancel=ByName $script:settings 'Cancel'; $bounds=$cancel.Current.BoundingRectangle
    # Focus another control without activating Cancel.
    $toggle=ById $script:settings 'StartWithWindowsToggle'; $toggle.SetFocus()
    Selected $script:activeLabel
    Note 'Pointer activation and control focus loss' 'Listening canceled without changing draft'

    Start-Capture
    $mainHandle=[IntPtr]$script:main.Current.NativeWindowHandle
    $visible=[RecorderInput]::IsWindowVisible($mainHandle)
    Press $script:activeMods $script:activeKey
    Selected $script:activeLabel
    Check ([RecorderInput]::IsWindowVisible($mainHandle)-eq $visible) 'Current chord also toggled shelf during capture'
    Note 'Saved-chord suppression including completion event' 'Recorder captured saved chord; shelf unchanged'

    Record @(0x11,0x10) 0x4B 'Ctrl + Shift + K'
    $diskBefore=Get-Content $settingsPath -Raw
    Press @(0x11,0x10) 0x4B
    Check ([RecorderInput]::IsWindowVisible($mainHandle)-eq $visible) 'Unsaved shortcut active'
    Toggle
    Check ((Get-Content $settingsPath -Raw)-eq $diskBefore) 'Draft changed durable settings'
    Foreground ([IntPtr]$script:settings.Current.NativeWindowHandle)
    SaveDraft @(0x11,0x10) 0x4B 'Ctrl + Shift + K'
    Note 'Draft, Save, numeric persistence, clean restart and activation' $script:activeLabel

    # Registration is reserved AFTER capture so the foreign registration cannot consume capture input.
    Record @(0x11,0x10) 0x7B 'Ctrl + Shift + F12'
    $diskBefore=Get-Content $settingsPath -Raw
    $reservation=[RecorderInput+Reservation]::new(6,0x7B)
    Invoke (SaveButton)
    Selected 'Ctrl + Shift + F12'
    $message="DropCove couldn’t register this hotkey. It may be unavailable or reserved by Windows. Choose another."
    Check ($null-ne (ByName $script:settings $message)) 'Accurate registration warning not displayed'
    Check ((Get-Content $settingsPath -Raw)-eq $diskBefore) 'Conflict changed durable settings'
    Toggle
    $reservation.Dispose(); $reservation=$null
    Note 'Real native rejection' 'Warning visible; draft retained; durable settings unchanged; prior hotkey works'

    Record @(0x11,0x10) 0x79 'Ctrl + Shift + F10'
    $diskBefore=Get-Content $settingsPath -Raw
    $lock=[IO.File]::Open($settingsPath,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
    Invoke (SaveButton)
    Check ([RecorderInput]::IsWindowVisible([IntPtr]$script:settings.Current.NativeWindowHandle)) 'Save failure closed Settings'
    $errors=$script:settings.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
    Check (@($errors|Where-Object {$_.Current.Name-like 'Settings could not be saved:*'}).Count-gt 0) 'Save error missing'
    $lock.Dispose();$lock=$null
    Check ((Get-Content $settingsPath -Raw)-eq $diskBefore) 'Persistence failure changed settings'
    Toggle
    $mainHandle=[IntPtr]$script:main.Current.NativeWindowHandle
    $visible=[RecorderInput]::IsWindowVisible($mainHandle)
    Press @(0x11,0x10) 0x79
    Check ([RecorderInput]::IsWindowVisible($mainHandle)-eq $visible) 'Failed shortcut retained registration'
    Note 'Deterministic persistence failure rollback' 'Prior registration works; new registration absent; durable JSON unchanged'

    if(-not $Basic) {
        $punctuation=[char]([RecorderInput]::MapVirtualKeyEx(0xBA,2,[RecorderInput]::GetKeyboardLayout(0))-band 0xFFFF)
        $cases=@(
            @{Key=0x09;Mods=@(0x11);Label='Ctrl + Tab'},
            @{Key=0x0D;Mods=@(0x11);Label='Ctrl + Enter'},
            @{Key=0x20;Mods=@(0x11);Label='Ctrl + Space'},
            @{Key=0x35;Mods=@(0x11,0x10);Label='Ctrl + Shift + 5'},
            @{Key=0x08;Mods=@(0x11,0x10);Label='Ctrl + Shift + Backspace'},
            @{Key=0x22;Mods=@(0x11,0x10);Label='Ctrl + Shift + Page Down'},
            @{Key=0x7D;Mods=@(0x11,0x10);Label='Ctrl + Shift + F14'},
            @{Key=0x61;Mods=@(0x11,0x12);Label='Ctrl + Alt + Num 1'},
            @{Key=0x6B;Mods=@(0x11,0x12);Label='Ctrl + Alt + Num +'},
            @{Key=0xBA;Mods=@(0x11,0x10);Label="Ctrl + Shift + $punctuation"},
            @{Key=0x7C;Mods=@(0x10,0x5B);Label='Shift + Win + F13'}
        )
        foreach($case in $cases) {
            Record $case.Mods $case.Key $case.Label
            SaveDraft $case.Mods $case.Key $case.Label
            Note ('Capture/save/restart/activate '+$case.Label) ('VK='+$case.Key)
        }
        Record @(0x11,0xA4) 0x7E 'Ctrl + Alt + F15'
        Record @(0x11,0xA5) 0x7E 'Ctrl + Alt + F15'
        Record @(0x10,0x5B) 0x7D 'Shift + Win + F14'
        Record @(0x10,0x5C) 0x7D 'Shift + Win + F14'
        Record @(0xA2,0xA0) 0x4A 'Ctrl + Shift + J'
        Record @(0xA3,0xA1) 0x4A 'Ctrl + Shift + J'
        Note 'Left/right aggregate modifiers' 'Ctrl, Alt, Shift and Win left/right variants produce identical normalized chords'
        Start-Capture;Press @() 0x13
        Check ((Recorder).Current.ItemStatus-eq 'Listening') 'Pause accepted'
        Check ($null-ne (ByName $script:settings 'Use at least one modifier and a supported key.')) 'Unsupported input warning missing'
        Press @() 0x1B;Selected 'Ctrl + Shift + J'
        Note 'Unsupported Pause rejected' 'Listening retained; prior draft restored by Escape'
        Start-Capture
        Press @(0x5B) 0x52
        $foreground=[RecorderInput]::GetForegroundWindow()
        Check ($foreground-ne [IntPtr]$script:settings.Current.NativeWindowHandle) 'Reserved Win+R did not move focus; probe unproven'
        $run=[Windows.Automation.AutomationElement]::FromHandle($foreground)
        Check ($run.Current.Name-eq 'Run') 'Unexpected foreground after Win+R; refusing arbitrary Escape'
        Press @() 0x1B
        Foreground ([IntPtr]$script:settings.Current.NativeWindowHandle)
        Selected 'Ctrl + Shift + J'
        Note 'Windows-reserved Win+R' 'Run took focus; recorder restored prior draft; Run closed'
    }

    Start-Capture
    Invoke (ByName $script:settings 'Cancel')
    Check (-not [RecorderInput]::IsWindowVisible([IntPtr]$script:settings.Current.NativeWindowHandle)) 'Cancel did not close listening Settings'
    OpenSettings;Selected $script:activeLabel
    Start-Capture
    [RecorderInput]::PostMessage([IntPtr]$script:settings.Current.NativeWindowHandle,0x0010,[IntPtr]::Zero,[IntPtr]::Zero)|Out-Null
    Wait {-not [RecorderInput]::IsWindowVisible([IntPtr]$script:settings.Current.NativeWindowHandle)} 'window close during listening'|Out-Null
    Note 'Cancel/window close during listening' 'Discarded unsaved capture and Settings draft'

    # Prove startup conflict survival with the saved shortcut reserved by this other process.
    StopApp
    $flags=0
    if($script:activeMods-contains 0x11){$flags=$flags-bor 2};if($script:activeMods-contains 0x12){$flags=$flags-bor 1}
    if($script:activeMods-contains 0x10){$flags=$flags-bor 4};if($script:activeMods-contains 0x5B){$flags=$flags-bor 8}
    $reservation=[RecorderInput+Reservation]::new($flags,$script:activeKey)
    StartApp
    OpenSettings -FromTray
    Selected $script:activeLabel
    Check (-not $script:app.HasExited) 'Startup conflict killed resident'
    Note 'Startup conflict survival and tray Settings recovery' 'Resident responsive; actual tray menu Settings opens recorder with persisted chord unchanged (balloon rendering not claimed)'
    $reservation.Dispose();$reservation=$null

    # The unchanged loader can accept a pre-existing modifierless definition.
    # Recording still validates this native WM_HOTKEY path, not only WinUI keydown.
    StopApp
    @{ HotKey=@{Control=$false;Alt=$false;Shift=$false;Windows=$false;VirtualKey=0x7D};StartWithWindows=$false;ShakeEnabled=$false;ShakeSensitivity='Normal';RailEdge='Right';RailMonitorId='' }|ConvertTo-Json|Set-Content $settingsPath -Encoding utf8
    $script:activeMods=[ushort[]]@();$script:activeKey=0x7D;$script:activeLabel='F14'
    StartApp;OpenSettings
    Start-Capture
    $mainHandle=[IntPtr]$script:main.Current.NativeWindowHandle
    $visible=[RecorderInput]::IsWindowVisible($mainHandle)
    Press @() 0x7D
    Check ((Recorder).Current.ItemStatus-eq 'Listening') 'Native registered input bypassed modifier validation'
    Check ($null-ne (ByName $script:settings 'Use at least one modifier and a supported key.')) 'Native invalid-input guidance missing'
    Check ([RecorderInput]::IsWindowVisible($mainHandle)-eq $visible) 'Invalid native input toggled shelf while recording'
    Press @() 0x1B
    Toggle
    Note 'Native registered input obeys recorder validation' 'Modifierless legacy input rejected during recording; its real global registration still toggles the shelf after capture cancellation'
    $report.Passed=$true
}
catch {
    $report.Failure=$_.Exception.Message
    throw
}
finally {
    if($lock){$lock.Dispose()}
    if($reservation){$reservation.Dispose()}
    if($script:app-and -not $script:app.HasExited){Stop-Process -Id $script:app.Id -Force}
    if(Test-Path $profile){Remove-Item $profile -Recurse -Force}
    $report.Cleanup=@{ProfileRemoved=(-not(Test-Path $profile));OwnedProcessStopped=($null-eq $script:app-or $script:app.HasExited)}
    $report.Finished=(Get-Date).ToString('o')
    if($ArtifactPath){$report|ConvertTo-Json -Depth 8|Set-Content $ArtifactPath -Encoding utf8}
}
