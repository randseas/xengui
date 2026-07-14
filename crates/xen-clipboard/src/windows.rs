use std::{ ptr, slice };

use windows_sys::Win32::{
    Foundation::HWND,
    System::{
        DataExchange::{
            CloseClipboard,
            EmptyClipboard,
            GetClipboardData,
            IsClipboardFormatAvailable,
            OpenClipboard,
            SetClipboardData,
        },
        Memory::{ GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE },
    },
};

const CF_UNICODETEXT: u32 = 13;

use crate::{ ClipboardError };

use super::ClipboardBackend;

pub struct WindowsClipboard;

impl WindowsClipboard {
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

impl ClipboardBackend for WindowsClipboard {
    fn get_text(&self, callback: Box<dyn FnOnce(Result<Option<String>, ClipboardError>) + Send>) {
        let result = unsafe {
            if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 {
                Err(ClipboardError::FormatUnavailable)
            } else if OpenClipboard(0 as HWND) == 0 {
                Err(ClipboardError::OpenFailed)
            } else {
                let handle = GetClipboardData(CF_UNICODETEXT);

                if handle.is_null() {
                    CloseClipboard();
                    Err(ClipboardError::ReadFailed)
                } else {
                    let ptr = GlobalLock(handle) as *const u16;

                    if ptr.is_null() {
                        CloseClipboard();
                        Err(ClipboardError::LockFailed)
                    } else {
                        let mut len = 0;

                        while *ptr.add(len) != 0 {
                            len += 1;
                        }

                        let slice = slice::from_raw_parts(ptr, len);

                        let text = String::from_utf16(slice)
                            .map(Some)
                            .map_err(|_| ClipboardError::ReadFailed);

                        GlobalUnlock(handle);
                        CloseClipboard();

                        text
                    }
                }
            }
        };

        callback(result);
    }

    fn set_text(&self, text: &str) -> Result<(), ClipboardError> {
        unsafe {
            if OpenClipboard(0 as HWND) == 0 {
                return Err(ClipboardError::OpenFailed);
            }

            EmptyClipboard();

            let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

            let size = utf16.len() * std::mem::size_of::<u16>();

            let handle = GlobalAlloc(GMEM_MOVEABLE, size);

            if handle.is_null() {
                CloseClipboard();
                return Err(ClipboardError::ReadFailed);
            }

            let ptr = GlobalLock(handle) as *mut u16;

            if ptr.is_null() {
                CloseClipboard();
                return Err(ClipboardError::ReadFailed);
            }

            ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());

            GlobalUnlock(handle);

            if SetClipboardData(CF_UNICODETEXT, handle).is_null() {
                CloseClipboard();
                return Err(ClipboardError::ReadFailed);
            }

            CloseClipboard();

            Ok(())
        }
    }

    fn has_text(&self, callback: Box<dyn FnOnce(Result<bool, ClipboardError>) + Send>) {
        callback(Ok(unsafe { IsClipboardFormatAvailable(CF_UNICODETEXT) != 0 }));
    }
}
