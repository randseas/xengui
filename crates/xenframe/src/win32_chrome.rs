// SPDX-License-Identifier: Apache-2.0
#![cfg(target_os = "windows")]

use std::sync::Arc;
use raw_window_handle::{ HasWindowHandle, RawWindowHandle };
use winit::window::Window;
use windows_sys::Win32::Foundation::{ HWND, LPARAM, LRESULT, RECT, WPARAM };
use windows_sys::Win32::UI::Shell::{ DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass };
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowRect,
    IsZoomed,
    SetWindowPos,
    HTBOTTOM,
    HTBOTTOMLEFT,
    HTBOTTOMRIGHT,
    HTCLIENT,
    HTLEFT,
    HTRIGHT,
    HTTOP,
    HTTOPLEFT,
    HTTOPRIGHT,
    NCCALCSIZE_PARAMS,
    SWP_FRAMECHANGED,
    SWP_NOACTIVATE,
    SWP_NOMOVE,
    SWP_NOSIZE,
    SWP_NOZORDER,
    WM_DESTROY,
    WM_NCCALCSIZE,
    WM_NCHITTEST,
};
use windows_sys::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea,
    DwmSetWindowAttribute,
    DWMWA_USE_IMMERSIVE_DARK_MODE,
    DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_ROUND,
};
use windows_sys::Win32::UI::Controls::MARGINS;

const SUBCLASS_ID: usize = 1;

unsafe extern "system" fn custom_chrome_subclass(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _dwrefdata: usize
) -> LRESULT {
    match msg {
        WM_NCCALCSIZE if wparam != 0 => {
            let params = unsafe { &mut *(lparam as *mut NCCALCSIZE_PARAMS) };

            // Adjust client margins when window is maximized to prevent overflow
            if (unsafe { IsZoomed(hwnd) }) != 0 {
                let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                unsafe {
                    GetWindowRect(hwnd, &mut rect);
                }
                params.rgrc[0] = rect;
            }

            // Return 0 to remove the native OS titlebar area entirely
            return 0;
        }
        WM_NCHITTEST => {
            // Call default proc first to let OS evaluate base hit areas
            let hit = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
            if hit != (HTCLIENT as LRESULT) {
                return hit;
            }

            let x = (lparam & 0xffff) as i16 as i32;
            let y = ((lparam >> 16) & 0xffff) as i16 as i32;

            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            unsafe {
                GetWindowRect(hwnd, &mut rect);
            }

            let border_width = 8;
            let left = x < rect.left + border_width;
            let right = x >= rect.right - border_width;
            let top = y < rect.top + border_width;
            let bottom = y >= rect.bottom - border_width;

            let custom_hit = match (left, right, top, bottom) {
                (true, _, true, _) => HTTOPLEFT,
                (_, true, true, _) => HTTOPRIGHT,
                (true, _, _, true) => HTBOTTOMLEFT,
                (_, true, _, true) => HTBOTTOMRIGHT,
                (true, _, _, _) => HTLEFT,
                (_, true, _, _) => HTRIGHT,
                (_, _, true, _) => HTTOP,
                (_, _, _, true) => HTBOTTOM,
                _ => HTCLIENT,
            };

            if custom_hit != HTCLIENT {
                return custom_hit as LRESULT;
            }
        }
        WM_DESTROY => {
            // Remove subclass hook when window is destroyed
            unsafe {
                RemoveWindowSubclass(hwnd, Some(custom_chrome_subclass), SUBCLASS_ID);
            }
        }
        _ => {}
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub fn install_for_window(window: &Arc<Window>) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };

    unsafe {
        let hwnd = handle.hwnd.get() as HWND;

        // Force frame recalculation without stripping WS_CAPTION
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
        );

        // Extend DWM frame for native shadows
        let margins = MARGINS {
            cxLeftWidth: 1,
            cxRightWidth: 1,
            cyTopHeight: 1,
            cyBottomHeight: 1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

        let corner_pref = DWMWCP_ROUND as u32;
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

        // Attach subclassing via comctl32 safely
        SetWindowSubclass(hwnd, Some(custom_chrome_subclass), SUBCLASS_ID, 0);
    }
}
