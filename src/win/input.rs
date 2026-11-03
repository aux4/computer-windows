// Real input (SendInput) and the foreground fallback: save the cursor + foreground window, bring
// the target forward, act, then put both back.

use super::*;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

fn send(inputs: &[INPUT]) -> Result<(), String> {
    let n = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if n as usize != inputs.len() {
        return Err("Windows rejected the input (SendInput): the target may run elevated (UIPI), or the desktop is locked or not interactive".into());
    }
    Ok(())
}

fn key_input(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VIRTUAL_KEY(vk), wScan: scan, dwFlags: flags, time: 0, dwExtraInfo: 0 } },
    }
}

fn mouse_input(flags: MOUSE_EVENT_FLAGS, data: i32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 { mi: MOUSEINPUT { dx: 0, dy: 0, mouseData: data as u32, dwFlags: flags, time: 0, dwExtraInfo: 0 } },
    }
}

pub fn cursor_physical() -> POINT {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    p
}

pub fn set_cursor_physical(x: i32, y: i32) -> Result<(), String> {
    unsafe { SetCursorPos(x, y) }.map_err(|e| format!("Could not move the cursor: {}", e.message()))
}

pub fn click_physical(x: i32, y: i32, right: bool, count: u32) -> Result<(), String> {
    set_cursor_physical(x, y)?;
    sleep_ms(30);
    let (down, up) = if right { (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP) } else { (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP) };
    for i in 0..count.max(1) {
        send(&[mouse_input(down, 0), mouse_input(up, 0)])?;
        if i + 1 < count {
            sleep_ms(40);
        }
    }
    Ok(())
}

pub fn drag_physical(x: i32, y: i32, tx: i32, ty: i32) -> Result<(), String> {
    set_cursor_physical(x, y)?;
    sleep_ms(30);
    send(&[mouse_input(MOUSEEVENTF_LEFTDOWN, 0)])?;
    let steps = 12;
    for i in 1..=steps {
        let nx = x + (tx - x) * i / steps;
        let ny = y + (ty - y) * i / steps;
        set_cursor_physical(nx, ny)?;
        sleep_ms(12);
    }
    send(&[mouse_input(MOUSEEVENTF_LEFTUP, 0)])
}

pub fn wheel(dx: f64, dy: f64) -> Result<(), String> {
    // positive dy scrolls down (content moves up), matching computer-mac; 120 = one notch ≈ 40px
    let mut v = Vec::new();
    if dy != 0.0 {
        v.push(mouse_input(MOUSEEVENTF_WHEEL, (-dy * 3.0).round() as i32));
    }
    if dx != 0.0 {
        v.push(mouse_input(MOUSEEVENTF_HWHEEL, (dx * 3.0).round() as i32));
    }
    if v.is_empty() {
        return Ok(());
    }
    send(&v)
}

pub fn type_unicode(text: &str) -> Result<(), String> {
    for unit in text.encode_utf16() {
        if unit == '\n' as u16 {
            send(&[key_input(VK_RETURN.0, 0, KEYBD_EVENT_FLAGS(0)), key_input(VK_RETURN.0, 0, KEYEVENTF_KEYUP)])?;
        } else if unit == '\r' as u16 {
            continue;
        } else if unit == '\t' as u16 {
            send(&[key_input(VK_TAB.0, 0, KEYBD_EVENT_FLAGS(0)), key_input(VK_TAB.0, 0, KEYEVENTF_KEYUP)])?;
        } else {
            send(&[key_input(0, unit, KEYEVENTF_UNICODE), key_input(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP)])?;
        }
        sleep_ms(2);
    }
    Ok(())
}

/// Virtual key for a key name from parse_combo.
pub fn vk_for(key: &str) -> Option<(u16, bool)> {
    // (vk, extended)
    let k = key;
    let named = match k {
        "enter" | "return" => Some((VK_RETURN.0, false)),
        "tab" => Some((VK_TAB.0, false)),
        "space" => Some((VK_SPACE.0, false)),
        "escape" | "esc" => Some((VK_ESCAPE.0, false)),
        "backspace" => Some((VK_BACK.0, false)),
        "delete" | "del" | "forwarddelete" => Some((VK_DELETE.0, true)),
        "insert" => Some((VK_INSERT.0, true)),
        "home" => Some((VK_HOME.0, true)),
        "end" => Some((VK_END.0, true)),
        "pageup" => Some((VK_PRIOR.0, true)),
        "pagedown" => Some((VK_NEXT.0, true)),
        "up" => Some((VK_UP.0, true)),
        "down" => Some((VK_DOWN.0, true)),
        "left" => Some((VK_LEFT.0, true)),
        "right" => Some((VK_RIGHT.0, true)),
        "capslock" => Some((VK_CAPITAL.0, false)),
        "printscreen" => Some((VK_SNAPSHOT.0, true)),
        "menu" | "apps" => Some((VK_APPS.0, true)),
        "plus" => Some((VK_OEM_PLUS.0, false)),
        "minus" => Some((VK_OEM_MINUS.0, false)),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    if let Some(n) = k.strip_prefix('f').and_then(|n| n.parse::<u16>().ok()) {
        if (1..=24).contains(&n) {
            return Some((VK_F1.0 + n - 1, false));
        }
    }
    let mut chars = k.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        if c.is_ascii_alphanumeric() {
            return Some((c.to_ascii_uppercase() as u16, false));
        }
        let r = unsafe { VkKeyScanW(c as u16) };
        if r != -1 {
            return Some(((r as u16) & 0xff, false));
        }
    }
    None
}

pub fn press_combo(c: &Combo) -> Result<(), String> {
    let (vk, ext) = vk_for(&c.key).ok_or_else(|| format!("Unknown key: {}", c.key))?;
    let mut mods: Vec<u16> = Vec::new();
    if c.ctrl {
        mods.push(VK_CONTROL.0);
    }
    if c.alt {
        mods.push(VK_MENU.0);
    }
    if c.shift {
        mods.push(VK_SHIFT.0);
    }
    if c.win {
        mods.push(VK_LWIN.0);
    }
    let mut v = Vec::new();
    for m in &mods {
        v.push(key_input(*m, 0, KEYBD_EVENT_FLAGS(0)));
    }
    let ef = if ext { KEYEVENTF_EXTENDEDKEY } else { KEYBD_EVENT_FLAGS(0) };
    v.push(key_input(vk, 0, ef));
    v.push(key_input(vk, 0, ef | KEYEVENTF_KEYUP));
    for m in mods.iter().rev() {
        v.push(key_input(*m, 0, KEYEVENTF_KEYUP));
    }
    send(&v)
}

// ---------------------------------------------------------------------------------------------
// foreground handling

pub fn foreground_window() -> HWND {
    unsafe { GetForegroundWindow() }
}

/// SetForegroundWindow with the usual workarounds for the foreground lock.
pub fn bring_to_front(h: HWND) -> bool {
    unsafe {
        if IsIconic(h).as_bool() {
            let _ = ShowWindow(h, SW_RESTORE);
        }
        if GetForegroundWindow() == h {
            return true;
        }
        let _ = SetForegroundWindow(h);
        if wait_foreground(h, 300) {
            return true;
        }
        // Being the source of the last input event grants the right to set the foreground.
        let _ = send(&[mouse_input(MOUSEEVENTF_MOVE, 0)]);
        let _ = SetForegroundWindow(h);
        if wait_foreground(h, 300) {
            return true;
        }
        let fg = GetForegroundWindow();
        let fg_thread = GetWindowThreadProcessId(fg, None);
        let me = GetCurrentThreadId();
        let attached = fg_thread != 0 && fg_thread != me && AttachThreadInput(me, fg_thread, true).as_bool();
        let _ = BringWindowToTop(h);
        let _ = SetForegroundWindow(h);
        if attached {
            let _ = AttachThreadInput(me, fg_thread, false);
        }
        wait_foreground(h, 500)
    }
}

fn wait_foreground(h: HWND, ms: u64) -> bool {
    let start = std::time::Instant::now();
    loop {
        let fg = unsafe { GetForegroundWindow() };
        if fg == h || unsafe { GetAncestor(fg, GA_ROOTOWNER) } == h {
            return true;
        }
        if start.elapsed().as_millis() as u64 >= ms {
            return false;
        }
        sleep_ms(20);
    }
}

struct Saved {
    cursor: POINT,
    fg: HWND,
}

/// The foreground path for an action on `target` (None: act on whatever is in front).
pub struct Fg {
    pub target: Option<HWND>,
    pub name: String,
}

impl Foreground for Fg {
    fn save(&self) -> Option<Box<dyn std::any::Any>> {
        Some(Box::new(Saved { cursor: cursor_physical(), fg: foreground_window() }))
    }
    fn bring_forward(&self) -> Result<(), String> {
        if let Some(h) = self.target {
            if !bring_to_front(h) {
                return Err(format!("Could not bring '{}' to the front for real input", self.name));
            }
            sleep_ms(120);
        }
        Ok(())
    }
    fn restore(&self, saved: Box<dyn std::any::Any>) -> Value {
        let s = match saved.downcast::<Saved>() {
            Ok(s) => s,
            Err(_) => return json!({"cursor": false, "frontmost": false}),
        };
        let now = cursor_physical();
        if now.x != s.cursor.x || now.y != s.cursor.y {
            let _ = set_cursor_physical(s.cursor.x, s.cursor.y);
        }
        let mut front = true;
        if !s.fg.0.is_null() && unsafe { IsWindow(Some(s.fg)).as_bool() } && foreground_window() != s.fg {
            front = bring_to_front(s.fg);
        }
        let c = cursor_physical();
        json!({"cursor": (c.x - s.cursor.x).abs() <= 1 && (c.y - s.cursor.y).abs() <= 1, "frontmost": front})
    }
}

/// Restore the foreground window if something we did (open, maximize) stole it.
pub fn restore_foreground_if_changed(prev: HWND) -> Option<bool> {
    if prev.0.is_null() || foreground_window() == prev || !unsafe { IsWindow(Some(prev)).as_bool() } {
        return None;
    }
    Some(bring_to_front(prev))
}
