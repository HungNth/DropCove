# Windows Rail Placement & Display Identity Technical Research

This document outlines the technical research for Ticket 01 (`EdgeDocked Rail Handle and Placement Settings`) in DropCove. It establishes:
1. Persistent display identification (`RailMonitorId`) using Windows Win32 APIs and its mapping to Tauri `Monitor.name`.
2. Win32 non-activating window styles (`WS_EX_NOACTIVATE`, `WM_MOUSEACTIVATE`, `SWP_NOACTIVATE`) and interaction with child WebView2 controls.
3. Accurate 16 × 280 logical pixel placement and DPI conversion using existing project dependencies (`windows 0.62.2` and `tauri 2.12`).

---

## 1. Stable Monitor Identification: `EnumDisplayDevicesW` vs `QueryDisplayConfig`

### 1.1 The Distinction: Temporary Handles vs Volatile Adapters vs Persistent Display IDs

- **`HMONITOR`**: A transient session handle allocated by the Windows Graphics Subsystem (`win32k.sys`). It changes upon reboot, display topology reconfiguration, and driver reload. It cannot be persisted.
- **Adapter Names (`\\.\DISPLAY1`, `\\.\DISPLAY2`)**: Volatile GDI adapter names stored in `MONITORINFOEXW.szDevice`. Windows assigns these numbers dynamically based on detection order. If a cable is moved or a monitor is turned off and on, `\\.\DISPLAY1` can swap with `\\.\DISPLAY2`. **These are NOT persistent monitor identifiers.**
- **Monitor Device Interface Path (`RailMonitorId`)**: A persistent hardware interface link identifying the specific physical display endpoint. For example:
  ```text
  \\?\DISPLAY#DELD0D6#5&5839117&0&UID4354#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}
  ```
  This path contains the manufacturer/EDID model (`DELD0D6`), the hardware instance/output index (`UID4354`), and the display device interface GUID (`{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}` / `GUID_DEVINTERFACE_MONITOR`). It remains constant across reboots, display hotplugs, and cable reordering.

### 1.2 Established API Comparison

| Capability / Trait | `EnumDisplayDevicesW` + `EDD_GET_DEVICE_INTERFACE_NAME` | `QueryDisplayConfig` (`Win32_Devices_Display`) |
| :--- | :--- | :--- |
| **Header / DLL** | `winuser.h` / `user32.dll` | `winuser.h` / `user32.dll` |
| **windows crate (0.62.2)** | **Included**: `Win32_Graphics_Gdi` & `Win32_UI_WindowsAndMessaging` | **Excluded** in `Cargo.toml`: requires new feature `Win32_Devices_Display` |
| **New Dependencies** | **None** (zero new crate features or crates required) | Requires modifying `Cargo.toml` |
| **Primary Mapping** | Maps directly from `MONITORINFOEXW.szDevice` in 1 call | Requires querying active paths, matching source/target IDs, and calling `DisplayConfigGetDeviceInfo` |
| **Reliability** | Supported on Windows 7 through Windows 11 24H2+ | Supported on Windows 7+; fails in remote desktop sessions (RDP returns `ERROR_NOT_SUPPORTED`) |
| **Persistent Output** | Device interface path (`DeviceID` in `DISPLAY_DEVICEW`) | Target EDID / adapter LUID + target ID path |

**Conclusion & Recommendation:**
Use **`EnumDisplayDevicesW`** with `EDD_GET_DEVICE_INTERFACE_NAME`. It solves the problem with zero dependency changes, maps directly from `EnumDisplayMonitors` / `GetMonitorInfoW`, and is rock-solid across standard Windows desktop sessions.

### 1.3 Exact Windows 0.62.2 Signatures and Types

#### `EnumDisplayDevicesW`
Crated in `windows::Win32::Graphics::Gdi`:
```rust
pub unsafe fn EnumDisplayDevicesW<P0>(
    lpdevice: P0,
    idevnum: u32,
    lpdisplaydevice: *mut DISPLAY_DEVICEW,
    dwflags: u32,
) -> windows_core::BOOL
where
    P0: windows_core::Param<windows_core::PCWSTR>;
```

#### `EDD_GET_DEVICE_INTERFACE_NAME`
Defined in `windows::Win32::UI::WindowsAndMessaging`:
```rust
pub const EDD_GET_DEVICE_INTERFACE_NAME: u32 = 1u32;
```
*(Notice: In `windows` 0.62, this constant is placed in `UI::WindowsAndMessaging`, not `Graphics::Gdi`)*.

#### `DISPLAY_DEVICEW`
Defined in `windows::Win32::Graphics::Gdi`:
```rust
#[repr(C)]
pub struct DISPLAY_DEVICEW {
    pub cb: u32,
    pub DeviceName: [u16; 32],
    pub DeviceString: [u16; 128],
    pub StateFlags: DISPLAY_DEVICE_STATE_FLAGS,
    pub DeviceID: [u16; 128],
    pub DeviceKey: [u16; 128],
}
```

#### `MONITORINFOEXW`
Defined in `windows::Win32::Graphics::Gdi`:
```rust
#[repr(C)]
pub struct MONITORINFOEXW {
    pub monitorInfo: MONITORINFO,
    pub szDevice: [u16; 32],
}
```

#### Primary Monitor Detection
In `MONITORINFO`:
```rust
pub const MONITORINFOF_PRIMARY: u32 = 1u32;
// Check via: (mi.dwFlags & MONITORINFOF_PRIMARY) != 0
```

---

## 2. Mapping from Tauri `Monitor.name` to Persistent `RailMonitorId`

### 2.1 Tauri Monitor Name Relationship

In Tauri 2 (`tauri::Monitor`), the `name()` method delegates internally to Tao's Windows platform implementation:
```rust
// tao/src/platform_impl/windows/monitor.rs
pub fn name(&self) -> Option<String> {
    let monitor_info = get_monitor_info(self.hmonitor()).ok()?;
    Some(util::wchar_ptr_to_string(PCWSTR::from_raw(
        monitor_info.szDevice.as_ptr(),
    )))
}
```
Thus, on Windows, **`tauri::Monitor::name()` returns the adapter device name** (e.g., `Some(r"\\.\DISPLAY1")`).

### 2.2 Stable Identification Algorithm

Given an adapter name `adapter_name` (e.g. from `tauri::Monitor::name()` or from `MONITORINFOEXW.szDevice`):

```rust
use std::mem::size_of;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW};
use windows::Win32::UI::WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME;

pub fn get_persistent_monitor_id(adapter_name: &str) -> Option<String> {
    let wide_adapter: Vec<u16> = adapter_name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut display_device = DISPLAY_DEVICEW {
        cb: size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
    };

    // Query monitor interface path for the given adapter (idevnum = 0)
    let ok = unsafe {
        EnumDisplayDevicesW(
            PCWSTR(wide_adapter.as_ptr()),
            0,
            &mut display_device,
            EDD_GET_DEVICE_INTERFACE_NAME,
        )
    };

    if ok.as_bool() {
        let len = display_device.DeviceID.iter().position(|&c| c == 0).unwrap_or(display_device.DeviceID.len());
        let id = String::from_utf16_lossy(&display_device.DeviceID[..len]);
        if !id.is_empty() {
            return Some(id);
        }
    }

    // Fallback: Query PnP instance ID without EDD_GET_DEVICE_INTERFACE_NAME
    let mut fallback_device = DISPLAY_DEVICEW {
        cb: size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
    };
    let ok_fallback = unsafe {
        EnumDisplayDevicesW(
            PCWSTR(wide_adapter.as_ptr()),
            0,
            &mut fallback_device,
            0,
        )
    };
    if ok_fallback.as_bool() {
        let len = fallback_device.DeviceID.iter().position(|&c| c == 0).unwrap_or(fallback_device.DeviceID.len());
        let id = String::from_utf16_lossy(&fallback_device.DeviceID[..len]);
        if !id.is_empty() {
            return Some(id);
        }
    }

    None
}
```

---

## 3. Window Behavior: `WS_EX_NOACTIVATE`, `WM_MOUSEACTIVATE`, and Child WebView2

### 3.1 Win32 Window Styles

To ensure the Rail Handle window never steals focus from the user's active foreground window:
- Extended Style:
  - `WS_EX_NOACTIVATE` (`0x08000000`): Informs Windows that clicking the window or its non-client area does not activate it.
  - `WS_EX_TOOLWINDOW` (`0x00000080`): Prevents the rail window from appearing in the Alt+Tab task switcher.
  - `WS_EX_TOPMOST` (`0x00000008`): Keeps the rail handle floating above normal desktop windows.
- In `windows` 0.62.2:
  ```rust
  use windows::Win32::UI::WindowsAndMessaging::{
      GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
  };

  let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
  let new_style = ex_style 
      | (WS_EX_NOACTIVATE.0 as isize) 
      | (WS_EX_TOOLWINDOW.0 as isize) 
      | (WS_EX_TOPMOST.0 as isize);
  unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style) };
  ```

### 3.2 Display / Show Rules: `SWP_NOACTIVATE`

Never call `ShowWindow(hwnd, SW_SHOW)` or Tauri `window.show()` directly if it activates the window.
Instead, display or reposition the window using:
```rust
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW,
};

unsafe {
    SetWindowPos(
        hwnd,
        Some(HWND_TOPMOST),
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    )?;
}
```

### 3.3 Child WebView2 Mouse Activation & Window Subclassing

#### The Child Window Activation Problem
Tauri hosts WebView2 via an internal HWND tree:
```text
Root Tauri HWND
  └── Chrome_WidgetWin_0 (WebView2 HWND)
        └── Intermediate D3D Window
```
When a user clicks inside WebView2, Windows sends `WM_MOUSEACTIVATE` to the clicked child HWND. If the child does not explicitly consume it, the message bubbles up to the root window. If WebView2 receives focus internally, it can request top-level window activation.

#### The Solution: Subclassing the Root HWND
Subclass the root HWND using `SetWindowSubclass` (available in `windows::Win32::UI::Shell`):
```rust
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{WM_MOUSEACTIVATE, MA_NOACTIVATE};

const RAIL_SUBCLASS_ID: usize = 0x5241494C; // 'RAIL'

unsafe extern "system" fn rail_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _ref_data: usize,
) -> LRESULT {
    if msg == WM_MOUSEACTIVATE {
        // MA_NOACTIVATE (3): Do not activate this window or its parent, and do not discard the mouse message
        return LRESULT(MA_NOACTIVATE as isize);
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}
```
Returning `MA_NOACTIVATE` (`3u32`) ensures:
1. Windows will **not** activate the Rail Handle window (the active application retains foreground status).
2. The click event is **not** discarded; it is dispatched to the hit test and reaches the DOM in the WebView.

---

## 4. Tauri `WebviewWindowBuilder` Configuration

Tauri 2 (`2.12.3`) provides the following builder methods in `tauri::WebviewWindowBuilder`:

```rust
let rail_window = tauri::WebviewWindowBuilder::new(
    app_handle,
    "rail",
    tauri::WebviewUrl::App("index.html".into()), // or custom rail route
)
.title("DropCove Rail")
.decorations(false)        // Frameless / borderless
.transparent(true)          // Transparent webview background
.shadow(false)              // No OS drop shadow
.skip_taskbar(true)         // Do not show on taskbar
.always_on_top(true)        // Desktop overlay
.focused(false)             // Initial creation does not grab focus
.focusable(false)           // Prevents OS focus transfer
.resizable(false)           // Fixed size
.visible(false)             // Create hidden; position and show via SWP_NOACTIVATE
.inner_size(16.0, 280.0)    // 16x280 logical pixels
.build()?;
```

### Type Verification in Tauri 2.12.3:
- `.focused(bool)`: Controls whether the created window takes initial focus.
- `.focusable(bool)`: Windows/macOS/Linux method configuring `WS_EX_NOACTIVATE` on supported backends.
- `.decorations(bool)`, `.transparent(bool)`, `.shadow(bool)`: All accept `bool`.
- `.inner_size(f64, f64)`: Accepts logical floating-point dimensions.

---

## 5. Exact Geometry & DPI Conversion for 16 × 280 Logical Pixels

### 5.1 Native Client vs Outer Bounds
With `decorations(false)` and `shadow(false)`:
- `AdjustWindowRectEx` with `WS_POPUP` and `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` adds **0 pixels** of margin.
- Outer window rectangle (`GetWindowRect`) is identical to client rectangle (`GetClientRect`).
- Therefore, outer bounds `(width, height)` equal physical dimensions computed directly from logical pixels.

### 5.2 DPI Conversion Math
Windows standard DPI is $96$. For monitor DPI $D$:
$$\text{scale} = \frac{D}{96.0}$$
$$\text{physical\_width} = \operatorname{round}(16.0 \times \text{scale})$$
$$\text{physical\_height} = \operatorname{round}(280.0 \times \text{scale})$$

### 5.3 Vertical Centering on Work Area
For target work area `rcWork` (`left`, `top`, `right`, `bottom`):
$$\text{work\_height} = rcWork.bottom - rcWork.top$$
$$y = rcWork.top + \left\lfloor \frac{\text{work\_height} - \text{physical\_height}}{2} \right\rfloor$$

For Right Edge docking:
$$x = rcWork.right - \text{physical\_width}$$

For Left Edge docking:
$$x = rcWork.left$$

---

## 6. Official Documentation Citations & References

1. **`EnumDisplayDevicesW` function (Windows)**:
   - Microsoft Learn: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaydevicesw
   - Remarks: Specifies `EDD_GET_DEVICE_INTERFACE_NAME` (`0x00000001`) to retrieve the device interface name for `GUID_DEVINTERFACE_MONITOR`.

2. **`DISPLAY_DEVICEW` structure (Windows)**:
   - Microsoft Learn: https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-display_devicew
   - Specifies `DeviceID[128]` contains the device interface path when called with `EDD_GET_DEVICE_INTERFACE_NAME`.

3. **`WM_MOUSEACTIVATE` message (Windows)**:
   - Microsoft Learn: https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mouseactivate
   - Return values: `MA_NOACTIVATE` (`3`): "Does not activate the window, and does not discard the mouse message."

4. **Extended Window Styles (`WS_EX_NOACTIVATE`)**:
   - Microsoft Learn: https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
   - `WS_EX_NOACTIVATE` (`0x08000000`): "A top-level window created with this style does not become the foreground window when the user clicks it. The system does not bring this window to the foreground when the user minimizes or closes the foreground window."

5. **`SetWindowPos` function (Windows)**:
   - Microsoft Learn: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos
   - `SWP_NOACTIVATE` (`0x0010`): "Does not activate the window. If this flag is not set, the window is activated and moved to the top of either the topmost or non-topmost group."

6. **Tauri 2 WebviewWindow Documentation**:
   - Tauri v2 Docs: https://docs.rs/tauri/2.12.3/tauri/webview/struct.WebviewWindowBuilder.html
   - `focused`: Configures whether the window is focused upon launch.
   - `focusable`: Configures OS-level focusability (`WS_EX_NOACTIVATE`).
