//! No-activate Rail Handle and persistent display identity; visibility belongs to ResidentState.
use crate::{
    resident::{PhysicalPlacement, WorkArea},
    settings_store::{RailEdge, Settings},
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use windows::{
    core::{BOOL, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{
            EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, DISPLAY_DEVICEW, HDC,
            HMONITOR, MONITORINFOEXW,
        },
        UI::{
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
            Shell::{DefSubclassProc, SetWindowSubclass},
            WindowsAndMessaging::{
                GetCursorPos, GetWindowLongPtrW, GetWindowRect, IsWindowVisible, SetWindowLongPtrW, SetWindowPos,
                EDD_GET_DEVICE_INTERFACE_NAME, GWL_EXSTYLE, GWL_STYLE, HWND_TOPMOST, MA_NOACTIVATE,
                SWP_FRAMECHANGED, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
                SWP_NOZORDER, SWP_SHOWWINDOW, WM_MOUSEACTIVATE, WS_CAPTION, WS_EX_NOACTIVATE,
                WS_EX_TOOLWINDOW, WS_THICKFRAME,
            },
        },
    },
};

pub(crate) const LABEL: &str = "edge-rail";
const SUBCLASS: usize = 0xDC09;
pub(crate) const RAIL_TIMER_ID: usize = 0xDC10;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MonitorChoice {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) connected: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DisplayMonitor {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) device: String,
    pub(crate) monitor_rect: WorkArea,
    pub(crate) work: WorkArea,
    pub(crate) dpi: u32,
    pub(crate) primary: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Placement {
    pub(crate) monitor: DisplayMonitor,
    pub(crate) edge: RailEdge,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Projection {
    pub(crate) edge: RailEdge,
    pub(crate) expanded: bool,
    pub(crate) width: f64,
    pub(crate) height: f64,
}
fn text(buffer: &[u16]) -> String {
    String::from_utf16_lossy(&buffer[..buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len())])
}

/// # Safety
/// `data` identifies the mutable output vector for this synchronous enumeration only.
unsafe extern "system" fn collect(handle: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    // SAFETY: EnumDisplayMonitors synchronously calls with our live exclusive vector pointer.
    let output = unsafe { &mut *(data.0 as *mut Vec<Result<DisplayMonitor, String>>) };
    output.push(read_monitor(handle));
    BOOL(1)
}
fn read_monitor(handle: HMONITOR) -> Result<DisplayMonitor, String> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    // SAFETY: Live enumeration handle and fully sized MONITORINFOEXW output.
    if !unsafe { GetMonitorInfoW(handle, &mut info.monitorInfo) }.as_bool() {
        return Err("could not query rail monitor".into());
    }
    let mut device = DISPLAY_DEVICEW {
        cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
    };
    // SAFETY: szDevice is a NUL-terminated adapter name; device is the required output size.
    if !unsafe {
        EnumDisplayDevicesW(
            PCWSTR(info.szDevice.as_ptr()),
            0,
            &mut device,
            EDD_GET_DEVICE_INTERFACE_NAME,
        )
    }
    .as_bool()
    {
        return Err("could not query persistent display identity".into());
    }
    let id = text(&device.DeviceID);
    if id.is_empty() {
        return Err("display has no persistent interface identity".into());
    }
    let mut dpi = 0;
    let mut dpi_y = 0;
    // SAFETY: Monitor is connected, writable DPI outputs are valid for the synchronous call.
    unsafe { GetDpiForMonitor(handle, MDT_EFFECTIVE_DPI, &mut dpi, &mut dpi_y) }
        .map_err(|e| e.to_string())?;
    if dpi == 0 {
        return Err("rail monitor returned zero DPI".into());
    }
    let monitor_rect = info.monitorInfo.rcMonitor;
    let work = info.monitorInfo.rcWork;
    let device_name = text(&info.szDevice);
    Ok(DisplayMonitor {
        id,
        label: format!("{device_name} — {}", text(&device.DeviceString)),
        device: device_name,
        monitor_rect: WorkArea {
            left: monitor_rect.left,
            top: monitor_rect.top,
            right: monitor_rect.right,
            bottom: monitor_rect.bottom,
        },
        work: WorkArea {
            left: work.left,
            top: work.top,
            right: work.right,
            bottom: work.bottom,
        },
        dpi,
        primary: info.monitorInfo.dwFlags & 1 != 0,
    })
}
pub(crate) fn monitors() -> Result<Vec<DisplayMonitor>, String> {
    let mut output = Vec::new();
    // SAFETY: collect borrows this live output vector only until synchronous enumeration returns.
    unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM(&mut output as *mut _ as isize),
        )
    }
    .ok()
    .map_err(|e| e.to_string())?;
    output.into_iter().collect()
}
pub(crate) fn choices() -> Result<Vec<MonitorChoice>, String> {
    Ok(monitors()?
        .into_iter()
        .map(|monitor| MonitorChoice {
            id: monitor.id,
            label: monitor.label,
            connected: true,
        })
        .collect())
}

pub(crate) fn select_for_dismissal<'a>(
    monitors: &'a [DisplayMonitor],
    target: &str,
    pointer: Option<(i32, i32)>,
    shelf_device: Option<&str>,
    remembered: Option<&str>,
) -> Result<&'a DisplayMonitor, String> {
    let selected = (!target.is_empty())
        .then(|| monitors.iter().find(|monitor| monitor.id.eq_ignore_ascii_case(target)))
        .flatten()
        .or_else(|| {
            pointer.and_then(|(x, y)| {
                monitors.iter().find(|monitor| {
                    let area = monitor.monitor_rect;
                    x >= area.left && x < area.right && y >= area.top && y < area.bottom
                })
            })
        })
        .or_else(|| {
            shelf_device.and_then(|device| monitors.iter().find(|monitor| monitor.device == device))
        })
        .or_else(|| {
            remembered.and_then(|id| {
                monitors
                    .iter()
                    .find(|monitor| monitor.id.eq_ignore_ascii_case(id))
            })
        });
    selected
        .or_else(|| monitors.iter().find(|monitor| monitor.primary))
        .ok_or_else(|| "no selected or primary rail monitor is connected".into())
}
pub(crate) fn placement_for_dismissal(app: &AppHandle, settings: &Settings) -> Result<Placement, String> {
    let mut point = POINT::default();
    // SAFETY: GetCursorPos writes to a valid local POINT struct.
    let pointer = unsafe { GetCursorPos(&mut point) }
        .ok()
        .map(|()| (point.x, point.y));
    let monitors = monitors()?;
    let remembered = app
        .state::<crate::shelf::ShelfState>()
        .rail_monitor()
        .ok()
        .flatten();
    let shelf_monitor = app
        .get_webview_window("main")
        .and_then(|shelf| shelf.current_monitor().ok().flatten());
    let device = shelf_monitor.as_ref().and_then(|m| m.name()).map(String::as_str);
    Ok(Placement {
        monitor: select_for_dismissal(
            &monitors,
            &settings.rail_monitor_id,
            pointer,
            device,
            remembered.as_deref(),
        )?
        .clone(),
        edge: settings.rail_edge,
    })
}
pub(crate) fn reconcile(
    monitors: &[DisplayMonitor],
    settings: &Settings,
    current: Option<&Placement>,
    pointer: Option<(i32, i32)>,
    shelf_device: Option<&str>,
    remembered: Option<&str>,
) -> Result<Placement, String> {
    if let Some(current) = current {
        if !settings.rail_monitor_id.is_empty() {
            if let Some(target_mon) = monitors
                .iter()
                .find(|m| m.id.eq_ignore_ascii_case(&settings.rail_monitor_id))
            {
                return Ok(Placement {
                    monitor: target_mon.clone(),
                    edge: settings.rail_edge,
                });
            }
            if let Some(curr_mon) = monitors
                .iter()
                .find(|m| m.id.eq_ignore_ascii_case(&current.monitor.id))
            {
                return Ok(Placement {
                    monitor: curr_mon.clone(),
                    edge: settings.rail_edge,
                });
            }
            let fallback_mon = select_for_dismissal(
                monitors,
                "",
                pointer,
                shelf_device,
                remembered,
            )?;
            return Ok(Placement {
                monitor: fallback_mon.clone(),
                edge: settings.rail_edge,
            });
        }

        if let Some(curr_mon) = monitors
            .iter()
            .find(|m| m.id.eq_ignore_ascii_case(&current.monitor.id))
        {
            return Ok(Placement {
                monitor: curr_mon.clone(),
                edge: settings.rail_edge,
            });
        }

        let fallback_mon = select_for_dismissal(
            monitors,
            "",
            pointer,
            shelf_device,
            remembered,
        )?;
        return Ok(Placement {
            monitor: fallback_mon.clone(),
            edge: settings.rail_edge,
        });
    }

    if settings.rail_monitor_id.is_empty() {
        let mon = remembered
            .and_then(|id| monitors.iter().find(|m| m.id.eq_ignore_ascii_case(id)))
            .or_else(|| {
                select_for_dismissal(monitors, "", pointer, shelf_device, None).ok()
            })
            .ok_or_else(|| String::from("no selected or primary rail monitor is connected"))?;
        return Ok(Placement {
            monitor: mon.clone(),
            edge: settings.rail_edge,
        });
    }

    let mon = select_for_dismissal(
        monitors,
        &settings.rail_monitor_id,
        pointer,
        shelf_device,
        remembered,
    )?;
    Ok(Placement {
        monitor: mon.clone(),
        edge: settings.rail_edge,
    })
}
pub(crate) fn placement(app: &AppHandle, settings: &Settings) -> Result<Placement, String> {
    let mut point = POINT::default();
    let pointer = unsafe { GetCursorPos(&mut point) }
        .ok()
        .map(|()| (point.x, point.y));
    let monitors = monitors()?;
    let remembered = app
        .state::<crate::shelf::ShelfState>()
        .rail_monitor()
        .ok()
        .flatten();
    let shelf_monitor = app
        .get_webview_window("main")
        .and_then(|shelf| shelf.current_monitor().ok().flatten());
    let device = shelf_monitor.as_ref().and_then(|m| m.name()).map(String::as_str);
    let current = crate::resident::rail_placement(app)?;
    reconcile(
        &monitors,
        settings,
        current.as_ref(),
        pointer,
        device,
        remembered.as_deref(),
    )
}
pub(crate) fn bounds(
    placement: &Placement,
    logical_width: f64,
    logical_height: f64,
) -> Result<PhysicalPlacement, String> {
    let area = placement.monitor.work;
    if area.width() == 0 || area.height() == 0 {
        return Err("rail work area is empty".into());
    }
    let scale = f64::from(placement.monitor.dpi) / 96.0;
    let width = (logical_width * scale).round().max(1.0).min(f64::from(area.width())) as u32;
    let height = (logical_height * scale)
        .round()
        .max(1.0)
        .min(f64::from(area.height())) as u32;
    Ok(PhysicalPlacement {
        x: match placement.edge {
            RailEdge::Left => area.left,
            RailEdge::Right => area.right - width as i32,
        },
        y: area.top + (area.height() - height as i32) / 2,
        width,
        height,
    })
}

/// # Safety
/// This subclass is installed on a managed live rail HWND and forwards all other messages.
unsafe extern "system" fn no_activate(
    hwnd: HWND,
    message: u32,
    w: WPARAM,
    l: LPARAM,
    _: usize,
    subclass_data: usize,
) -> LRESULT {
    if message == WM_MOUSEACTIVATE {
        return LRESULT(MA_NOACTIVATE as isize);
    }
    if message == windows::Win32::UI::WindowsAndMessaging::WM_TIMER && w.0 == RAIL_TIMER_ID {
        // SAFETY: Live window HWND owns this timer identifier.
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd), RAIL_TIMER_ID) };
        // SAFETY: subclass_data points to a valid live AppHandle allocation created during initialize.
        // We clone the AppHandle before calling into resident functions to prevent use-after-free
        // in case a reentrant message (e.g. WM_NCDESTROY) deallocates subclass_data.
        let app = unsafe { (*(subclass_data as *const AppHandle)).clone() };
        if let Err(error) = crate::resident::tick_rail(&app) {
            let _ = crate::resident::publish_warning(&app, &error);
        }
        return LRESULT(0);
    }
    if message == windows::Win32::UI::WindowsAndMessaging::WM_NCDESTROY {
        // SAFETY: Clean up the subclass and reclaim the allocated AppHandle on window destruction.
        unsafe {
            let _ = windows::Win32::UI::Shell::RemoveWindowSubclass(hwnd, Some(no_activate), SUBCLASS);
            drop(Box::from_raw(subclass_data as *mut AppHandle));
            return DefSubclassProc(hwnd, message, w, l);
        }
    }
    // SAFETY: Forward the original native message to the remaining subclass chain.
    unsafe { DefSubclassProc(hwnd, message, w, l) }
}
pub(crate) fn initialize(app: &AppHandle) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("DropCove Edge Rail")
        .inner_size(16.0, 280.0)
        .visible(false)
        .focused(false)
        .decorations(false)
        .transparent(true)
        // Keep transparent CSS corners composited without a native backing bitmap.
        .no_redirection_bitmap(true)
        .shadow(false)
        .focusable(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    // SAFETY: Only our new hidden managed rail HWND is styled; FRAMECHANGED recalculates the border.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            style & !(WS_CAPTION.0 | WS_THICKFRAME.0) as isize,
        );
        let extended = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            extended | (WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0) as isize,
        );
        let app_handle_ptr = Box::into_raw(Box::new(app.clone()));
        if !SetWindowSubclass(hwnd, Some(no_activate), SUBCLASS, app_handle_ptr as usize).as_bool() {
            drop(Box::from_raw(app_handle_ptr));
            return Err("could not prevent rail mouse activation".into());
        }
    }
    crate::windows_drop::register(&window).map_err(|e| e.to_string())?;
    Ok(())
}
pub(crate) fn show(
    app: &AppHandle,
    placement: &Placement,
    width: f64,
    height: f64,
    expanded: bool,
) -> Result<(), String> {
    let window = app
        .get_webview_window(LABEL)
        .ok_or("Rail Handle window is missing")?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    let bounds = bounds(placement, width, height)?;
    // SAFETY: Exact physical bounds affect only the live rail HWND; NOACTIVATE preserves foreground.
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            bounds.x,
            bounds.y,
            bounds.width as i32,
            bounds.height as i32,
            SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
        )
    }
    .map_err(|e| e.to_string())?;
    window
        .emit(
            "rail-changed",
            Projection {
                edge: placement.edge,
                expanded,
                width,
                height,
            },
        )
        .map_err(|e| e.to_string())
}
pub(crate) fn hide(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(LABEL) {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        // SAFETY: The rail is shown through Win32, not Tao's cached visibility flags. Hide it
        // through the same native seam; a Tao hide can be skipped when its cached flag is false.
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd), RAIL_TIMER_ID);
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_HIDEWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            )
            .map_err(|e| e.to_string())?;
            if IsWindowVisible(hwnd).as_bool() {
                return Err("native rail hide did not hide the window".into());
            }
        }
    }
    Ok(())
}
pub(crate) fn cancel_rail_timer(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        if let Ok(hwnd) = window.hwnd() {
            // SAFETY: Live window HWND owns this timer identifier.
            let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd), RAIL_TIMER_ID) };
        }
    }
}
pub(crate) fn arm_rail_timer(app: &AppHandle, delay_ms: u32) -> Result<(), String> {
    let window = app
        .get_webview_window(LABEL)
        .ok_or_else(|| String::from("Rail window is missing"))?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    // SAFETY: Live window HWND owns this timer identifier. Kill any existing timer first to avoid stale accumulation.
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd), RAIL_TIMER_ID);
        let timer_res = windows::Win32::UI::WindowsAndMessaging::SetTimer(
            Some(hwnd),
            RAIL_TIMER_ID,
            delay_ms,
            None,
        );
        if timer_res == 0 {
            return Err(String::from("native SetTimer returned 0 (failed to arm)"));
        }
    }
    Ok(())
}
pub(crate) fn description(app: &AppHandle) -> Result<String, String> {
    let window = app
        .get_webview_window(LABEL)
        .ok_or("Rail Handle window is missing")?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    let mut rectangle = RECT::default();
    // SAFETY: This live window is owned by the app; rectangle is a valid writable native output.
    unsafe { GetWindowRect(hwnd, &mut rectangle) }.map_err(|e| e.to_string())?;
    Ok(format!(
        "bounds [{},{},{},{}], visible={}",
        rectangle.left,
        rectangle.top,
        rectangle.right,
        rectangle.bottom,
        window.is_visible().map_err(|e| e.to_string())?
    ))
}

#[inline]
pub(crate) fn rect_covers_monitor(window_rect: &WorkArea, monitor_rect: &WorkArea) -> bool {
    window_rect.left <= monitor_rect.left
        && window_rect.top <= monitor_rect.top
        && window_rect.right >= monitor_rect.right
        && window_rect.bottom >= monitor_rect.bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    fn monitor(id: &str, left: i32, dpi: u32) -> DisplayMonitor {
        DisplayMonitor {
            id: id.into(),
            label: id.into(),
            device: id.into(),
            monitor_rect: WorkArea {
                left,
                top: 0,
                right: left + 1920,
                bottom: 1080,
            },
            work: WorkArea {
                left,
                top: 0,
                right: left + 1920,
                bottom: 1032,
            },
            dpi,
            primary: left == 0,
        }
    }
    #[test]
    fn selected_and_remembered_display_precedence_preserves_unavailable_intent() {
        let screens = [monitor("primary", 0, 96), monitor("other", 1920, 144)];
        assert_eq!(
            select_for_dismissal(&screens, "other", None, None, Some("primary")).unwrap().id,
            "other"
        );
        // Disconnected explicit target falls back pointer-first: pointer on other -> other
        assert_eq!(
            select_for_dismissal(&screens, "missing", Some((2500, 500)), None, Some("primary")).unwrap().id,
            "other"
        );
        // Disconnected explicit target with no pointer falls back to remembered
        assert_eq!(
            select_for_dismissal(&screens, "missing", None, None, Some("other")).unwrap().id,
            "other"
        );
        assert_eq!(
            select_for_dismissal(&screens, "", None, None, Some("other")).unwrap().id,
            "other"
        );
        assert_eq!(
            select_for_dismissal(&screens, "", None, Some("other"), Some("missing"))
                .unwrap()
                .id,
            "other"
        );
        assert_eq!(
            select_for_dismissal(&screens, "", None, None, None).unwrap().id,
            "primary"
        );
    }
    #[test]
    fn fresh_dismissal_follows_pointer_before_remembered_monitor() {
        let screens = [monitor("primary", 0, 96), monitor("other", 1920, 144)];

        let selected = select_for_dismissal(
            &screens,
            "",
            Some((960, 500)),
            Some("other"),
            Some("other"),
        )
        .unwrap();

        assert_eq!(selected.id, "primary");
    }
    #[test]
    fn fresh_dismissal_precedence_and_fallback_ladder() {
        let screens = [monitor("primary", 0, 96), monitor("other", 1920, 144)];

        // 1. Connected explicit target wins over everything
        let s1 = select_for_dismissal(
            &screens,
            "other",
            Some((500, 500)),
            Some("primary"),
            Some("primary"),
        )
        .unwrap();
        assert_eq!(s1.id, "other");

        // 2. Disconnected explicit target falls back to pointer monitor without error
        let s2 = select_for_dismissal(
            &screens,
            "disconnected_mon",
            Some((2500, 500)),
            Some("primary"),
            Some("primary"),
        )
        .unwrap();
        assert_eq!(s2.id, "other");

        // 3. Pointer resolution failure falls back to shelf device
        let s3 = select_for_dismissal(
            &screens,
            "",
            Some((-500, -500)),
            Some("other"),
            Some("primary"),
        )
        .unwrap();
        assert_eq!(s3.id, "other");

        // 4. Shelf device failure falls back to remembered monitor
        let s4 = select_for_dismissal(
            &screens,
            "",
            None,
            Some("missing_shelf"),
            Some("other"),
        )
        .unwrap();
        assert_eq!(s4.id, "other");

        // 5. Remembered failure falls back to primary display
        let s5 = select_for_dismissal(
            &screens,
            "",
            None,
            Some("missing_shelf"),
            Some("missing_remembered"),
        )
        .unwrap();
        assert_eq!(s5.id, "primary");
    }
    #[test]
    fn docked_rail_reconciliation_preserves_placement_or_moves_predictably() {
        let screens = [monitor("primary", 0, 96), monitor("other", 1920, 144)];
        let docked_other = Placement {
            monitor: monitor("other", 1920, 144),
            edge: RailEdge::Right,
        };

        // 1. Follow pointer Save preserves currently docked monitor, applying new edge
        let mut settings = Settings::default();
        settings.rail_monitor_id = String::new();
        settings.rail_edge = RailEdge::Left;
        let r1 = reconcile(
            &screens,
            &settings,
            Some(&docked_other),
            Some((500, 500)), // pointer on primary
            Some("primary"),
            Some("primary"),
        )
        .unwrap();
        assert_eq!(r1.monitor.id, "other");
        assert_eq!(r1.edge, RailEdge::Left);

        // 2. Explicit Save moves immediately to configured monitor
        settings.rail_monitor_id = "primary".into();
        settings.rail_edge = RailEdge::Right;
        let r2 = reconcile(
            &screens,
            &settings,
            Some(&docked_other),
            Some((2500, 500)), // pointer on other
            Some("other"),
            Some("other"),
        )
        .unwrap();
        assert_eq!(r2.monitor.id, "primary");
        assert_eq!(r2.edge, RailEdge::Right);

        // 3. Explicit monitor disconnected while current docked monitor is still connected:
        // preserves current docked placement without jumping to pointer
        settings.rail_monitor_id = "disconnected".into();
        let r3 = reconcile(
            &screens,
            &settings,
            Some(&docked_other),
            Some((500, 500)), // pointer on primary
            None,
            None,
        )
        .unwrap();
        assert_eq!(r3.monitor.id, "other");

        // 3b. Explicit monitor disconnected AND current docked monitor is gone:
        // falls back to pointer monitor
        let single_screen = [monitor("primary", 0, 96)];
        let r3b = reconcile(
            &single_screen, // only "primary" is connected, "other" (docked) and "disconnected" are gone
            &settings,
            Some(&docked_other),
            Some((500, 500)), // pointer on primary
            None,
            None,
        )
        .unwrap();
        assert_eq!(r3b.monitor.id, "primary");

        // 4. Follow pointer hotplug: current monitor disappears -> moves to pointer monitor
        let single_screen = [monitor("primary", 0, 96)];
        settings.rail_monitor_id = String::new();
        let r4 = reconcile(
            &single_screen,
            &settings,
            Some(&docked_other), // "other" is now gone
            Some((500, 500)),
            None,
            None,
        )
        .unwrap();
        assert_eq!(r4.monitor.id, "primary");

        // 5. Non-docked startup/initial placement in automatic mode restores remembered connected monitor ahead of pointer
        let r5 = reconcile(
            &screens,
            &settings,
            None,
            Some((500, 500)), // pointer on primary
            None,
            Some("other"),    // remembered connected monitor is other
        )
        .unwrap();
        assert_eq!(r5.monitor.id, "other");

        // 6. Disconnected explicit target still falls back pointer-first
        settings.rail_monitor_id = "missing".into();
        let r6 = reconcile(
            &screens,
            &settings,
            None,
            Some((500, 500)), // pointer on primary
            None,
            Some("other"),    // remembered is other
        )
        .unwrap();
        assert_eq!(r6.monitor.id, "primary");
    }
    #[test]
    fn rail_geometry_is_centered_scaled_and_edge_aligned() {
        let mut placement = Placement {
            monitor: monitor("other", 1920, 144),
            edge: RailEdge::Right,
        };
        assert_eq!(
            bounds(&placement, 16.0, 280.0).unwrap(),
            PhysicalPlacement {
                x: 3816,
                y: 306,
                width: 24,
                height: 420,
            }
        );
        placement.edge = RailEdge::Left;
        assert_eq!(bounds(&placement, 16.0, 280.0).unwrap().x, 1920);
        assert_eq!(bounds(&placement, 16.0, 280.0).unwrap().width, 24);
    }
}
