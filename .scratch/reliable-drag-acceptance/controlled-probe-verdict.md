# Controlled destination investigation — correction to initial gate verdict

Date: 2026-10-05. Ticket 01 follow-up. Production source and user profile unchanged.

> **Later follow-up:** The user subsequently approved the test-profile seam, and the actual DropCove side-by-side Release matrix was exercised. See [real-app verdict](installed-profile-verdict.md). This document preserves the earlier isolated-probe measurements and the profile prerequisite as they stood before that approval; its “not run” statements are historical, not the latest verification status.

## Decision

Do not start native production cutover from this evidence. The controlled native probe passes the four required outcomes, but an equivalent WinUI source passes the same matrix. The earlier inference that source Copy after primary-target exit proves a false-positive is unsupported: it did not instrument or exclude the destination exposed underneath the primary target.

This does **not** retroactively prove which destination handled the historical installed DropCove attempt. That observation remains recorded. It requires a controlled installed reproduction before deciding whether a product defect remains or a native engine is necessary. No responsive-shelf acceptance, performance, or release criterion is marked complete by this isolated comparison.

## Measured results

All matrix rows contain five distinct, correlated attempts. Source and every controlled destination have separate event streams. The backing window is registered before the primary target at the same native rectangle; primary and source are launched afterward. After primary exit, the pointer remains over the controlled backing surface.

| Primary outcome; backing Reject | Native source result (5/5) | WinUI DropCompleted (5/5) | Acceptance oracle |
|---|---|---|---|
| Copy | DROP / COPY | Copy | Primary Drop entry, exact fixture path, return S_OK/COPY |
| Reject | DROP / NONE | None | No Drop in primary or backing |
| Esc cancel | CANCEL / NONE | None | No Drop in primary or backing |
| Exit during DragEnter before effect assignment | DROP / NONE | None | Primary intentional exit; backing enters and returns NONE; neither receives Drop |

The entire native 5x4 matrix was run with Reject backing, not just the disappearing case. The entire WinUI 5x4 matrix was also run with Reject backing. A second controlled matrix variant (five native and five WinUI attempts) placed a Copy backing behind the disappearing primary:

- Primary exits before returning DragEnter; primary receives no Drop.
- Backing receives Drop, the exact offered fixture path, and returns S_OK/COPY.
- Native source returns DROP/COPY; WinUI returns Copy.
- This is **genuine successful acceptance by the backing destination**, not acceptance by the failed primary and not evidence of a stale primary effect.

## Diagnostics and hypotheses

1. **Input effect is Copy — confirmed.** Direct IDropTarget::DragEnter logs effect=1 before the target writes it. Effect on entry is not an acceptance acknowledgement.
2. **WinForms target wrapper is necessary for the old tuple — rejected.** A directly registered IDropTarget also returns source DROP/COPY in five uncontrolled-backing primary-exit attempts. However those attempts have no complete destination oracle and cannot prove false acceptance.
3. **Effect survives failures as universal stale-success behavior — not established.** The Reject backing makes the required disappearing case return NONE in five native and five WinUI attempts. An explicit E_FAIL return from DragEnter under uncontrolled backing yielded four DROP/NONE and one DROP/COPY; all retained, no cherry-picking. None is proof of which uninstrumented destination handled the operation.

Additional diagnostics, **not substituted for the four required scenarios**:

| Native diagnostic, uncontrolled backing | Recorded results | Scope |
|---|---|---|
| Target sets ref effect=NONE then exits inside DragEnter | 5/5 DROP/COPY | No callback return; backing not instrumented. Cannot assert final accepted destination. |
| Target returns E_FAIL from DragEnter | 4 DROP/NONE, 1 DROP/COPY | Uncontrolled backing; inconsistent source tuple does not identify final destination. |
| Target exits inside Drop before accepting/returning | 5/5 HRESULT 0x800706BE, effect COPY | HRESULT is failure, predicate rejects; target Drop entry alone is not acceptance. |

No cached-effect mechanism inside OLE was proved. No target polling, output-file inference, timeout, destination cooperation protocol, or production heuristic was introduced.

## Evidence and verification

[Full event streams](controlled-probe-evidence.json) preserve 85 successful driver runs: 35 direct native diagnostics without controlled backing; 20 native Reject-backing matrix runs; five native Copy-backing runs; 20 WinUI Reject-backing matrix runs; five WinUI Copy-backing runs. Two additional driver setup failures are retained separately: one source missed MouseDown before an OLE call; one driver incorrectly compared a WinUI child HWND to its root HWND before starting a gesture. Corrected driver checks native bounds and GetAncestor(GA_ROOT).

- Native project: final Release build 0 warnings, 0 errors. Initial build's WFDEV005 warning was corrected using GetFileDropList rather than obsolete untyped GetData.
- WinUI comparison source: unpackaged self-contained Release publish succeeded; real window Loaded/ready, native HWND hit checks, actual DragStarting/DropCompleted and target callbacks verified. Uses the existing project's WindowsAppSDK 2.5.1 and SDK BuildTools 10.0.28000.2705; no new dependency or tool installed.
- `dotnet new winui` template was not installed. The minimal comparison project was composed with the current repository's verified project/manifest conventions, outside the repository; no template or workload installed.
- WinUI payload preparation matches production DragDropService: DragStarting deferral, StorageFile.GetFileFromPathAsync, SetStorageItems(readOnly:true), RequestedOperation=Copy, AllowedOperations=Copy. This is an isolated surface, **not the installed DropCove popup or rail**.
- Hidden helper processes, absolute SendInput, real left-button gesture, actual Escape key input. No SetCursorPos-only drag driver.
- Assertions checked all 20 native controlled classifications, all 25 WinUI classifications, exact payload paths for all accepted backing runs, and no backing Drop in Reject-backing runs. Original fixture SHA-256 was recorded before gestures and unchanged afterward.
- All helper processes exited; disposable source/build/publish workspace removed after evidence capture. No production build, production tests, installed DropCove mutation, or profile replacement.

## Installed reproduction prerequisite

The installed comparison has **not** run. MainWindow.xaml.cs builds its database path from `Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData)` plus `DropCove/shelf.db`; SettingsStore.cs uses the same folder for settings. Search of App/Services/Core found no command-line or environment profile override. A separate disposable WinUI process was launched with LOCALAPPDATA pointing at an existing temporary directory. Its startup log reported:

```text
LOCALAPPDATA = C:\Users\Hung\AppData\Local\Temp\DropCove-Direct-Ole-tq5qujm_\isolated-local-data
GetFolderPath(LocalApplicationData) = C:\Users\Hung\AppData\Local
```

Overriding that environment variable therefore does not safely isolate the actual .NET runtime's folder lookup. No installed DropCove process was started for the probe, no live-profile reference was added or removed, and no profile was swapped. To continue installed verification safely, approve a scoped explicit profile-directory seam (used by database and settings, with default behavior unchanged), or provide an already isolated Windows user environment. This is a planning prerequisite, not an implementation added by this investigation.

App.xaml.cs also uses the fixed `Local\DropCove.SingleInstance` mutex and `Local\DropCove.Activate` event. An approved same-user test-profile mode must account for that instance boundary and avoid changing the live profile's Windows startup registration/settings; a directory override alone is not a complete isolation design. No such mode was added here.

## Reproduction

The following code listings are evidence-only; they are not shipping projects. Recreate them in a temporary directory outside the repository. Run:

```text
dotnet build Probe.csproj -c Release
dotnet publish WinUIProbe/WinUIProbe.csproj -c Release -p:Platform=x64 -o WinUIProbe/publish
```

Create a real fixture path. For each attempt, launch backing first and wait for ready, then primary and wait for ready, then source and wait for ready. Helpers must be hidden (CREATE_NO_WINDOW), with distinct JSONL paths and matching attempt identifiers:

```text
Probe.exe target reject <id>-backing <backing.jsonl> <fixture>
Probe.exe target <copy|reject|cancel|fail> <id> <primary.jsonl> <fixture>
Probe.exe source <mode> <id> <source.jsonl> <fixture>
```

For WinUI comparison, replace only the third process:

```text
WinUIProbe.exe <id> <source.jsonl> <fixture>
```

Source is at (100,180), targets at (600,180). Use native GetWindowRect centers, foreground source, verify the root HWND under the pointer, hold left mouse, move at least 20 pixels to start WinUI DragStarting, then move to the primary target. For cancel send Escape before release; for other cases release. Compare **all destination** event streams with the source result. Run five attempts per mode. For the accepting-backing diagnostic replace the backing mode with copy and primary mode with fail; Copy is accepted only if that backing logs Drop path receipt and S_OK/COPY return. Close every process before the next attempt.

### Native Probe.csproj

```xml
<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>WinExe</OutputType><TargetFramework>net10.0-windows</TargetFramework><UseWindowsForms>true</UseWindowsForms><ImplicitUsings>enable</ImplicitUsings><Nullable>enable</Nullable></PropertyGroup></Project>
```

### Native Program.cs

```csharp
using System.Runtime.InteropServices;
using ComData = System.Runtime.InteropServices.ComTypes.IDataObject;

internal static class Program
{
    [DllImport("ole32.dll", PreserveSig = true)] private static extern int DoDragDrop(ComData data, IDropSource source, uint allowed, out uint effect);
    [DllImport("ole32.dll", PreserveSig = true)] private static extern int RegisterDragDrop(nint hwnd, IDropTarget target);
    [DllImport("ole32.dll", PreserveSig = true)] private static extern int RevokeDragDrop(nint hwnd);
    [STAThread]
    private static void Main(string[] args)
    {
        Application.OleRequired();
        string role = args[0], mode = args[1], attempt = args[2], log = args[3], path = args[4];
        void Record(string kind, object? details) => File.AppendAllText(log,
            System.Text.Json.JsonSerializer.Serialize(new { attempt, role, mode, kind, details }) + "\n");
        using var form = new Form { Text = $"Direct OLE {role} {attempt}", StartPosition = FormStartPosition.Manual,
            Location = role == "source" ? new Point(100, 180) : new Point(600, 180),
            ClientSize = new Size(300, 240), TopMost = true, AllowDrop = false };
        DropTarget? target = null;
        form.Shown += (_, _) =>
        {
            if (role == "target")
            {
                target = new DropTarget(mode, Record);
                int hr = RegisterDragDrop(form.Handle, target);
                Record("registration", new { hr });
                Marshal.ThrowExceptionForHR(hr);
            }
            Record("ready", new { hwnd = form.Handle.ToInt64() });
        };
        form.FormClosed += (_, _) => { if (target != null) Record("revoked", new { hr = RevokeDragDrop(form.Handle) }); };
        if (role == "source")
        {
            form.MouseDown += (_, e) =>
            {
                if (e.Button != MouseButtons.Left) return;
                var data = new DataObject();
                data.SetData(DataFormats.FileDrop, new[] { path });
                Record("start", new { paths = new[] { path }, allowed = 1 });
                int hr = DoDragDrop((ComData)data, new DropSource(Record), 1, out uint effect);
                Record("result", new { hr, effect, accepted = hr == 0x40100 && effect == 1 });
            };
        }
        Application.Run(form);
        GC.KeepAlive(target);
    }
}
[StructLayout(LayoutKind.Sequential)]
internal struct PointL { public int X; public int Y; }
[ComVisible(true), Guid("00000122-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IDropTarget
{
    [PreserveSig] int DragEnter(ComData data, uint keys, PointL point, ref uint effect);
    [PreserveSig] int DragOver(uint keys, PointL point, ref uint effect);
    [PreserveSig] int DragLeave();
    [PreserveSig] int Drop(ComData data, uint keys, PointL point, ref uint effect);
}
[ComVisible(true), ClassInterface(ClassInterfaceType.None)]
internal sealed class DropTarget(string mode, Action<string, object?> record) : IDropTarget
{
    public int DragEnter(ComData data, uint keys, PointL point, ref uint effect)
    {
        record("DragEnter_entry", new { effect, keys, point.X, point.Y });
        if (mode == "fail" || mode == "fail-none")
        {
            if (mode == "fail-none") { effect = 0; record("effect_set_none_before_exit", new { effect }); }
            record("intentional_exit", null);
            Environment.Exit(0);
        }
        if (mode == "error") { record("DragEnter_error", new { hr = unchecked((int)0x80004005), effect }); return unchecked((int)0x80004005); }
        effect = mode == "reject" ? 0u : 1u;
        record("DragEnter_return", new { hr = 0, effect });
        return 0;
    }
    public int DragOver(uint keys, PointL point, ref uint effect)
    {
        record("DragOver_entry", new { effect });
        effect = mode == "reject" ? 0u : 1u;
        record("DragOver_return", new { hr = 0, effect });
        return 0;
    }
    public int DragLeave() { record("DragLeave", null); return 0; }
    public int Drop(ComData data, uint keys, PointL point, ref uint effect)
    {
        record("Drop_entry", new { effect });
        if (mode == "fail-drop") { record("intentional_exit", null); Environment.Exit(0); }
        var wrapper = new DataObject(data);
        var paths = wrapper.GetFileDropList().Cast<string>().ToArray();
        record("Drop_paths", new { paths });
        effect = 1;
        record("Drop_return", new { hr = 0, effect });
        return 0;
    }
}
[ComVisible(true), Guid("00000121-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IDropSource
{
    [PreserveSig] int QueryContinueDrag([MarshalAs(UnmanagedType.Bool)] bool escape, uint keys);
    [PreserveSig] int GiveFeedback(uint effect);
}
[ComVisible(true), ClassInterface(ClassInterfaceType.None)]
internal sealed class DropSource(Action<string, object?> record) : IDropSource
{
    public int QueryContinueDrag(bool escape, uint keys)
    {
        int hr = escape ? 0x40101 : (keys & 1) == 0 ? 0x40100 : 0;
        if (hr != 0) record("QueryContinueDrag_terminal", new { escape, keys, hr });
        return hr;
    }
    public int GiveFeedback(uint effect) { record("GiveFeedback", new { effect }); return 0x40102; }
}
```

### WinUIProbe/WinUIProbe.csproj

```xml
<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>WinExe</OutputType><TargetFramework>net10.0-windows10.0.22000.0</TargetFramework><TargetPlatformMinVersion>10.0.22000.0</TargetPlatformMinVersion><RootNamespace>WinUIProbe</RootNamespace><ApplicationManifest>app.manifest</ApplicationManifest><Platforms>x64</Platforms><PlatformTarget>x64</PlatformTarget><RuntimeIdentifier>win-x64</RuntimeIdentifier><SelfContained>true</SelfContained><WindowsPackageType>None</WindowsPackageType><WindowsAppSDKSelfContained>true</WindowsAppSDKSelfContained><UseWinUI>true</UseWinUI><WinUISDKReferences>false</WinUISDKReferences><EnableMsixTooling>true</EnableMsixTooling><ImplicitUsings>enable</ImplicitUsings><Nullable>enable</Nullable></PropertyGroup><ItemGroup><Manifest Include="$(ApplicationManifest)"/><PackageReference Include="Microsoft.Windows.SDK.BuildTools" Version="10.0.28000.2705"/><PackageReference Include="Microsoft.WindowsAppSDK" Version="2.5.1"/></ItemGroup></Project>
```

### WinUIProbe/app.manifest

```xml
<?xml version="1.0" encoding="utf-8"?><assembly manifestVersion="1.0" xmlns="urn:schemas-microsoft-com:asm.v1"><assemblyIdentity version="1.0.0.0" name="WinUIProbe.app"/><compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1"><application><supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/></application></compatibility><application xmlns="urn:schemas-microsoft-com:asm.v3"><windowsSettings><dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness></windowsSettings></application></assembly>
```

### WinUIProbe/App.xaml

```xml
<Application x:Class="WinUIProbe.App" xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"><Application.Resources><ResourceDictionary><ResourceDictionary.MergedDictionaries><XamlControlsResources xmlns="using:Microsoft.UI.Xaml.Controls"/></ResourceDictionary.MergedDictionaries></ResourceDictionary></Application.Resources></Application>
```

### WinUIProbe/App.xaml.cs

```csharp
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using System.Runtime.InteropServices;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;

namespace WinUIProbe;
public partial class App : Application
{
    private Window? _window;
    [DllImport("user32.dll")] private static extern bool SetWindowPos(nint hwnd, nint after, int x, int y, int cx, int cy, uint flags);
    public App() => InitializeComponent();
    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        var cli = Environment.GetCommandLineArgs();
        string attempt = cli[1], log = cli[2], path = cli[3];
        void Record(string kind, object? details) => File.AppendAllText(log,
            System.Text.Json.JsonSerializer.Serialize(new { attempt, role = "winui-source", kind, details }) + "\n");
        _window = new Window { Title = $"WinUI source {attempt}" };
        var root = new Border { CanDrag = true, Background = new SolidColorBrush(Microsoft.UI.Colors.SteelBlue), Child = new TextBlock { Text = "Drag fixture", HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center } };
        _window.Content = root;
        nint hwnd = WinRT.Interop.WindowNative.GetWindowHandle(_window);
        _window.AppWindow.MoveAndResize(new Windows.Graphics.RectInt32(100, 180, 316, 280));
        SetWindowPos(hwnd, -1, 0, 0, 0, 0, 0x13);
        root.Loaded += (_, _) => Record("ready", new { hwnd = hwnd.ToInt64() });
        root.DragStarting += async (_, e) =>
        {
            var deferral = e.GetDeferral();
            try
            {
                var file = await StorageFile.GetFileFromPathAsync(path);
                e.Data.SetStorageItems(new[] { file }, readOnly: true);
                e.Data.RequestedOperation = DataPackageOperation.Copy;
                e.AllowedOperations = DataPackageOperation.Copy;
                Record("start", new { paths = new[] { path } });
            }
            catch (Exception error) { e.Cancel = true; Record("error", new { error.Message }); }
            finally { deferral.Complete(); }
        };
        root.DropCompleted += (_, e) => Record("result", new { effect = (int)e.DropResult });
        _window.Activate();
    }
}
```
