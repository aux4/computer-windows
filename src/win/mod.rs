// Windows platform layer: process/window enumeration, app resolution, DPI-aware logical
// coordinates, integrity levels (UIPI) and the command dispatcher.

pub mod act;
pub mod appwin;
pub mod capture;
pub mod clip;
pub mod cmds;
pub mod input;
pub mod ocr;
pub mod uia;

use crate::common::*;
use serde_json::{json, Value};
use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, HDC, HMONITOR, MONITORINFO,
    MONITORINFOEXW, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TokenIntegrityLevel, TokenUIAccess, TOKEN_ELEVATION, TOKEN_MANDATORY_LABEL, TOKEN_QUERY};
use windows::Win32::System::StationsAndDesktops::{CloseDesktop, OpenInputDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::*;

pub use cmds::exec;

// ---------------------------------------------------------------------------------------------
// init

pub fn init() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let _ = windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_MULTITHREADED);
    }
}

pub fn wstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

pub fn hwnd_id(h: HWND) -> i64 {
    h.0 as isize as i64
}

pub fn hwnd_from_id(id: i64) -> HWND {
    HWND(id as isize as *mut core::ffi::c_void)
}

// ---------------------------------------------------------------------------------------------
// desktop + permissions

pub fn interactive_desktop() -> bool {
    if permissions_denied() {
        return false;
    }
    unsafe {
        match OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_READOBJECTS) {
            Ok(d) => {
                let _ = CloseDesktop(d);
                true
            }
            Err(_) => false,
        }
    }
}

pub fn require_desktop() {
    if !interactive_desktop() {
        crate::desktop_denied();
    }
}

fn token_info_u32(token: HANDLE, class: windows::Win32::Security::TOKEN_INFORMATION_CLASS) -> Option<u32> {
    unsafe {
        let mut v: u32 = 0;
        let mut len = 0u32;
        GetTokenInformation(token, class, Some(&mut v as *mut u32 as *mut _), 4, &mut len).ok()?;
        Some(v)
    }
}

/// Integrity RID of a process token: 0x1000 low, 0x2000 medium, 0x3000 high, 0x4000 system.
fn token_integrity(token: HANDLE) -> Option<u32> {
    unsafe {
        let mut len = 0u32;
        let _ = GetTokenInformation(token, TokenIntegrityLevel, None, 0, &mut len);
        if len == 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        GetTokenInformation(token, TokenIntegrityLevel, Some(buf.as_mut_ptr() as *mut _), len, &mut len).ok()?;
        let label = &*(buf.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let sid = label.Label.Sid;
        let count = *windows::Win32::Security::GetSidSubAuthorityCount(sid);
        if count == 0 {
            return None;
        }
        Some(*windows::Win32::Security::GetSidSubAuthority(sid, (count - 1) as u32))
    }
}

pub fn integrity_name(rid: u32) -> &'static str {
    match rid {
        r if r >= 0x4000 => "system",
        r if r >= 0x3000 => "high",
        r if r >= 0x2000 => "medium",
        r if r >= 0x1000 => "low",
        _ => "untrusted",
    }
}

pub struct SelfToken {
    pub elevated: bool,
    pub ui_access: bool,
    pub integrity: u32,
}

pub fn self_token() -> SelfToken {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return SelfToken { elevated: false, ui_access: false, integrity: 0x2000 };
        }
        let mut el = TOKEN_ELEVATION::default();
        let mut len = 0u32;
        let elevated = GetTokenInformation(token, TokenElevation, Some(&mut el as *mut _ as *mut _), std::mem::size_of::<TOKEN_ELEVATION>() as u32, &mut len).is_ok() && el.TokenIsElevated != 0;
        let ui_access = token_info_u32(token, TokenUIAccess).unwrap_or(0) != 0;
        let integrity = token_integrity(token).unwrap_or(0x2000);
        let _ = CloseHandle(token);
        SelfToken { elevated, ui_access, integrity }
    }
}

pub fn process_integrity(pid: u32) -> Option<u32> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut token = HANDLE::default();
        let r = if OpenProcessToken(h, TOKEN_QUERY, &mut token).is_ok() {
            let v = token_integrity(token);
            let _ = CloseHandle(token);
            v
        } else {
            None
        };
        let _ = CloseHandle(h);
        r
    }
}

pub fn permissions_json() -> Value {
    let t = self_token();
    let denied = permissions_denied();
    let desktop = interactive_desktop();
    json!({
        "desktop": {
            "granted": desktop,
            "guidance": if desktop { "An interactive, unlocked desktop is available: UI Automation, input and screen capture can reach apps." } else { "No interactive, unlocked desktop: run aux4 from a signed-in user's desktop session (not a Windows service, a non-interactive scheduled task or SSH) and unlock the screen." }
        },
        "elevated": {
            "granted": t.elevated && !denied,
            "integrity": integrity_name(t.integrity),
            "guidance": if t.elevated && !denied {
                "This process runs elevated (administrator): it can drive both normal and elevated apps."
            } else {
                "This process is not elevated: Windows (UIPI) blocks UI Automation actions and input into apps that run as administrator; normal apps work. To drive an elevated app, run aux4 from an elevated terminal (Run as administrator)."
            }
        },
        "uiAccess": {
            "granted": t.ui_access && !denied,
            "guidance": if t.ui_access && !denied { "uiAccess is set on this process: it can drive elevated apps." } else { "uiAccess is not set (it needs a signed binary installed under Program Files); only needed to drive elevated apps from a non-elevated process." }
        }
    })
}

/// UIPI check before acting on a process: a non-elevated process can't drive an elevated one.
pub fn check_uipi(app: &App) {
    let me = self_token();
    if me.ui_access {
        return;
    }
    if let Some(target) = process_integrity(app.pid) {
        if target > me.integrity {
            fail_json(json!({
                "error": format!(
                    "Cannot drive '{}': it runs with {} integrity (elevated) and this process runs with {} integrity. Windows (UIPI) blocks UI Automation actions and input into elevated apps from non-elevated processes. Run aux4 from an elevated terminal (Run as administrator) to drive it.",
                    app.name, integrity_name(target), integrity_name(me.integrity)
                ),
                "uipi": true,
                "app": app.name,
                "targetIntegrity": integrity_name(target),
                "integrity": integrity_name(me.integrity)
            }));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// monitors + logical coordinates (per-monitor DPI: logical = physical / scale)

#[derive(Clone, Debug)]
pub struct Monitor {
    pub handle: isize,
    pub rect: RECT, // physical
    pub work: RECT,
    pub scale: f64,
    pub primary: bool,
    pub name: String,
}

impl Monitor {
    pub fn logical_rect(&self) -> (f64, f64, f64, f64) {
        let s = self.scale;
        (self.rect.left as f64 / s, self.rect.top as f64 / s, (self.rect.right - self.rect.left) as f64 / s, (self.rect.bottom - self.rect.top) as f64 / s)
    }
}

fn monitor_info(h: HMONITOR) -> Option<Monitor> {
    unsafe {
        let mut mi = MONITORINFOEXW::default();
        mi.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if !GetMonitorInfoW(h, &mut mi as *mut MONITORINFOEXW as *mut MONITORINFO).as_bool() {
            return None;
        }
        let (mut dx, mut dy) = (96u32, 96u32);
        let _ = GetDpiForMonitor(h, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
        Some(Monitor {
            handle: h.0 as isize,
            rect: mi.monitorInfo.rcMonitor,
            work: mi.monitorInfo.rcWork,
            scale: if dx == 0 { 1.0 } else { dx as f64 / 96.0 },
            primary: mi.monitorInfo.dwFlags & 1 != 0,
            name: from_wide(&mi.szDevice),
        })
    }
}

unsafe extern "system" fn enum_monitor(h: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    let v = &mut *(data.0 as *mut Vec<Monitor>);
    if let Some(m) = monitor_info(h) {
        v.push(m);
    }
    BOOL(1)
}

pub fn monitors() -> Vec<Monitor> {
    let mut v: Vec<Monitor> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(enum_monitor), LPARAM(&mut v as *mut _ as isize));
    }
    // primary first, then left-to-right
    v.sort_by(|a, b| b.primary.cmp(&a.primary).then(a.rect.left.cmp(&b.rect.left)).then(a.rect.top.cmp(&b.rect.top)));
    v
}

pub fn monitor_for_physical(p: POINT) -> Monitor {
    unsafe { monitor_info(MonitorFromPoint(p, MONITOR_DEFAULTTONEAREST)) }.unwrap_or(Monitor {
        handle: 0,
        rect: RECT { left: 0, top: 0, right: 1024, bottom: 768 },
        work: RECT { left: 0, top: 0, right: 1024, bottom: 768 },
        scale: 1.0,
        primary: true,
        name: String::new(),
    })
}

pub fn monitor_for_window(h: HWND) -> Monitor {
    unsafe { monitor_info(MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST)) }.unwrap_or_else(|| monitor_for_physical(POINT { x: 0, y: 0 }))
}

pub fn scale_for_physical(x: f64, y: f64) -> f64 {
    monitor_for_physical(POINT { x: x as i32, y: y as i32 }).scale
}

pub fn to_logical(x: f64, y: f64) -> (f64, f64) {
    let s = scale_for_physical(x, y);
    (x / s, y / s)
}

/// Logical screen point -> physical, using the monitor whose logical rect contains it.
pub fn to_physical(x: f64, y: f64) -> (i32, i32) {
    let ms = monitors();
    for m in &ms {
        let (lx, ly, lw, lh) = m.logical_rect();
        if x >= lx && x < lx + lw && y >= ly && y < ly + lh {
            return ((x * m.scale).round() as i32, (y * m.scale).round() as i32);
        }
    }
    let s = unsafe { monitor_info(MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY)) }.map(|m| m.scale).unwrap_or(1.0);
    ((x * s).round() as i32, (y * s).round() as i32)
}

/// A physical rect -> logical frame JSON (scale of the monitor holding its center).
pub fn frame_json(r: &RECT) -> Value {
    let s = scale_for_physical((r.left + r.right) as f64 / 2.0, (r.top + r.bottom) as f64 / 2.0);
    json!({
        "x": num(r.left as f64 / s),
        "y": num(r.top as f64 / s),
        "width": num((r.right - r.left) as f64 / s),
        "height": num((r.bottom - r.top) as f64 / s)
    })
}

pub fn rect_scale(r: &RECT) -> f64 {
    scale_for_physical((r.left + r.right) as f64 / 2.0, (r.top + r.bottom) as f64 / 2.0)
}

pub fn center_json(r: &RECT) -> Value {
    let s = rect_scale(r);
    json!({"x": num((r.left + r.right) as f64 / 2.0 / s), "y": num((r.top + r.bottom) as f64 / 2.0 / s)})
}

/// Center of r relative to the window frame `win` (logical px), the space of `--app` coordinates.
pub fn window_center_json(r: &RECT, win: &RECT) -> Value {
    let s = rect_scale(win);
    json!({
        "x": num(((r.left + r.right) as f64 / 2.0 - win.left as f64) / s),
        "y": num(((r.top + r.bottom) as f64 / 2.0 - win.top as f64) / s)
    })
}

// ---------------------------------------------------------------------------------------------
// windows + apps

#[derive(Clone, Debug)]
pub struct Win {
    pub hwnd: HWND,
    pub pid: u32,
    pub title: String,
    pub class: String,
    pub frame: RECT, // visible frame, physical
    pub minimized: bool,
    pub maximized: bool,
    pub visible: bool,
    pub owner: Option<HWND>,
}

#[derive(Clone, Debug)]
pub struct App {
    pub name: String,
    pub pid: u32,
    pub path: String,
    pub windows: Vec<Win>, // z-order, topmost first
}

impl App {
    pub fn main_window(&self) -> Option<&Win> {
        self.windows.iter().find(|w| w.owner.is_none() && !w.minimized).or_else(|| self.windows.iter().find(|w| w.owner.is_none())).or(self.windows.first())
    }
}

pub fn window_text(h: HWND) -> String {
    unsafe {
        let mut buf = [0u16; 512];
        let n = GetWindowTextW(h, &mut buf);
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}

pub fn class_name(h: HWND) -> String {
    unsafe {
        let mut buf = [0u16; 256];
        let n = GetClassNameW(h, &mut buf);
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}

pub fn window_frame(h: HWND) -> RECT {
    unsafe {
        let mut r = RECT::default();
        if DwmGetWindowAttribute(h, DWMWA_EXTENDED_FRAME_BOUNDS, &mut r as *mut _ as *mut _, std::mem::size_of::<RECT>() as u32).is_ok() && r.right > r.left {
            return r;
        }
        let _ = GetWindowRect(h, &mut r);
        r
    }
}

pub fn window_rect(h: HWND) -> RECT {
    let mut r = RECT::default();
    unsafe {
        let _ = GetWindowRect(h, &mut r);
    }
    r
}

fn is_cloaked(h: HWND) -> bool {
    unsafe {
        let mut c: u32 = 0;
        DwmGetWindowAttribute(h, DWMWA_CLOAKED, &mut c as *mut _ as *mut _, 4).is_ok() && c != 0
    }
}

pub fn window_pid(h: HWND) -> u32 {
    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(h, Some(&mut pid));
    }
    pid
}

pub fn process_path(pid: u32) -> String {
    unsafe {
        let h = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return String::new(),
        };
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(h);
        if ok {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

pub fn exe_stem(path: &str) -> String {
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path);
    file.strip_suffix(".exe").or_else(|| file.strip_suffix(".EXE")).unwrap_or(file).to_string()
}

unsafe extern "system" fn enum_top(h: HWND, data: LPARAM) -> BOOL {
    let v = &mut *(data.0 as *mut Vec<HWND>);
    v.push(h);
    BOOL(1)
}

unsafe extern "system" fn enum_child_pid(h: HWND, data: LPARAM) -> BOOL {
    let (host, out) = &mut *(data.0 as *mut (u32, u32));
    let pid = window_pid(h);
    if pid != *host && class_name(h) == "Windows.UI.Core.CoreWindow" {
        *out = pid;
        return BOOL(0);
    }
    BOOL(1)
}

/// The process that really owns a window: UWP apps are hosted by ApplicationFrameHost.
pub fn effective_pid(h: HWND, pid: u32, class: &str) -> u32 {
    if class != "ApplicationFrameWindow" {
        return pid;
    }
    let mut data: (u32, u32) = (pid, 0);
    unsafe {
        let _ = EnumChildWindows(Some(h), Some(enum_child_pid), LPARAM(&mut data as *mut _ as isize));
    }
    if data.1 != 0 {
        data.1
    } else {
        pid
    }
}

const IGNORED_CLASSES: &[&str] = &["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "Windows.UI.Core.CoreWindow", "IME", "MSCTFIME UI", "tooltips_class32", "NotifyIconOverflowWindow", "TopLevelWindowForOverflowXamlIsland", "XamlExplorerHostIslandWindow"];

/// Top-level windows in z-order (topmost first). `all` includes minimized + untitled visible ones.
pub fn top_windows() -> Vec<Win> {
    let mut hs: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(enum_top), LPARAM(&mut hs as *mut _ as isize));
    }
    let me = unsafe { windows::Win32::System::Threading::GetCurrentProcessId() };
    let mut out = Vec::new();
    for h in hs {
        unsafe {
            if !IsWindowVisible(h).as_bool() {
                continue;
            }
        }
        let class = class_name(h);
        if IGNORED_CLASSES.contains(&class.as_str()) || is_cloaked(h) {
            continue;
        }
        let ex = unsafe { GetWindowLongW(h, GWL_EXSTYLE) } as u32;
        let title = window_text(h);
        let owner = unsafe { GetWindow(h, GW_OWNER) }.ok().filter(|o| !o.0.is_null());
        if ex & WS_EX_TOOLWINDOW.0 != 0 && title.is_empty() {
            continue;
        }
        let frame = window_frame(h);
        if frame.right - frame.left <= 1 || frame.bottom - frame.top <= 1 {
            continue;
        }
        let pid0 = window_pid(h);
        let pid = effective_pid(h, pid0, &class);
        if pid == me {
            continue;
        }
        let minimized = unsafe { IsIconic(h).as_bool() };
        let maximized = unsafe { IsZoomed(h).as_bool() };
        out.push(Win { hwnd: h, pid, title, class, frame, minimized, maximized, visible: true, owner });
    }
    out
}

pub fn apps_list() -> Vec<App> {
    let mut apps: Vec<App> = Vec::new();
    for w in top_windows() {
        if let Some(a) = apps.iter_mut().find(|a| a.pid == w.pid) {
            a.windows.push(w);
        } else {
            let path = process_path(w.pid);
            let name = if path.is_empty() { format!("pid {}", w.pid) } else { exe_stem(&path) };
            apps.push(App { name, pid: w.pid, path, windows: vec![w] });
        }
    }
    apps
}

const ALIASES: &[(&str, &[&str])] = &[
    ("calculator", &["calculatorapp", "win32calc", "calc"]),
    ("calc", &["calculatorapp", "win32calc", "calc"]),
    ("notepad", &["notepad"]),
    ("explorer", &["explorer"]),
    ("file explorer", &["explorer"]),
    ("terminal", &["windowsterminal"]),
    ("paint", &["mspaint"]),
    ("edge", &["msedge"]),
    ("chrome", &["chrome"]),
    ("word", &["winword"]),
    ("excel", &["excel"]),
];

/// Find a running app by exe name (notepad, notepad.exe), alias (Calculator), pid, or window title.
pub fn find_app(query: &str) -> Option<App> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }
    let apps = apps_list();
    let q_stem = q.strip_suffix(".exe").unwrap_or(&q).to_string();
    if let Ok(pid) = q.parse::<u32>() {
        if let Some(a) = apps.iter().find(|a| a.pid == pid) {
            return Some(a.clone());
        }
    }
    if let Some(a) = apps.iter().find(|a| a.name.to_lowercase() == q_stem) {
        return Some(a.clone());
    }
    if q.contains('\\') {
        if let Some(a) = apps.iter().find(|a| a.path.to_lowercase() == q) {
            return Some(a.clone());
        }
    }
    if let Some((_, names)) = ALIASES.iter().find(|(k, _)| *k == q_stem) {
        for n in names.iter() {
            if let Some(a) = apps.iter().find(|a| a.name.to_lowercase() == *n) {
                return Some(a.clone());
            }
        }
    }
    if let Some(a) = apps.iter().find(|a| a.windows.iter().any(|w| w.title.to_lowercase() == q)) {
        return Some(a.clone());
    }
    // "Notepad" also matches "Untitled - Notepad"
    if let Some(a) = apps.iter().find(|a| a.windows.iter().any(|w| {
        let t = w.title.to_lowercase();
        t.ends_with(&format!(" - {}", q)) || t.starts_with(&format!("{} - ", q))
    })) {
        return Some(a.clone());
    }
    None
}

/// Is any process with this exe stem running (even without windows)?
pub fn process_running(pid: u32) -> bool {
    unsafe {
        match OpenProcess(windows::Win32::System::Threading::PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => {
                let r = windows::Win32::System::Threading::WaitForSingleObject(h, 0);
                let _ = CloseHandle(h);
                r.0 == 0x102 // WAIT_TIMEOUT: still running
            }
            Err(_) => false,
        }
    }
}

pub fn foreground_app() -> Option<App> {
    let fg = unsafe { GetForegroundWindow() };
    if fg.0.is_null() {
        return None;
    }
    let root = unsafe { GetAncestor(fg, GA_ROOTOWNER) };
    let class = class_name(root);
    let pid = effective_pid(root, window_pid(root), &class);
    apps_list().into_iter().find(|a| a.pid == pid).or_else(|| {
        let path = process_path(pid);
        Some(App { name: exe_stem(&path), pid, path, windows: vec![] })
    })
}

/// Resolve --app or the foreground app; fails with "App not running" when not found.
pub fn resolve_app(query: &str) -> App {
    if query.trim().is_empty() {
        match foreground_app() {
            Some(a) => a,
            None => fail("No foreground application: pass --app"),
        }
    } else {
        match find_app(query) {
            Some(a) => a,
            None => fail(format!("App not running: {}", query)),
        }
    }
}

pub fn win_json(w: &Win, index: usize) -> Value {
    json!({
        "index": index,
        "ref": index.to_string(),
        "id": hwnd_id(w.hwnd),
        "title": w.title,
        "frame": frame_json(&w.frame),
        "minimized": w.minimized,
        "fullscreen": w.maximized,
        "maximized": w.maximized,
        "dialog": w.owner.is_some() || w.class == "#32770",
        "focused": unsafe { GetForegroundWindow() } == w.hwnd,
        "class": w.class
    })
}

/// Pick one of the app's windows by --title / --index / main window.
pub fn pick_window(app: &App, title: &str, index: &str) -> (usize, Win) {
    if app.windows.is_empty() {
        fail(format!("{} has no windows", app.name));
    }
    if !index.trim().is_empty() {
        let i: usize = index.trim().parse().unwrap_or(usize::MAX);
        if i >= app.windows.len() {
            fail(format!("No window at --index {}: {} has {} window{}", index.trim(), app.name, app.windows.len(), if app.windows.len() == 1 { "" } else { "s" }));
        }
        return (i, app.windows[i].clone());
    }
    if !title.trim().is_empty() {
        let t = title.trim().to_lowercase();
        if let Some(i) = app.windows.iter().position(|w| w.title.to_lowercase() == t) {
            return (i, app.windows[i].clone());
        }
        if let Some(i) = app.windows.iter().position(|w| w.title.to_lowercase().contains(&t)) {
            return (i, app.windows[i].clone());
        }
        let list: Vec<String> = app.windows.iter().map(|w| format!("'{}'", w.title)).collect();
        fail(format!("No window titled '{}' in {}. Windows: {}", title, app.name, list.join(", ")));
    }
    let main = app.main_window().cloned().unwrap();
    let i = app.windows.iter().position(|w| w.hwnd == main.hwnd).unwrap_or(0);
    (i, main)
}

pub fn refresh_window(h: HWND) -> Option<Win> {
    let alive = unsafe { IsWindow(Some(h)).as_bool() };
    if !alive {
        return None;
    }
    let class = class_name(h);
    Some(Win {
        hwnd: h,
        pid: effective_pid(h, window_pid(h), &class),
        title: window_text(h),
        class,
        frame: window_frame(h),
        minimized: unsafe { IsIconic(h).as_bool() },
        maximized: unsafe { IsZoomed(h).as_bool() },
        visible: unsafe { IsWindowVisible(h).as_bool() },
        owner: unsafe { GetWindow(h, GW_OWNER) }.ok().filter(|o| !o.0.is_null()),
    })
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
