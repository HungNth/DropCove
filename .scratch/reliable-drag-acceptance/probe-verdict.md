# Native OLE acceptance probe — failed gate

Date: 2026-10-05. Ticket 01. Production drag-out unchanged.

> **Correction:** This is the historical first-run report. Its false-positive inference is superseded by [the controlled destination follow-up](controlled-probe-verdict.md): it instrumented the primary target but did not exclude acceptance by a target underneath. Keep the measured tuples and raw logs; do not use the old inference as proof that native OLE or WinUI misclassified the complete drag. The design gate remains blocked pending isolated installed reproduction.

## Verdict

`DoDragDrop` returning `DRAGDROP_S_DROP` (0x00040100) with `DROPEFFECT_COPY` (1) does **not** distinguish accepted Copy from a destination exiting during DragEnter. Both returned this exact tuple in 5/5 correlated attempts. No production cutover is authorized by this evidence; Tickets 02–04 remain blocked and ADR 0003 remains proposed.

| Outcome | Attempts | HRESULT | Effect | Target Drop entered | Predicate accepted |
|---|---:|---|---:|---|---|
| Copy | 5/5 | 0x00040100 | 1 | Yes; exact offered path received | Yes |
| Reject | 5/5 | 0x00040100 | 0 | No | No |
| Esc cancel | 5/5 | 0x00040101 | 0 | No | No |
| Exit in DragEnter before setting effect | 5/5 | 0x00040100 | 1 | No | **Yes: false positive** |

Full correlated source/target events: [probe-evidence.json](probe-evidence.json). Attempt IDs are copy-01-retry, copy-02..05, reject-01..05, cancel-01..05, fail-01..05. A preliminary copy-01 driver failure is also preserved: relative SendInput movement never reached the target, no DragEnter occurred, and source returned DROP/NONE. It is not counted as a controlled Copy outcome; no successful-target failure was hidden.

## Harness and verification

- Isolated temporary .NET 10 Windows Forms WinExe project outside the repository and `src`; no new package, workload, or developer tool. Existing Windows Desktop framework only.
- Source invokes ole32 DoDragDrop directly through managed COM/P/Invoke, using a real CF_HDROP DataObject and custom IDropSource; allowed operation is Copy only.
- Controlled target uses Windows Forms' real OLE drop registration. Logs DragEnter before effect assignment, Drop entry with received paths, and Drop handler return effect. These are managed target handler observations, not a trace of the framework's internal IDropTarget implementation.
- Fail target flushes its intentional-exit log then calls Environment.Exit(0) inside DragEnter, before assigning an effect. No Drop event follows. Copy target receives the exact offered fixture path and returns Copy.
- Processes launched with CREATE_NO_WINDOW. Real SendInput absolute movement crosses source/target windows; SendInput Escape cancels. Fixture contents remain unchanged. Every source and target exited; temporary harness removed after capture.
- `dotnet build Probe.csproj -c Release`: 0 warnings, 0 errors. Twenty real OLE modal drags exercised. Python assertions verified every result tuple, required target events, Copy path identity, and absence of Drop for non-success outcomes.
- No production build, installed-app interaction, profile modification, or production tests required for this failed external gate. Full production suite intentionally not run: no production change and no cutover.

## Facts versus inference

Measured: the source return tuple collides; the disappearing target has no Drop entry. API identifiers above name constants; the probe does not claim any constant guarantees destination acceptance. Why Copy survives target exit is **not established** by this probe; cached-effect behavior is a hypothesis, not evidence. No reliable alternative predicate was established.

Setting NONE before exit would change the required scenario (exit before assigning any effect), so it cannot repair this gate. An out-of-band acknowledgement requires cooperating destinations and would change the approved Explorer/browser/editor/chat compatibility contract. Neither is shipped or substituted.

## Reproduction

Create a temporary directory outside the repository. Save the project and source below, run `dotnet build Probe.csproj -c Release`, create a real fixture file, then launch two instances for each attempt:

```text
Probe.exe target <copy|reject|cancel|fail> <attempt-id> <target.jsonl> <fixture-path>
Probe.exe source <copy|reject|cancel|fail> <attempt-id> <source.jsonl> <fixture-path>
```

Press and hold left mouse in the source client area (window at 100,180), move into target client area (window at 600,180), then release; for cancel, press Escape before release. Run five attempts per mode, using unique JSONL paths. Compare source result against correlated target Drop entry and returned effect, never copied-file existence. Helpers must run without consoles; close both windows after each attempt.

### Probe.csproj

```xml
<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>WinExe</OutputType><TargetFramework>net10.0-windows</TargetFramework><UseWindowsForms>true</UseWindowsForms><ImplicitUsings>enable</ImplicitUsings><Nullable>enable</Nullable></PropertyGroup></Project>
```

### Program.cs (evidence-only reproduction listing; not production code)

```csharp
using System.Runtime.InteropServices;
using ComData = System.Runtime.InteropServices.ComTypes.IDataObject;

internal static class Program
{
    [DllImport("ole32.dll", PreserveSig = true)]
    private static extern int DoDragDrop(ComData data, IDropSource source, uint allowed, out uint effect);
    [STAThread]
    private static void Main(string[] args)
    {
        Application.OleRequired();
        string role = args[0], mode = args[1], attempt = args[2], log = args[3], path = args[4];
        void Record(string kind, object? details = null) => File.AppendAllText(log,
            System.Text.Json.JsonSerializer.Serialize(new { attempt, role, mode, kind, details }) + "\n");
        using var form = new Form { Text = $"OLE {role} {attempt}", StartPosition = FormStartPosition.Manual,
            Location = role == "source" ? new Point(100, 180) : new Point(600, 180),
            ClientSize = new Size(300, 240), TopMost = true, AllowDrop = role == "target" };
        form.Shown += (_, _) => Record("ready", new { hwnd = form.Handle.ToInt64() });
        if (role == "source")
        {
            form.MouseDown += (_, e) =>
            {
                if (e.Button != MouseButtons.Left) return;
                var data = new DataObject();
                data.SetData(DataFormats.FileDrop, new[] { path });
                Record("start", new { paths = new[] { path }, allowed = 1 });
                int hr = DoDragDrop((ComData)data, new DropSource(), 1, out uint effect);
                Record("result", new { hr, effect, accepted = hr == 0x40100 && effect == 1 });
            };
        }
        else
        {
            form.DragEnter += (_, e) =>
            {
                Record("DragEnter");
                if (mode == "fail") { Record("intentional_exit_before_effect"); Environment.Exit(0); }
                e.Effect = mode == "reject" ? DragDropEffects.None : DragDropEffects.Copy;
                Record("DragEnter_return", new { effect = (int)e.Effect });
            };
            form.DragOver += (_, e) => { e.Effect = mode == "reject" ? DragDropEffects.None : DragDropEffects.Copy; Record("DragOver", new { effect = (int)e.Effect }); };
            form.DragLeave += (_, _) => Record("DragLeave");
            form.DragDrop += (_, e) =>
            {
                Record("Drop_enter", new { paths = e.Data?.GetData(DataFormats.FileDrop) as string[] });
                e.Effect = DragDropEffects.Copy;
                Record("Drop_return", new { effect = (int)e.Effect });
            };
        }
        Application.Run(form);
    }
}

[ComVisible(true), Guid("00000121-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IDropSource
{
    [PreserveSig] int QueryContinueDrag([MarshalAs(UnmanagedType.Bool)] bool escape, uint keys);
    [PreserveSig] int GiveFeedback(uint effect);
}
[ComVisible(true), ClassInterface(ClassInterfaceType.None)]
internal sealed class DropSource : IDropSource
{
    public int QueryContinueDrag(bool escape, uint keys) => escape ? 0x40101 : (keys & 1) == 0 ? 0x40100 : 0;
    public int GiveFeedback(uint effect) => 0x40102;
}
```
