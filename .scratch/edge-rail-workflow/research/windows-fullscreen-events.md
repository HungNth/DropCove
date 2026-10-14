# Windows Foreground, Fullscreen, and Display Reflow Technical Research

This document establishes the technical research for Ticket 04 (`Fullscreen, Display Reflow and Settings Transaction`) in DropCove. It grounds:
1. **Geometric Monitor Coverage vs Work Area**: Determining true fullscreen occlusions without app-name or title heuristics.
2. **WinEvent Hook Design**: Efficient filtering for foreground change (`EVENT_SYSTEM_FOREGROUND`) and location/size reflow (`EVENT_OBJECT_LOCATIONCHANGE`), avoiding hook storms and focus deadlocks.
3. **Display & DPI Event Routing**: Handling `WM_DISPLAYCHANGE`, monitor enumeration, and DPI changes with non-destructive fallback.
4. **Window Qualification & Cloak Filtering**: Excluding shell surfaces, taskbars, cloaked UWP windows, minimized states, and DropCove's own windows (`main`, `edge-rail`, `settings`, `manage_items`).
5. **Effective External Foreground Policy during Settings Configuration**: Evaluating whether a candidate or active rail is occluded when the DropCove Settings window is currently foreground.
6. **Active Native OLE `DoDragDrop` Survival**: Preserving the drag message pump and native data transfer without cancellation or drop-source destruction during rail transitions.

---

## 1. Geometric Fullscreen Coverage vs Work Area

### 1.1 The Definition of Fullscreen Occlusion
Ticket 04 specifies:
> "With fullscreen overlay Off, a foreground window that geometrically covers the selected monitor hides requested rail presentation; application identity/name is irrelevant. Fullscreen coverage on a different monitor does not hide the selected rail."

A window is considered **fullscreen on a monitor** if and only if its physical window rectangle (`GetWindowRect`) completely covers or exceeds the physical **full monitor bounding rectangle** (`MONITORINFO.rcMonitor`), **NOT** the work area (`MONITORINFO.rcWork`).
- `rcWork` excludes the taskbar and docked appbars (e.g. standard maximized windows cover `rcWork`, not `rcMonitor`).
- Standard maximized windows must **never** be considered fullscreen.
- Games, video players, remote desktop viewers, and presentation software in borderless or exclusive fullscreen expand to cover `rcMonitor` (often extending from `(rcMonitor.left, rcMonitor.top)` to `(rcMonitor.right, rcMonitor.bottom)` or slightly outside due to invisible window sizing frames).

### 1.2 Comparison: Full Monitor `rcMonitor` vs Work Area `rcWork`

| Aspect | Full Monitor Rect (`rcMonitor`) | Work Area Rect (`rcWork`) |
| :--- | :--- | :--- |
| **Source** | `MONITORINFOEXW.monitorInfo.rcMonitor` | `MONITORINFOEXW.monitorInfo.rcWork` |
| **Taskbar / Appbars** | Included | Excluded |
| **Maximized Window Fit** | `window_rect.top >= rcMonitor.top + taskbar_height` (Does not cover) | `window_rect == rcWork` (Covers) |
| **Borderless Fullscreen Fit**| `window_rect.left <= rcMonitor.left && window_rect.top <= rcMonitor.top && window_rect.right >= rcMonitor.right && window_rect.bottom >= rcMonitor.bottom` | Spans past `rcWork` into taskbar space |
| **DropCove Rail Role** | Rail handle docks at edge of `rcWork`. When fullscreen covers `rcMonitor`, rail must hide if overlay is Off. | Rail bounds and margins are computed against `rcWork`. |

### 1.3 Exact Geometry Evaluation Function
In Windows desktop coordinate space (virtual screen coordinates), monitors may have negative coordinates or offsets. Full coverage requires checking all 4 bounds:

```rust
use windows::Win32::Foundation::RECT;

#[inline]
pub fn rect_covers_monitor(window_rect: &RECT, monitor_rect: &RECT) -> bool {
    // Window bounds must cover or exceed monitor bounds on all four edges.
    window_rect.left <= monitor_rect.left
        && window_rect.top <= monitor_rect.top
        && window_rect.right >= monitor_rect.right
        && window_rect.bottom >= monitor_rect.bottom
}
```

### 1.4 Required Extension to `DisplayMonitor`
Currently, `edge_rail::DisplayMonitor` stores only `work: WorkArea`. Ticket 04 requires storing the full monitor bounds alongside `work`:
```rust
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DisplayMonitor {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) device: String,
    pub(crate) monitor_rect: WorkArea, // or RECT: full rcMonitor (left, top, right, bottom)
    pub(crate) work: WorkArea,         // rcWork
    pub(crate) dpi: u32,
    pub(crate) primary: bool,
}
```
During `read_monitor(handle: HMONITOR)`:
```rust
let monitor_rect = info.monitorInfo.rcMonitor;
let work = info.monitorInfo.rcWork;
```

---

## 2. Windows WinEvent Hook Design & Filtering

### 2.1 SetWinEventHook APIs & Hook Flag Semantics: Why `WINEVENT_SKIPOWNPROCESS` Breaks Own Foreground Evaluation
`windows_foreground.rs` currently registers:
```rust
SetWinEventHook(
    EVENT_SYSTEM_FOREGROUND,
    EVENT_SYSTEM_FOREGROUND,
    None,
    Some(foreground_changed),
    0,
    0,
    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
)
```

#### The Critical `WINEVENT_SKIPOWNPROCESS` Pitfall:
Per Microsoft WinEvent documentation:
> `WINEVENT_SKIPOWNPROCESS` (`0x0002`): Prevents this instance of the hook from receiving the events that are generated by threads in this process.

If `WINEVENT_SKIPOWNPROCESS` is enabled on the foreground hook:
1. When an external application (e.g. fullscreen video or game) takes focus, `EVENT_SYSTEM_FOREGROUND` fires, and the rail is hidden as occluded.
2. When the user subsequently focuses DropCove's own Settings window or Drop Shelf window (`main`), Windows generates `EVENT_SYSTEM_FOREGROUND` originating from DropCove's own process.
3. Because of `WINEVENT_SKIPOWNPROCESS`, **Windows suppresses this event entirely from our hook!**
4. DropCove never receives a foreground event for its own window activation. As a result, the previous external fullscreen occlusion state remains **stale** (the rail stays hidden even though Settings is now foreground and canonical policy dictates that own-process foreground means "not covered by foreground").

#### The Resolution:
- **Remove `WINEVENT_SKIPOWNPROCESS`** from `EVENT_SYSTEM_FOREGROUND`: Hook with `WINEVENT_OUTOFCONTEXT` only.
- **Process Filtering in Geometry / Evaluation Logic, Not Event Dispatch**:
  Deliver all foreground events to the coordinator. In the coverage check:
  ```rust
  let mut foreground_pid: u32 = 0;
  unsafe { GetWindowThreadProcessId(foreground_hwnd, Some(&mut foreground_pid)) };
  if foreground_pid == unsafe { GetCurrentProcessId() } {
      // DropCove's own window (Settings, Shelf, Rail, Manage Items) is foreground.
      // By canonical definition, the rail is NOT covered by an external foreground window.
      return false;
  }
  ```
  This ensures that when Settings or Main gains focus, `foreground_changed` is invoked, recognizes own-process focus, concludes there is no external fullscreen coverage, and unhides the rail handle.

### 2.2 Avoiding Hook Storms on Location Changes
`EVENT_OBJECT_LOCATIONCHANGE` (`0x800B`) fires for *every* UI element moving or resizing across the entire operating system (mouse cursor, scrolling list items, progress bars, animations). Listening globally to `EVENT_OBJECT_LOCATIONCHANGE` without immediate, inexpensive filters will saturate the thread message loop with thousands of callbacks per second.

Furthermore, calling synchronous cross-process COM/Win32 APIs (such as `SendMessageTimeout`, `GetWindowText`, or extensive UI Automation queries) inside an out-of-context WinEvent callback can deadlock or hang if the target window is unresponsive.

#### Mitigation Strategy:
1. **Targeted Hook Range (Two Discrete Hooks)**:
   - Hook 1: `EVENT_SYSTEM_FOREGROUND` (`0x0003`) with `WINEVENT_OUTOFCONTEXT` (NO `WINEVENT_SKIPOWNPROCESS`).
   - Hook 2: `EVENT_OBJECT_LOCATIONCHANGE` (`0x800B`) with `WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS` (own window movements never represent external fullscreen coverage).
   *Do not* hook the broad range `0x0003..=0x800B`, as that captures menu popups, scrolling, dialog creation, system alerts, and object state changes across all processes.
2. **Immediate Callback Gate for Location Changes**:
   Inside the `WINEVENTPROC` callback:
   - Check `id_object == OBJID_WINDOW` (`0x00000000`). If `id_object != OBJID_WINDOW` (e.g. `OBJID_CLIENT`, `OBJID_CURSOR`, `OBJID_CARET`), exit immediately.
   - Check `id_child == INDEXID_CONTAINER` (`0`). If not 0, exit immediately.
   - For `EVENT_OBJECT_LOCATIONCHANGE`: Check if `hwnd == current_tracked_foreground_hwnd`. If `hwnd != current_tracked_foreground_hwnd`, return immediately! This reduces handling cost to a single memory comparison, discarding 99.9% of location changes.
3. **Deadlock Prevention**:
   - Use only asynchronous / non-blocking Win32 calls: `GetWindowRect`, `IsWindowVisible`, `IsIconic`, `GetWindowLongPtrW`, `DwmGetWindowAttribute`.
   - Never use `SendMessage` to foreign HWNDs.

### 2.3 Win32 Constants & Types (`windows 0.62.2`)
From `windows::Win32::UI::WindowsAndMessaging` and `windows::Win32::UI::Accessibility`:
- `EVENT_SYSTEM_FOREGROUND = 0x0003`
- `EVENT_OBJECT_LOCATIONCHANGE = 0x800B`
- `OBJID_WINDOW = 0x00000000` (defined as `0i32`)
- `INDEXID_CONTAINER = 0` (defined as `0i32`)
- `WINEVENT_OUTOFCONTEXT = 0x0000`
- `WINEVENT_SKIPOWNPROCESS = 0x0002`

## 3. Window Qualification & Cloak Filtering

Not every top-level window returned by `GetForegroundWindow()` or receiving `EVENT_SYSTEM_FOREGROUND` is an authentic user-facing fullscreen application.

### 3.1 Window State Criteria
To qualify as an occluding fullscreen window, an HWND must satisfy:
1. **Live and Visible**:
   - `IsWindow(hwnd)` is true.
   - `IsWindowVisible(hwnd)` is true.
   - `!IsIconic(hwnd)` (window must not be minimized; minimized windows report offscreen coordinates or small placeholder rects).
2. **Not Cloaked (DWM Cloaking Check)**:
   Universal Windows Platform (UWP) apps, suspended Store apps, virtual desktop inactive windows, and background ApplicationFrameWindows may remain visible in Win32 terms but are "cloaked" by Desktop Window Manager (DWM).
   Use `DwmGetWindowAttribute` with `DWMWA_CLOAKED`:
   ```rust
   use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};

   let mut cloaked: u32 = 0;
   let hr = unsafe {
       DwmGetWindowAttribute(
           hwnd,
           DWMWA_CLOAKED,
           &mut cloaked as *mut _ as *mut _,
           std::mem::size_of::<u32>() as u32,
       )
   };
   if hr.is_ok() && cloaked != 0 {
       // Window is cloaked by DWM; ignore for fullscreen occlusion.
   }
   ```
   *(Note: `DwmGetWindowAttribute` and `DWMWA_CLOAKED` require `Win32_Graphics_Dwm` in `windows 0.62.2`; see Section 7 for dependency analysis).*
3. **Excluding DropCove Surfaces**:
   DropCove's own windows must never hide the rail:
   - Shelf (`"main"`).
   - Rail Handle (`"edge-rail"`).
   - Manage Items (`"manage_items"`).
   - Settings (`"settings"`).
   - Any window with process ID == `GetCurrentProcessId()`.
4. **Shell and Desktop Windows**:
   - Desktop HWNDs: `GetDesktopWindow()` and `GetShellWindow()`.
   - Windows Shell tray: class `Shell_TrayWnd` or `Progman` / `WorkerW`.

---

## 4. Effective External Foreground Policy during Settings Configuration

### 4.1 The Settings Foreground Dilemma
When the user is in the DropCove Settings window (configuring hotkeys, docking edges, or toggling "Show over fullscreen applications"):
- The **foreground window is the DropCove Settings window itself**.
- If a fullscreen game or media player is running on the target monitor *behind* the Settings window, checking `GetForegroundWindow()` directly would find the Settings window (or a non-fullscreen window).
- Conversely, if the Settings window is opened on Monitor 1 while a fullscreen app is active on Monitor 1, the user expects "Show over fullscreen applications" preview/application to reflect whether an external fullscreen window exists on that monitor, OR evaluate what happens once Settings yields focus.

### 4.2 Canonical Resolution Rule
Ticket 04 specifies:
> "Save candidate rail/fullscreen Apply while Settings window foreground must evaluate effective external foreground expected policy (selected monitor fullscreen exists behind Settings not foreground => not full covered by foreground as canonical)."

Therefore, the canonical rule is:
1. **Definition of "Covered by Foreground"**:
   The rail is hidden (when `show_rail_over_fullscreen == false`) if and only if the **current active foreground window** is an **external** window that geometrically covers the rail's selected monitor.
2. **Settings as Foreground Window**:
   When the DropCove Settings window (or any DropCove internal window) has foreground focus:
   - The DropCove window is NOT an external occluding application.
   - An external fullscreen window that is currently *behind* the Settings window is **not** the foreground window.
   - Therefore, the selected monitor is **not** covered by foreground. The rail remains visible while Settings is active.
   - If focus transitions away from Settings back to the external fullscreen window, `EVENT_SYSTEM_FOREGROUND` fires, the external window becomes foreground, and the rail immediately hides.
3. **No Z-Order Crawling Needed**:
   We do **not** enumerate all windows down the z-order to guess what might become foreground later. Foreground policy is strictly driven by the actual OS foreground window (`GetForegroundWindow`). When DropCove owns the foreground, it evaluates as "no external foreground coverage".

---

## 5. Active Native OLE `DoDragDrop` Survival

### 5.1 OLE Modal Message Loop Dynamics
When a drag begins from the expanded rail (Ticket 03) or an item is being dragged over DropCove:
- `DoDragDrop` runs a dedicated modal message loop in `ole32.dll`.
- As the cursor moves across external application windows, Windows changes focus or active window states, and foreground events may fire.
- If an external application triggers a fullscreen or foreground change while an OLE drag is in progress, the rail presentation coordinator may receive a notification to hide.

### 5.2 Preservation Contract
Ticket 04 mandates:
> "Fullscreen presentation changes do not cancel or corrupt an already-running native drag; OLE completion remains authoritative. Fullscreen 04 later closes transients without canceling active OLE; reserve no speculative API but publish actual 03 transient close interface to parent."

1. **Hiding Presentation Without Destroying Native Drop Source**:
   - `edge_rail::hide(app)` uses `SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_HIDEWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE)`.
   - Calling `ShowWindow(hwnd, SW_HIDE)` or `SetWindowPos(SWP_HIDEWINDOW)` on the rail window **does not** terminate an in-flight `IDropSource` or `DoDragDrop` call.
   - The OLE message pump continues until the user releases the mouse button (`DRAGDROP_S_DROP`) or cancels with Escape (`DRAGDROP_S_CANCEL`).
2. **Transient Cleanup**:
   - When fullscreen triggers hiding, transient UI such as Manage Items (`manage_items::close(app)`) and Clear confirmation dialogs must be closed.
   - However, `DragSession.is_active()` must be inspected. If `is_active()` is true, the internal drag state must remain intact until `DoDragDrop` returns in its owning thread.

---

## 6. Display Connection & DPI Reflow Architecture

### 6.1 Display Events: `WM_DISPLAYCHANGE` vs `WM_DPICHANGED`
1. **`WM_DISPLAYCHANGE`**:
   - Sent to all top-level windows when display resolution, monitor topology (add/remove monitor), or color depth changes.
   - Handled in `windows_foreground.rs` via `SetWindowSubclass` on the Shelf window.
   - Subclass procedure receives:
     - `wparam`: new image depth (bits per pixel).
     - `lparam`: loword = horizontal resolution, hiword = vertical resolution.
2. **`WM_DPICHANGED`**:
   - Sent when a window moves to a monitor with a different DPI or system DPI scales.
   - For `edge_rail`, the Rail Handle is pinned to its designated monitor. When `WM_DISPLAYCHANGE` occurs, `edge_rail::monitors()` must be re-queried to fetch updated `rcMonitor`, `rcWork`, and `dpi`.

### 6.2 Monitor Resolution & Precedence Order
Ticket 04 specifies:
> "Connected configured/remembered monitor precedence and temporary primary fallback remain correct across display events without rewriting settings or remembered placement solely because of fallback."

The selection algorithm in `edge_rail::select`:
1. Check `settings.rail_monitor_id`: if connected, use it.
2. Check `remembered_rail_monitor`: if connected, use it.
3. Check `shelf_device` (monitor where Shelf was last positioned): if connected, use it.
4. Fallback to `monitors.iter().find(|m| m.primary)`.
5. Fallback to first available monitor.

**Critical Invariant**:
Fallback to primary must be purely a runtime resolution for `Placement`. It must **never** call `settings_store.save()` or alter `settings.rail_monitor_id`. If the disconnected monitor is reconnected later, subsequent `WM_DISPLAYCHANGE` will naturally re-select it.

---

## 7. Dependency & API Audit (`windows 0.62.2`)

Inspecting `src-tauri/Cargo.toml`:
```toml
windows = { version = "0.62.2", features = [
    "Win32_Foundation",
    "Win32_Globalization",
    "Win32_Graphics_Gdi",
    "Win32_UI_HiDpi",
    "Win32_UI_Accessibility",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_Storage_FileSystem",
    "Win32_System_Com",
    "Win32_System_DataExchange",
    "Win32_System_Registry",
    "Win32_System_Ole",
    "Win32_System_SystemServices",
    "Win32_System_Threading",
    "Win32_Security",
    "Win32_UI_Shell",
    "Win32_UI_Shell_Common",
    "Win32_UI_WindowsAndMessaging",
] }
```

### 7.1 Existing Enabled APIs
- `SetWinEventHook`, `UnhookWinEvent`, `HWINEVENTHOOK`, `WINEVENTPROC`: in `Win32_UI_Accessibility`.
- `GetForegroundWindow`, `GetWindowRect`, `IsWindowVisible`, `IsIconic`, `GetWindowLongPtrW`, `SetWindowPos`: in `Win32_UI_WindowsAndMessaging`.
- `EnumDisplayMonitors`, `GetMonitorInfoW`, `EnumDisplayDevicesW`, `MONITORINFOEXW`: in `Win32_Graphics_Gdi`.
- `GetDpiForMonitor`: in `Win32_UI_HiDpi`.
- `GetCurrentProcessId`, `GetCurrentThreadId`: in `Win32_System_Threading`.

### 7.2 APIs Requiring Evaluation
- `DwmGetWindowAttribute` with `DWMWA_CLOAKED`:
  Lives in `windows::Win32::Graphics::Dwm`.
  Currently, `Win32_Graphics_Dwm` is **not** listed in `Cargo.toml`.
  *Native alternatives without changing Cargo.toml*:
  1. Dynamically load `dwmapi.dll` via `LoadLibraryA` / `GetProcAddress` (safe, 10 lines of code, zero Cargo changes).
  2. Or add `"Win32_Graphics_Dwm"` to `Cargo.toml` dependencies during ticket 04 integration.

---

## 8. Concrete Recommendations for Ticket 04 Implementation

1. **Monitor Rectangle Model**:
   Update `DisplayMonitor` in `edge_rail.rs` to include `monitor_rect: WorkArea` alongside `work: WorkArea`. Capture `info.monitorInfo.rcMonitor` in `read_monitor`.
2. **Hook Range in `windows_foreground.rs`**:
   - Register `SetWinEventHook` for `EVENT_SYSTEM_FOREGROUND` (`0x0003`) with `WINEVENT_OUTOFCONTEXT` only (**remove `WINEVENT_SKIPOWNPROCESS`** so own Settings/Main foreground focus events are received).
   - Register a second `SetWinEventHook` for `EVENT_OBJECT_LOCATIONCHANGE` (`0x800B`) with `WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS`.
   - In the location change callback: immediately reject unless `id_object == OBJID_WINDOW`, `id_child == INDEXID_CONTAINER`, and `hwnd == current_tracked_foreground_hwnd`.
   - In the foreground callback: check own-process PID in the evaluation/geometry logic (`GetWindowThreadProcessId` == `GetCurrentProcessId()`). When DropCove's own window is foreground, evaluate as "not covered by foreground", clearing any previous stale fullscreen hide.
3. **Geometric Fullscreen Check**:
   Implement `is_monitor_covered_by_fullscreen(app, placement)`:
   - Query `GetForegroundWindow()`.
   - If foreground HWND belongs to own process, return `false`.
   - If `!IsWindowVisible(hwnd) || IsIconic(hwnd)`, return `false`.
   - Query `GetWindowRect(hwnd)`.
   - Compare with `placement.monitor.monitor_rect`: window must cover monitor on left, top, right, bottom.
4. **Resident Coordinator State Flow**:
   - Introduce `fullscreen_occluded: bool` in `ResidentState`.
   - When foreground changes or current foreground moves: evaluate geometric coverage.
   - If occluded and `!settings.show_rail_over_fullscreen`:
     - If rail is visible: call `edge_rail::hide(app)`, close transients (`manage_items::close`, dismiss clear prompt), and keep `Visibility::EdgeDocked` state intact.
   - When un-occluded:
     - Restore rail handle with collapsed dimensions (`expanded: false`).
