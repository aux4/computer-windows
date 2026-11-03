// Command dispatcher + the read-only commands (permissions, screen, cursor, apps, windows, read,
// find, state, screenshot, ocr, wait).

use super::capture::{self, Image};
use super::uia::{self, Node};
use super::*;
use serde_json::Map;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};

pub fn exec(command: &str, a: &[String]) -> ! {
    init();
    match command {
        "permissions" => print_json(permissions_json()),
        "screen" => cmd_screen(),
        "cursor" => {
            let p = super::input::cursor_physical();
            let (x, y) = to_logical(p.x as f64, p.y as f64);
            print_json(json!({"x": num(x), "y": num(y)}))
        }
        "apps" => cmd_apps(),
        "windows" => cmd_windows(),
        "read" => cmd_read(a),
        "find" => cmd_find(a),
        "state" => cmd_state(a),
        "screenshot" => cmd_screenshot(a),
        "ocr" => cmd_ocr(a),
        "wait" => cmd_wait(a),
        "clipboard-read" => match super::clip::read() {
            Ok(t) => print_json(json!({"text": t})),
            Err(e) => fail(e),
        },
        "click" => super::act::click(a),
        "move" => super::act::mouse_move(a),
        "drag" => super::act::drag(a),
        "scroll" => super::act::scroll(a),
        "type" => super::act::type_text(a),
        "shortcut" => super::act::shortcut(a),
        "clipboard-copy" => super::act::clipboard_copy(a),
        "clipboard-paste" => super::act::clipboard_paste(a),
        "set" => super::act::set(a),
        "select" => super::act::select(a),
        "open" => super::appwin::open(a),
        "focus" => super::appwin::focus(a),
        "quit" => super::appwin::quit(a),
        "hide" => super::appwin::hide(a, true),
        "unhide" => super::appwin::hide(a, false),
        "window" => super::appwin::window(a),
        "menu" => super::appwin::menu(a),
        "dialog" => super::appwin::dialog(a),
        _ => fail(format!("Unknown command: {}", command)),
    }
}

/// Hard deadline: whatever is stuck (an app not answering UI Automation, a capture, OCR), the
/// process exits with a JSON error at `seconds`.
pub fn watchdog(seconds: f64, error: Value) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs_f64(seconds.max(0.05)));
        fail_json(error);
    });
}

// ---------------------------------------------------------------------------------------------
// Win32 text controls (Edit / RichEdit): messages the system marshals across processes

pub const WM_CLOSE_MSG: u32 = 0x0010;
pub const EM_GETSEL: u32 = 0x00B0;
pub const EM_SETSEL: u32 = 0x00B1;
pub const EM_REPLACESEL: u32 = 0x00C2;

pub fn is_edit_class(class: &str) -> bool {
    let c = class.to_lowercase();
    c == "edit" || c.starts_with("richedit") || c.contains("richedit")
}

pub fn send_msg(h: HWND, msg: u32, w: usize, l: isize) -> Option<isize> {
    let mut res: usize = 0;
    let r = unsafe { SendMessageTimeoutW(h, msg, WPARAM(w), LPARAM(l), SMTO_ABORTIFHUNG | SMTO_BLOCK, 3000, Some(&mut res)) };
    if r == LRESULT(0) {
        None
    } else {
        Some(res as isize)
    }
}

pub fn get_text(h: HWND) -> String {
    let len = send_msg(h, WM_GETTEXTLENGTH, 0, 0).unwrap_or(0).max(0) as usize;
    let mut buf = vec![0u16; len + 1];
    let n = send_msg(h, WM_GETTEXT, buf.len(), buf.as_mut_ptr() as isize).unwrap_or(0).max(0) as usize;
    String::from_utf16_lossy(&buf[..n.min(len)])
}

/// Selection in UTF-16 units (texts up to 65535 units).
pub fn get_sel(h: HWND) -> (usize, usize) {
    let r = send_msg(h, EM_GETSEL, 0, 0).unwrap_or(0) as usize;
    (r & 0xffff, (r >> 16) & 0xffff)
}

pub fn set_sel(h: HWND, start: usize, end: isize) -> bool {
    send_msg(h, EM_SETSEL, start, end).is_some()
}

pub fn replace_sel(h: HWND, text: &str) -> bool {
    let w = wstr(text);
    send_msg(h, EM_REPLACESEL, 1, w.as_ptr() as isize).is_some()
}

pub fn utf16_to_char(s: &str, u: usize) -> usize {
    let mut units = 0;
    for (i, c) in s.chars().enumerate() {
        if units >= u {
            return i;
        }
        units += c.len_utf16();
    }
    s.chars().count()
}

pub fn char_to_utf16(s: &str, c: usize) -> usize {
    s.chars().take(c).map(|ch| ch.len_utf16()).sum()
}

/// The focused control of an app's GUI thread (works when the app is in the background).
pub fn app_focus_hwnd(w: &Win) -> Option<HWND> {
    unsafe {
        let tid = GetWindowThreadProcessId(w.hwnd, None);
        let mut gi = GUITHREADINFO { cbSize: std::mem::size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        if GetGUIThreadInfo(tid, &mut gi).is_ok() && !gi.hwndFocus.0.is_null() {
            return Some(gi.hwndFocus);
        }
    }
    None
}

/// The app's text control: the focused control if it's a text control, else the first one.
pub fn app_text_target(app: &App) -> Option<(usize, Win, Node)> {
    let (wi, w) = (0..app.windows.len()).find(|i| !app.windows[*i].minimized).map(|i| (i, app.windows[i].clone())).or_else(|| app.windows.first().map(|w| (0, w.clone())))?;
    let nodes = uia::tree(w.hwnd, wi, 32);
    if let Some(fh) = app_focus_hwnd(&w) {
        if let Some(n) = nodes.iter().find(|n| n.hwnd == fh && matches!(n.ctype, 50004 | 50030 | 50003)) {
            return Some((wi, w, n.clone()));
        }
    }
    if let Some(n) = nodes.iter().find(|n| n.focused && matches!(n.ctype, 50004 | 50030)) {
        return Some((wi, w, n.clone()));
    }
    nodes.iter().find(|n| matches!(n.ctype, 50004 | 50030) && !n.value_readonly && n.enabled).map(|n| (wi, w.clone(), n.clone()))
}

// ---------------------------------------------------------------------------------------------
// screen / apps / windows

fn cmd_screen() -> ! {
    let ms = monitors();
    let displays: Vec<Value> = ms
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let (x, y, w, h) = m.logical_rect();
            json!({"index": i, "id": m.handle as i64, "name": m.name, "main": m.primary, "scale": num(m.scale), "x": num(x), "y": num(y), "width": num(w), "height": num(h),
                   "pixels": {"width": m.rect.right - m.rect.left, "height": m.rect.bottom - m.rect.top}})
        })
        .collect();
    print_json(json!({"displays": displays}))
}

fn app_hidden(a: &App) -> bool {
    let tops: Vec<&Win> = a.windows.iter().filter(|w| w.owner.is_none()).collect();
    !tops.is_empty() && tops.iter().all(|w| w.minimized)
}

pub fn app_json(a: &App, fg_pid: u32) -> Value {
    json!({"name": a.name, "pid": a.pid, "path": a.path, "active": a.pid == fg_pid, "hidden": app_hidden(a), "windows": a.windows.len()})
}

fn fg_pid() -> u32 {
    foreground_app().map(|a| a.pid).unwrap_or(0)
}

fn cmd_apps() -> ! {
    let fp = fg_pid();
    let apps: Vec<Value> = apps_list().iter().map(|a| app_json(a, fp)).collect();
    print_json(json!({"apps": apps}))
}

fn cmd_windows() -> ! {
    let apps = apps_list();
    let mut out = Vec::new();
    for w in top_windows() {
        if w.minimized {
            continue;
        }
        let name = apps.iter().find(|a| a.pid == w.pid).map(|a| a.name.clone()).unwrap_or_default();
        out.push(json!({"app": name, "pid": w.pid, "id": hwnd_id(w.hwnd), "title": w.title, "frame": frame_json(&w.frame), "class": w.class}));
    }
    print_json(json!({"windows": out}))
}

// ---------------------------------------------------------------------------------------------
// read / find / state

fn depth_arg(raw: &str, default: usize) -> usize {
    raw.trim().parse().unwrap_or(default)
}

fn cmd_read(a: &[String]) -> ! {
    let app = resolve_app(&arg(a, 0));
    let depth = depth_arg(&arg(a, 1), 10);
    let mut elements = Vec::new();
    for (wi, w) in app.windows.iter().enumerate() {
        for n in uia::tree(w.hwnd, wi, depth) {
            let mut j = uia::node_json(&n, Some(&w.frame));
            j["depth"] = json!(n.depth);
            elements.push(j);
        }
    }
    print_json(json!({"app": app.name, "pid": app.pid, "elements": elements}))
}

pub fn find_matches(app: &App, text: &str, role: &str, loose: bool) -> Vec<Value> {
    let mut out = Vec::new();
    for (wi, w) in app.windows.iter().enumerate() {
        for n in uia::tree(w.hwnd, wi, 64) {
            if !role_matches(&n.role, role) {
                continue;
            }
            let hit = if text.is_empty() {
                true
            } else if loose {
                loose_contains(&n.name, text) || loose_contains(&n.value, text)
            } else {
                plain_contains(&n.name, text) || plain_contains(&n.value, text) || (!n.automation_id.is_empty() && n.automation_id.eq_ignore_ascii_case(text))
            };
            if hit {
                out.push(uia::node_json(&n, Some(&w.frame)));
            }
        }
    }
    out
}

fn cmd_find(a: &[String]) -> ! {
    let (text, role) = (arg(a, 0), arg(a, 1));
    let app = resolve_app(&arg(a, 2));
    let matches = find_matches(&app, &text, &role, false);
    print_json(json!({"app": app.name, "matches": matches}))
}

fn cmd_state(a: &[String]) -> ! {
    let app = resolve_app(&arg(a, 0));
    let depth = depth_arg(&arg(a, 1), 3);
    let limit = depth_arg(&arg(a, 2), 120);
    let fg = foreground_app();
    let fpid = fg.as_ref().map(|f| f.pid).unwrap_or(0);
    let windows: Vec<Value> = app.windows.iter().enumerate().map(|(i, w)| win_json(w, i)).collect();
    let mut elements = Vec::new();
    let mut truncated = false;
    let mut focused = Value::Null;
    if let Some(main) = app.main_window() {
        let wi = app.windows.iter().position(|w| w.hwnd == main.hwnd).unwrap_or(0);
        let nodes = uia::tree(main.hwnd, wi, 64);
        // meaningful depth: count only actionable/text ancestors
        let mut mdepth = vec![0usize; nodes.len()];
        for (i, n) in nodes.iter().enumerate() {
            let meaningful = uia::is_actionable(n) || (uia::is_text_role(n) && (!n.name.is_empty() || !n.value.is_empty()));
            let parent_depth = n.parent.map(|p| mdepth[p]).unwrap_or(0);
            mdepth[i] = parent_depth + if meaningful { 1 } else { 0 };
            if n.focused && focused.is_null() {
                focused = uia::node_json(n, Some(&main.frame));
            }
            if i == 0 || !meaningful || mdepth[i] > depth || n.offscreen {
                continue;
            }
            if elements.len() >= limit {
                truncated = true;
                continue;
            }
            let mut j = json!({"ref": n.r#ref, "role": n.role, "label": n.name, "frame": frame_json(&n.rect)});
            let v = uia::display_value(n);
            if !v.is_empty() {
                j["value"] = json!(v.chars().take(200).collect::<String>());
            }
            if !n.enabled {
                j["enabled"] = json!(false);
            }
            if n.toggle {
                j["checked"] = json!(n.toggle_state == 1);
            }
            elements.push(j);
        }
    }
    let dialog = super::appwin::find_dialog(&app).map(|d| d.json).unwrap_or(Value::Null);
    let mut out = json!({
        "app": {"name": app.name, "pid": app.pid, "path": app.path, "frontmost": app.pid == fpid, "hidden": app_hidden(&app)},
        "frontmost": fg.map(|f| json!({"name": f.name, "pid": f.pid})).unwrap_or(Value::Null),
        "windows": windows,
        "focused": focused,
        "dialog": dialog,
        "elements": elements,
    });
    if truncated {
        out["truncated"] = json!(true);
        out["note"] = json!(format!("Only the first {} elements are listed: raise --limit, or use find/read for the rest", limit));
    }
    print_json(out)
}

// ---------------------------------------------------------------------------------------------
// screenshot

fn temp_png(prefix: &str) -> String {
    let dir = std::env::temp_dir();
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    path_str(&dir.join(format!("{}-{}.png", prefix, ts)))
}

struct Shot {
    img: Image,
    frame: RECT, // physical area the image shows
    window: Option<(usize, Win)>,
    method: &'static str,
}

fn capture_target(app_arg: &str, display: &str, mode: Mode) -> (Option<App>, Shot, Map<String, Value>) {
    let mut extra = Map::new();
    if !app_arg.is_empty() {
        let app = resolve_app(app_arg);
        let idx = (0..app.windows.len()).find(|i| !app.windows[*i].minimized && app.windows[*i].owner.is_none()).or_else(|| (0..app.windows.len()).find(|i| !app.windows[*i].minimized));
        let (wi, w) = match idx {
            Some(i) => (i, app.windows[i].clone()),
            None => {
                // only minimized windows: restore without activating (auto), else error
                if mode == Mode::Background || app.windows.is_empty() {
                    fail(format!("Background mode not possible for capture: no on-screen window for app: {}", app_arg));
                }
                let w = app.main_window().cloned().unwrap();
                unsafe {
                    let _ = ShowWindow(w.hwnd, SW_SHOWNOACTIVATE);
                }
                sleep_ms(400);
                extra.insert("restoredFromMinimized".into(), json!(true));
                let w2 = refresh_window(w.hwnd).unwrap_or(w);
                (app.windows.iter().position(|x| x.hwnd == w2.hwnd).unwrap_or(0), w2)
            }
        };
        let (img, method) = match capture::capture_window(w.hwnd) {
            Ok(r) => r,
            Err(e) => fail(format!("Failed to capture window for app '{}': {}", app_arg, e)),
        };
        let frame = window_frame(w.hwnd);
        return (Some(app), Shot { img, frame, window: Some((wi, w)), method }, extra);
    }
    let ms = monitors();
    let di: usize = display.trim().parse().unwrap_or(0);
    let m = match ms.get(di) {
        Some(m) => m.clone(),
        None => fail(format!("Invalid display index: {}", display)),
    };
    let img = match capture::capture_screen_rect(&m.rect) {
        Ok(i) => i,
        Err(e) => fail(format!("Failed to capture display {}: {}", di, e)),
    };
    extra.insert("display".into(), json!(di));
    (None, Shot { img, frame: m.rect, window: None, method: "bitblt" }, extra)
}

fn cmd_screenshot(a: &[String]) -> ! {
    let (output, display, app_arg) = (arg(a, 0), arg(a, 1), arg(a, 2));
    let max_width: u32 = arg(a, 3).trim().parse::<f64>().map(|v| v as u32).unwrap_or(1568);
    let annotate = parse_bool("annotate", &arg(a, 4), false);
    let roles_all = arg(a, 5).to_lowercase() == "all";
    let limit: usize = arg(a, 6).trim().parse().unwrap_or(150);
    watchdog(30.0, json!({"error": "Screen capture did not respond within 30s", "phase": "capture"}));
    let (app, shot, extra) = capture_target(&app_arg, &display, Mode::Auto);
    let logical_w = (shot.frame.right - shot.frame.left) as f64 / rect_scale(&shot.frame);
    let mut img = shot.img.clone();
    if img.width > max_width {
        img = img.resize_to_width(max_width);
    }
    let scale = img.width as f64 / logical_w.max(1.0);
    let mut out = Map::new();
    for (k, v) in extra {
        out.insert(k, v);
    }
    if annotate {
        let target = app.clone().or_else(foreground_app);
        let target = match target {
            Some(t) => t,
            None => fail("No frontmost application to annotate: pass --app"),
        };
        let (marks, truncated) = annotate_marks(&mut img, &target, &shot, scale, roles_all, limit);
        out.insert("annotated".into(), json!(true));
        out.insert("marks".into(), Value::Array(marks));
        out.insert("roles".into(), json!(if roles_all { "all" } else { "actionable" }));
        if truncated > 0 {
            out.insert("truncated".into(), json!(truncated));
        }
        if app.is_none() {
            out.insert("app".into(), json!(target.name));
        }
    }
    let path = if output.is_empty() { temp_png("aux4-computer-windows") } else { output };
    if let Err(e) = img.save_png(&path) {
        fail(e);
    }
    out.insert("path".into(), json!(path));
    out.insert("width".into(), json!(img.width));
    out.insert("height".into(), json!(img.height));
    out.insert("scale".into(), num(scale));
    out.insert("method".into(), json!(shot.method));
    out.insert("frame".into(), frame_json(&shot.frame));
    if let (Some(app), Some((wi, w))) = (&app, &shot.window) {
        out.insert("app".into(), json!(app.name));
        out.insert("window".into(), json!({"frame": frame_json(&w.frame), "id": hwnd_id(w.hwnd), "title": w.title, "ref": wi.to_string()}));
    }
    print_json(Value::Object(out))
}

fn annotate_marks(img: &mut Image, app: &App, shot: &Shot, scale: f64, all: bool, limit: usize) -> (Vec<Value>, usize) {
    let area = shot.frame;
    let s_area = rect_scale(&area);
    let windows: Vec<(usize, Win)> = match &shot.window {
        Some((wi, w)) => vec![(*wi, w.clone())],
        None => app.windows.iter().enumerate().filter(|(_, w)| !w.minimized).map(|(i, w)| (i, w.clone())).collect(),
    };
    let mut cands: Vec<(Node, RECT)> = Vec::new();
    for (wi, w) in &windows {
        let nodes = uia::tree(w.hwnd, *wi, 64);
        for (i, n) in nodes.iter().enumerate() {
            if i == 0 || n.offscreen || n.rect.right - n.rect.left < 3 || n.rect.bottom - n.rect.top < 3 {
                continue;
            }
            let wanted = uia::is_actionable(n) || (all && (uia::is_text_role(n) || n.ctype == 50006));
            if !wanted {
                continue;
            }
            // skip children of a leaf control already marked (a button's label text)
            if let Some(p) = n.parent {
                let pn = &nodes[p];
                if p != 0 && matches!(pn.ctype, 50000 | 50002 | 50013 | 50005 | 50011 | 50031) {
                    continue;
                }
            }
            // clip to the window
            let r = RECT { left: n.rect.left.max(w.frame.left), top: n.rect.top.max(w.frame.top), right: n.rect.right.min(w.frame.right), bottom: n.rect.bottom.min(w.frame.bottom) };
            if r.right - r.left < 3 || r.bottom - r.top < 3 {
                continue;
            }
            cands.push((n.clone(), r));
        }
    }
    // reading order: rows of ~8 logical px, then x
    cands.sort_by(|a, b| {
        let ra = ((a.1.top + a.1.bottom) as f64 / 2.0 / s_area / 8.0).round() as i64;
        let rb = ((b.1.top + b.1.bottom) as f64 / 2.0 / s_area / 8.0).round() as i64;
        ra.cmp(&rb).then(a.1.left.cmp(&b.1.left))
    });
    let total = cands.len();
    let palette: [[u8; 3]; 6] = [[230, 25, 75], [0, 130, 200], [60, 180, 75], [245, 130, 48], [145, 30, 180], [0, 128, 128]];
    let mut marks = Vec::new();
    let px = scale / 1.0; // image px per logical px
    let label_scale = (2.0 * px).round().max(1.0) as i32;
    for (k, (n, r)) in cands.into_iter().take(limit).enumerate() {
        let num_ = k + 1;
        let ix = ((r.left - area.left) as f64 / s_area * px).round() as i32;
        let iy = ((r.top - area.top) as f64 / s_area * px).round() as i32;
        let iw = ((r.right - r.left) as f64 / s_area * px).round() as i32;
        let ih = ((r.bottom - r.top) as f64 / s_area * px).round() as i32;
        let c = palette[k % palette.len()];
        img.rect(ix, iy, iw, ih, c, label_scale.max(1));
        img.label(ix, iy, num_, c, label_scale);
        let wframe = windows.iter().find(|(wi, _)| n.r#ref.split('.').next() == Some(&wi.to_string())).map(|(_, w)| w.frame).unwrap_or(area);
        marks.push(json!({
            "n": num_,
            "ref": n.r#ref,
            "role": n.role,
            "label": n.name,
            "frame": frame_json(&n.rect),
            "center": center_json(&n.rect),
            "windowCenter": window_center_json(&n.rect, &wframe),
            "image": {"x": ix, "y": iy, "width": iw, "height": ih}
        }));
    }
    (marks, total.saturating_sub(limit))
}

// ---------------------------------------------------------------------------------------------
// ocr

pub struct OcrOut {
    pub matches: Vec<Value>,
    pub texts: Vec<String>,
    pub meta: Map<String, Value>,
}

/// Recognize text in an app's window / the main display / an image; positions in logical px.
pub fn run_ocr(app_arg: &str, image: &str, needle: &str, languages: &str, img_scale: f64, mode: Mode) -> Result<OcrOut, String> {
    let mut meta = Map::new();
    let (img, frame_logical, win_frame): (Image, (f64, f64, f64), Option<RECT>) = if !image.is_empty() {
        let img = Image::load_png(image)?;
        meta.insert("source".into(), json!("image"));
        meta.insert("image".into(), json!(image));
        (img, (0.0, 0.0, img_scale.max(0.0001)), None)
    } else {
        let (app, shot, extra) = capture_target(app_arg, "0", mode);
        for (k, v) in extra {
            meta.insert(k, v);
        }
        let s = rect_scale(&shot.frame);
        let px_per_logical = shot.img.width as f64 / ((shot.frame.right - shot.frame.left) as f64 / s).max(1.0);
        meta.insert("source".into(), json!(if shot.window.is_some() { "window" } else { "display" }));
        meta.insert("method".into(), json!(shot.method));
        if let (Some(app), Some((wi, w))) = (&app, &shot.window) {
            meta.insert("app".into(), json!(app.name));
            meta.insert("window".into(), json!({"frame": frame_json(&w.frame), "id": hwnd_id(w.hwnd), "title": w.title, "ref": wi.to_string()}));
        }
        let wf = shot.window.as_ref().map(|(_, w)| w.frame);
        (shot.img, (shot.frame.left as f64 / s, shot.frame.top as f64 / s, px_per_logical), wf)
    };
    meta.insert("width".into(), json!(img.width));
    meta.insert("height".into(), json!(img.height));
    meta.insert("scale".into(), num(frame_logical.2));
    let (lines, lang) = super::ocr::recognize(&img, languages)?;
    meta.insert("language".into(), json!(lang));
    let (ox, oy, pxs) = frame_logical;
    let to_frame = |x: f64, y: f64, w: f64, h: f64| -> (Value, Value, Option<Value>) {
        let (lx, ly, lw, lh) = (ox + x / pxs, oy + y / pxs, w / pxs, h / pxs);
        let frame = json!({"x": num(lx), "y": num(ly), "width": num(lw), "height": num(lh)});
        let center = json!({"x": num(lx + lw / 2.0), "y": num(ly + lh / 2.0)});
        let wc = win_frame.map(|_| json!({"x": num(x / pxs + lw / 2.0), "y": num(y / pxs + lh / 2.0)}));
        (frame, center, wc)
    };
    let mut matches = Vec::new();
    let mut texts = Vec::new();
    for l in &lines {
        texts.push(l.text.clone());
        if !needle.is_empty() && !loose_contains(&l.text, needle) {
            continue;
        }
        let (x, y, w, h) = l.bounds();
        let (frame, center, wc) = to_frame(x, y, w, h);
        let mut m = json!({"text": l.text, "frame": frame, "center": center});
        if let Some(wc) = wc {
            m["windowCenter"] = wc;
        }
        if !needle.is_empty() {
            if let Some(ws) = super::ocr::match_words(l, needle) {
                let owned: Vec<super::ocr::Word> = ws.iter().map(|w| super::ocr::Word { text: w.text.clone(), x: w.x, y: w.y, w: w.w, h: w.h }).collect();
                let (x, y, w, h) = super::ocr::bounds_of(&owned);
                let (frame, center, wc) = to_frame(x, y, w, h);
                let mut mm = json!({"frame": frame, "center": center});
                if let Some(wc) = wc {
                    mm["windowCenter"] = wc;
                }
                m["match"] = mm;
            }
        }
        matches.push(m);
    }
    Ok(OcrOut { matches, texts, meta })
}

fn cmd_ocr(a: &[String]) -> ! {
    let (app_arg, image, text, level, languages) = (arg(a, 0), arg(a, 1), arg(a, 2), arg(a, 3), arg(a, 4));
    let mode = parse_mode(&arg(a, 5));
    let img_scale = parse_number("scale", &arg(a, 6)).unwrap_or(1.0);
    let timeout = parse_duration("timeout", &arg(a, 7), 60.0);
    watchdog(timeout, json!({"error": format!("Text recognition did not finish within {}", format_seconds(timeout)), "phase": "recognition"}));
    match run_ocr(&app_arg, &image, &text, &languages, img_scale, mode) {
        Ok(o) => {
            let mut out = o.meta;
            out.insert("matches".into(), Value::Array(o.matches));
            out.insert("level".into(), json!(if level.is_empty() { "accurate".to_string() } else { level.to_lowercase() }));
            if image.is_empty() {
                out.insert("mode".into(), json!("background"));
            }
            if !text.is_empty() {
                out.insert("text".into(), json!(text));
            }
            print_json(Value::Object(out))
        }
        Err(e) => fail(e),
    }
}

// ---------------------------------------------------------------------------------------------
// wait

fn cmd_wait(a: &[String]) -> ! {
    let (text, role, app_arg) = (arg(a, 0), arg(a, 1), arg(a, 2));
    let gone = parse_bool("gone", &arg(a, 3), false);
    let source = match arg(a, 4).to_lowercase().as_str() {
        "" | "auto" => "auto",
        "ax" | "uia" => "ax",
        _ => "ocr",
    };
    let timeout = parse_duration("timeout", &arg(a, 5), 10.0);
    let interval = parse_duration("interval", &arg(a, 6), 0.5);
    let condition = if !text.is_empty() {
        format!("text '{}'{}", text, if role.is_empty() { String::new() } else { format!(" ({})", role) })
    } else if !role.is_empty() {
        format!("an element with role {}", role)
    } else {
        format!("a window of {}", app_arg)
    };
    let place = if app_arg.is_empty() { "the frontmost app".to_string() } else { app_arg.clone() };
    let start = std::time::Instant::now();
    let last: std::sync::Arc<std::sync::Mutex<Value>> = std::sync::Arc::new(std::sync::Mutex::new(json!({})));
    {
        let last = last.clone();
        let cond = condition.clone();
        let place = place.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs_f64(timeout + 0.25));
            let seen = last.lock().map(|v| v.clone()).unwrap_or(json!({}));
            fail_json(json!({"error": format!("Timed out after {} waiting for {} {} in {}", format_seconds(timeout), cond, if gone { "to be gone" } else { "to appear" }, place), "timeout": format_seconds(timeout), "lastSeen": seen}));
        });
    }
    loop {
        let check_start = std::time::Instant::now();
        let mut seen = Map::new();
        let found: Option<(Vec<Value>, &str)> = (|| {
            // app alone: running with an on-screen window
            if text.is_empty() && role.is_empty() {
                let app = find_app(&app_arg);
                let running = app.as_ref().map(|a| a.windows.iter().any(|w| !w.minimized)).unwrap_or(false);
                seen.insert("running".into(), json!(app.is_some()));
                seen.insert("window".into(), json!(running));
                if running {
                    let app = app.unwrap();
                    let ws: Vec<Value> = app.windows.iter().enumerate().filter(|(_, w)| !w.minimized).map(|(i, w)| win_json(w, i)).collect();
                    return Some((ws, "app"));
                }
                return None;
            }
            let app = if app_arg.is_empty() { foreground_app() } else { find_app(&app_arg) };
            seen.insert("running".into(), json!(app.is_some()));
            if source != "ocr" {
                if let Some(app) = &app {
                    let m = find_matches(app, &text, &role, true);
                    let sample: Vec<String> = find_matches(app, "", "", true).iter().filter_map(|v| v["label"].as_str().map(|s| s.to_string())).filter(|s| !s.is_empty()).take(12).collect();
                    seen.insert("ax".into(), json!(sample));
                    if !m.is_empty() {
                        return Some((m, "ax"));
                    }
                }
            }
            if source != "ax" && !text.is_empty() && role.is_empty() {
                let target = if app_arg.is_empty() { String::new() } else { app_arg.clone() };
                if target.is_empty() || app.is_some() {
                    if let Ok(o) = run_ocr(&target, "", &text, "", 1.0, Mode::Background) {
                        seen.insert("ocr".into(), json!(o.texts.iter().take(12).collect::<Vec<_>>()));
                        if !o.matches.is_empty() {
                            return Some((o.matches, "ocr"));
                        }
                    }
                }
            }
            None
        })();
        if let Ok(mut l) = last.lock() {
            *l = Value::Object(seen);
        }
        let waited = (start.elapsed().as_secs_f64() * 100.0).round() / 100.0;
        match (found, gone) {
            (Some((matches, src)), false) => {
                let mut out = json!({"matches": matches, "source": src, "waited": waited});
                if !app_arg.is_empty() {
                    out["app"] = json!(app_arg);
                }
                print_json(out)
            }
            (None, true) => {
                let mut out = json!({"matches": [], "gone": true, "waited": waited});
                if !app_arg.is_empty() {
                    out["app"] = json!(app_arg);
                }
                print_json(out)
            }
            _ => {}
        }
        let spent = check_start.elapsed().as_secs_f64();
        if start.elapsed().as_secs_f64() + interval.max(0.05) > timeout + 0.2 {
            // last chance passed; the watchdog reports the timeout
            std::thread::sleep(std::time::Duration::from_secs_f64((timeout + 0.3 - start.elapsed().as_secs_f64()).max(0.0)));
        }
        if spent < interval {
            std::thread::sleep(std::time::Duration::from_secs_f64(interval - spent));
        }
    }
}
