// SPDX-License-Identifier: Apache-2.0
#![cfg(target_os = "windows")]

//! Custom non-client handling for borderless-but-resizable windows on
//! Windows (`AppConfig::decorations = false`). winit only strips the
//! title bar; it doesn't restore edge-resize hit-testing once the
//! non-client area is gone, so this subclasses the HWND winit created and
//! answers WM_NCCALCSIZE/WM_NCHITTEST ourselves.

use std::sync::Arc;
use std::sync::atomic::{ AtomicIsize, Ordering };
use raw_window_handle::{ HasWindowHandle, RawWindowHandle };
use winit::window::Window;
use windows_sys::Win32::Foundation::{ HWND, LPARAM, LRESULT, POINT, RECT, WPARAM };
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW,
    GetClientRect,
    SetWindowLongPtrW,
    GWLP_WNDPROC,
    HTBOTTOM,
    HTBOTTOMLEFT,
    HTBOTTOMRIGHT,
    HTLEFT,
    HTRIGHT,
    HTTOP,
    HTTOPLEFT,
    HTTOPRIGHT,
    WM_NCCALCSIZE,
    WM_NCHITTEST,
};

use windows_sys::Win32::Graphics::Gdi::ScreenToClient;

static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);
const RESIZE_BORDER: i32 = 8;

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {
    match msg {
        // wParam != 0 asks how much of the window rect is client area;
        // returning 0 without touching the rect claims the whole window,
        // which is what erases the default title bar.
        WM_NCCALCSIZE if wparam != 0 => {
            return 0;
        }
        WM_NCHITTEST => {
            let original = ORIGINAL_WNDPROC.load(Ordering::Relaxed);
            let original_proc = unsafe {
                std::mem::transmute::<
                    isize,
                    unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT
                >(original)
            };

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

            return match (left, right, top, bottom) {
                (true, _, true, _) => HTTOPLEFT as LRESULT,
                (_, true, true, _) => HTTOPRIGHT as LRESULT,
                (true, _, _, true) => HTBOTTOMLEFT as LRESULT,
                (_, true, _, true) => HTBOTTOMRIGHT as LRESULT,
                (true, _, _, _) => HTLEFT as LRESULT,
                (_, true, _, _) => HTRIGHT as LRESULT,
                (_, _, true, _) => HTTOP as LRESULT,
                (_, _, _, true) => HTBOTTOM as LRESULT,
                _ => unsafe { CallWindowProcW(Some(original_proc), hwnd, msg, wparam, lparam) }
            };
        }
        _ => {}
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

/// Subclasses `window`'s HWND so it keeps OS-driven edge resizing while
/// losing the default title bar. Call once, right after window creation,
/// only when `AppConfig::decorations` is false.
pub fn install_for_window(window: &Arc<Window>) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };

    unsafe {
        let hwnd = handle.hwnd.get() as HWND;
        let previous = SetWindowLongPtrW(
            hwnd,
            GWLP_WNDPROC,
            subclass_proc as *const () as usize as isize
        );
        ORIGINAL_WNDPROC.store(previous, Ordering::Relaxed);
    }
}
