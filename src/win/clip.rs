// Win32 clipboard, text only (CF_UNICODETEXT).

use super::*;
use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};

const CF_UNICODETEXT: u32 = 13;

fn open() -> Result<(), String> {
    for _ in 0..20 {
        if unsafe { OpenClipboard(None) }.is_ok() {
            return Ok(());
        }
        sleep_ms(25);
    }
    Err("Could not open the clipboard (another app is holding it)".into())
}

pub fn read() -> Result<String, String> {
    open()?;
    let r = unsafe {
        match GetClipboardData(CF_UNICODETEXT) {
            Ok(h) if !h.0.is_null() => {
                let g = HGLOBAL(h.0);
                let p = GlobalLock(g) as *const u16;
                if p.is_null() {
                    Ok(String::new())
                } else {
                    let max = GlobalSize(g) / 2;
                    let mut len = 0usize;
                    while len < max && *p.add(len) != 0 {
                        len += 1;
                    }
                    let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
                    let _ = GlobalUnlock(g);
                    Ok(s)
                }
            }
            _ => Ok(String::new()),
        }
    };
    unsafe {
        let _ = CloseClipboard();
    }
    r
}

pub fn write(text: &str) -> Result<(), String> {
    let w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    open()?;
    let r = unsafe {
        let _ = EmptyClipboard();
        match GlobalAlloc(GMEM_MOVEABLE, w.len() * 2) {
            Ok(g) => {
                let p = GlobalLock(g) as *mut u16;
                if p.is_null() {
                    Err("Failed to write to clipboard".to_string())
                } else {
                    std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
                    let _ = GlobalUnlock(g);
                    SetClipboardData(CF_UNICODETEXT, Some(HANDLE(g.0))).map(|_| ()).map_err(|_| "Failed to write to clipboard".to_string())
                }
            }
            Err(_) => Err("Failed to write to clipboard".to_string()),
        }
    };
    unsafe {
        let _ = CloseClipboard();
    }
    r
}
