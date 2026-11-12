//! Initial monitor selection and complete-frame placement for Settings and About.

use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Clone, Copy)]
pub(crate) enum InvocationContext {
    DropShelf,
    NotificationArea,
}

pub(crate) fn target_monitor(
    app: &AppHandle,
    context: InvocationContext,
) -> Result<tauri::Monitor, String> {
    match context {
        InvocationContext::DropShelf => {
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.current_monitor() {
                    return Ok(monitor);
                }
            }
        }
        InvocationContext::NotificationArea => {
            if let Ok(cursor) = app.cursor_position() {
                if let Ok(monitors) = app.available_monitors() {
                    for monitor in monitors {
                        let position = monitor.position();
                        let size = monitor.size();
                        if cursor.x >= f64::from(position.x)
                            && cursor.x < f64::from(position.x) + f64::from(size.width)
                            && cursor.y >= f64::from(position.y)
                            && cursor.y < f64::from(position.y) + f64::from(size.height)
                        {
                            return Ok(monitor);
                        }
                    }
                }
            }
        }
    }
    app.primary_monitor()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "no monitor available".to_owned())
}

pub(crate) fn centered_outer_position(
    work_left: i32,
    work_top: i32,
    work_width: i32,
    work_height: i32,
    outer_width: i32,
    outer_height: i32,
) -> (i32, i32) {
    (
        work_left + (work_width - outer_width).max(0) / 2,
        work_top + (work_height - outer_height).max(0) / 2,
    )
}

pub(crate) fn center_in_work_area(
    window: &WebviewWindow,
    monitor: &tauri::Monitor,
) -> Result<(), String> {
    use windows::Win32::{Foundation::RECT, UI::WindowsAndMessaging::GetWindowRect};

    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let mut outer = RECT::default();
    // SAFETY: The live window owns HWND; outer is writable for the duration of the call.
    unsafe { GetWindowRect(hwnd, &mut outer) }.map_err(|error| error.to_string())?;
    let work = monitor.work_area();
    let (x, y) = centered_outer_position(
        work.position.x,
        work.position.y,
        work.size.width as i32,
        work.size.height as i32,
        outer.right - outer.left,
        outer.bottom - outer.top,
    );
    window
        .set_position(tauri::PhysicalPosition::new(x, y))
        .map_err(|error| format!("failed to position {} window: {error}", window.label()))
}

#[cfg(test)]
mod tests {
    use super::centered_outer_position;

    #[test]
    fn complete_frames_center_in_offset_work_areas() {
        assert_eq!(centered_outer_position(0, 0, 1920, 1040, 416, 499), (752, 270));
        assert_eq!(centered_outer_position(-1920, 100, 1920, 1080, 436, 559), (-1178, 360));
    }

    #[test]
    fn oversized_frames_anchor_to_the_work_area_origin() {
        assert_eq!(centered_outer_position(100, 50, 300, 350, 436, 559), (100, 50));
    }
}
