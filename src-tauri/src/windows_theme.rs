//! All HWND frames follow the Windows application theme, including live setting changes.
use windows::{core::w, Win32::{Foundation::{HWND, LPARAM, LRESULT, WPARAM, ERROR_SUCCESS}, Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE}, System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD}, UI::{Shell::{SetWindowSubclass, RemoveWindowSubclass, DefSubclassProc}, WindowsAndMessaging::{WM_SETTINGCHANGE, WM_THEMECHANGED, WM_NCDESTROY}}}};
const SUBCLASS:usize=0xDC07;
fn sync(hwnd:HWND) {
    let mut light=1u32;let mut size=4u32;
    // SAFETY: Query writes one DWORD into a valid stack buffer.
    let result=unsafe{RegGetValueW(HKEY_CURRENT_USER,w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),w!("AppsUseLightTheme"),RRF_RT_REG_DWORD,None,Some((&mut light as *mut u32).cast()),Some(&mut size))};
    let dark=i32::from(result==ERROR_SUCCESS && light==0);
    // SAFETY: Caller owns a live window; the attribute expects a four-byte BOOL.
    if let Err(error)=unsafe{DwmSetWindowAttribute(hwnd,DWMWA_USE_IMMERSIVE_DARK_MODE,(&dark as *const i32).cast(),4)}{eprintln!("native theme sync failed: {error}");}
}
pub(crate) fn register(hwnd:HWND)->Result<(),String>{
    sync(hwnd);
    // SAFETY: Registration runs on HWND's owning UI thread, with no borrowed context.
    if unsafe{SetWindowSubclass(hwnd,Some(message),SUBCLASS,0)}.as_bool(){Ok(())}else{Err("could not observe native theme changes".into())}
}
unsafe extern "system" fn message(hwnd:HWND,code:u32,wparam:WPARAM,lparam:LPARAM,id:usize,_:usize)->LRESULT{
    if code==WM_SETTINGCHANGE || code==WM_THEMECHANGED{sync(hwnd);}
    if code==WM_NCDESTROY{
        // SAFETY: Remove this observer on its owner thread before forwarding final destruction.
        unsafe{let _=RemoveWindowSubclass(hwnd,Some(message),id);}
    }
    // SAFETY: Preserve all native message arguments for the original subclass chain.
    unsafe{DefSubclassProc(hwnd,code,wparam,lparam)}
}
