// SPDX-License-Identifier: Apache-2.0
#![cfg(target_os = "windows")]

//! Chromium-style custom chrome for Windows: keeps the native window frame
//! (WS_THICKFRAME) so resizing, Snap Layouts, and the DWM shadow/rounded
//! corners all keep working, and only strips the title bar (WS_CAPTION).
//! Subclassing is used solely to answer WM_NCHITTEST for the resize
//! border; WM_NCCALCSIZE is left untouched so the OS keeps computing the
//! non-client area normally.

use std::sync::Arc;
use std::sync::atomic::{ AtomicIsize, Ordering };
use raw_window_handle::{ HasWindowHandle, RawWindowHandle };
use winit::window::Window;
use windows_sys::Win32::Foundation::{ HWND, LPARAM, LRESULT, POINT, RECT, WPARAM };
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW,
    GetClientRect,
    GetWindowLongPtrW,
    SetWindowLongPtrW,
    SetWindowPos,
    GWL_STYLE,
    GWLP_WNDPROC,
    HTBOTTOM,
    HTBOTTOMLEFT,
    HTBOTTOMRIGHT,
    HTCLIENT,
    HTLEFT,
    HTRIGHT,
    HTTOP,
    HTTOPLEFT,
    HTTOPRIGHT,
    SWP_FRAMECHANGED,
    SWP_NOACTIVATE,
    SWP_NOMOVE,
    SWP_NOSIZE,
    SWP_NOZORDER,
    WM_NCHITTEST,
    WS_CAPTION,
};
use windows_sys::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea,
    DwmSetWindowAttribute,
    DWMWA_USE_IMMERSIVE_DARK_MODE,
    DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_ROUND,
};
use windows_sys::Win32::UI::Controls::MARGINS;
use windows_sys::Win32::Graphics::Gdi::ScreenToClient;

static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);
const RESIZE_BORDER: i32 = 8;

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {
    if msg == WM_NCHITTEST {
        let x = (lparam & 0xffff) as i16 as i32;
        let y = ((lparam >> 16) & 0xffff) as i16 as i32;
        let mut pt = POINT { x, y };
        unsafe {
            ScreenToClient(hwnd, &mut pt);
        }

        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        unsafe {
            GetClientRect(hwnd, &mut rect);
        }

        let left = pt.x <= RESIZE_BORDER;
        let right = pt.x >= rect.right - RESIZE_BORDER;
        let top = pt.y <= RESIZE_BORDER;
        let bottom = pt.y >= rect.bottom - RESIZE_BORDER;

        let hit = match (left, right, top, bottom) {
            (true, _, true, _) => Some(HTTOPLEFT),
            (_, true, true, _) => Some(HTTOPRIGHT),
            (true, _, _, true) => Some(HTBOTTOMLEFT),
            (_, true, _, true) => Some(HTBOTTOMRIGHT),
            (true, _, _, _) => Some(HTLEFT),
            (_, true, _, _) => Some(HTRIGHT),
            (_, _, true, _) => Some(HTTOP),
            (_, _, _, true) => Some(HTBOTTOM),
            _ => None,
        };

        if let Some(hit) = hit {
            return hit as LRESULT;
        }

        // Everywhere else is ordinary client area; the app's own custom
        // titlebar widget drags the window via `drag_window()` instead of
        // this hit-test claiming HTCAPTION.
        return HTCLIENT as LRESULT;
    }

    let original = ORIGINAL_WNDPROC.load(Ordering::Relaxed);
    let original_proc = unsafe {
        std::mem::transmute::<
            isize,
            unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT
        >(original)
    };

    unsafe { CallWindowProcW(Some(original_proc), hwnd, msg, wparam, lparam) }
}

/// Strips only WS_CAPTION from `window`'s HWND, leaving WS_THICKFRAME /
/// WS_SYSMENU / WS_MINIMIZEBOX / WS_MAXIMIZEBOX intact, then re-enables
/// the DWM shadow, rounded corners, and dark title-bar tint. Call once,
/// right after window creation, only when `AppConfig::decorations` is
/// false. The window itself must still be created with real OS
/// decorations (`with_decorations(true)`) for this to have anything to
/// subclass correctly.
pub fn install_for_window(window: &Arc<Window>) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };

    unsafe {
        let hwnd = handle.hwnd.get() as HWND;

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let new_style = style & !(WS_CAPTION as isize);
        SetWindowLongPtrW(hwnd, GWL_STYLE, new_style);

        // Needed for a GWL_STYLE change to actually take visual effect.
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
        );

        // Keeps the OS's native drop shadow around a window that no
        // longer has a caption.
        let margins = MARGINS {
            cxLeftWidth: 1,
            cxRightWidth: 1,
            cyTopHeight: 1,
            cyBottomHeight: 1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

        let corner_pref: u32 = DWMWCP_ROUND as u32;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            &corner_pref as *const _ as *const _,
            std::mem::size_of_val(&corner_pref) as u32
        );

        let dark: i32 = 1;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            &dark as *const _ as *const _,
            std::mem::size_of_val(&dark) as u32
        );

        let previous = SetWindowLongPtrW(
            hwnd,
            GWLP_WNDPROC,
            subclass_proc as *const () as usize as isize
        );
        ORIGINAL_WNDPROC.store(previous, Ordering::Relaxed);
    }
}
