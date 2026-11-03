// Apps, windows, menus and dialogs: open, focus, quit, hide/unhide, window ops, menu, dialog.

use super::cmds::*;
use super::input::{self, Fg};
use super::uia::{self, Node};
use super::*;
use crate::{action_base, action_mode_index, action_name, menu_path_parts};
use serde_json::Map;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};

fn base_and_mode(command: &str, a: &[String]) -> (Map<String, Value>, Option<(bool, String)>, Mode, String) {
    let (base, plan) = action_base(command, a);
    let mode = action_mode_index(command).map(|i| parse_mode(&arg(a, i))).unwrap_or(Mode::Auto);
    (base, plan, mode, action_name(command, a))
}

fn done(pairs: Vec<(&str, Value)>) -> Outcome {
    let mut m = Map::new();
    for (k, v) in pairs {
        m.insert(k.to_string(), v);
    }
    Outcome::Done(m)
}

fn fg_for(app: &App) -> Fg {
    Fg { target: app.main_window().map(|w| w.hwnd), name: app.name.clone() }
}

// ---------------------------------------------------------------------------------------------
// open / focus / quit / hide

fn shell_execute(file: &str, params: &str, show: SHOW_WINDOW_CMD) -> Result<Option<u32>, String> {
    let f = wstr(file);
    let p = wstr(params);
    let verb = wstr("open");
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_FLAG_NO_UI | SEE_MASK_NOASYNC,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(f.as_ptr()),
        lpParameters: if params.is_empty() { PCWSTR::null() } else { PCWSTR(p.as_ptr()) },
        nShow: show.0,
        ..Default::default()
    };
    unsafe {
        ShellExecuteExW(&mut info).map_err(|e| e.message())?;
        let pid = if !info.hProcess.0.is_null() {
            let pid = windows::Win32::System::Threading::GetProcessId(info.hProcess);
            let _ = windows::Win32::Foundation::CloseHandle(info.hProcess);
            Some(pid)
        } else {
            None
        };
        Ok(pid)
    }
}

pub fn open(a: &[String]) -> ! {
    let (mut base, _, mode, _) = base_and_mode("open", a);
    let (app_q, file, url) = (arg(a, 0), arg(a, 2), arg(a, 3));
    let hidden = parse_bool("hidden", &arg(a, 5), false);
    let wait = parse_duration_value(&arg(a, 4)).ok().flatten();
    if dry_run() {
        base.insert("dryRun".into(), json!(true));
        base.insert("mode".into(), json!(if mode == Mode::Foreground { "foreground" } else { "background" }));
        print_json(Value::Object(base));
    }
    let prev_fg = input::foreground_window();
    let before: Vec<isize> = top_windows().iter().map(|w| w.hwnd.0 as isize).collect();
    // an app that's already running is left as it is (no second instance, nothing activated)
    if file.is_empty() && url.is_empty() {
        if let Some(app) = find_app(&app_q) {
            base.insert("app".into(), json!(app.name));
            base.insert("pid".into(), json!(app.pid));
            base.insert("alreadyRunning".into(), json!(true));
            if mode == Mode::Foreground {
                if let Some(w) = app.main_window() {
                    input::bring_to_front(w.hwnd);
                }
            }
            if let Some((wi, w)) = app.main_window().map(|w| (app.windows.iter().position(|x| x.hwnd == w.hwnd).unwrap_or(0), w.clone())) {
                base.insert("window".into(), json!({"frame": frame_json(&w.frame), "ref": wi.to_string(), "title": w.title}));
            }
            base.insert("waited".into(), json!(0));
            base.insert("mode".into(), json!(if mode == Mode::Foreground { "foreground" } else { "background" }));
            print_json(Value::Object(base));
        }
    }
    let show = if hidden { SW_SHOWMINNOACTIVE } else if mode == Mode::Foreground { SW_SHOWNORMAL } else { SW_SHOWNOACTIVATE };
    let target_file = if !url.is_empty() { url.clone() } else if !file.is_empty() && app_q.is_empty() { file.clone() } else { app_q.clone() };
    let params = if !app_q.is_empty() && !file.is_empty() {
        format!("\"{}\"", base.get("file").and_then(|v| v.as_str()).unwrap_or(&file))
    } else if !app_q.is_empty() && !url.is_empty() {
        url.clone()
    } else {
        String::new()
    };
    let (target_file, params) = if !app_q.is_empty() && !url.is_empty() { (app_q.clone(), params) } else { (target_file, params) };
    let launched = match shell_execute(&target_file, &params, show) {
        Ok(p) => p,
        Err(e) => {
            // "Calculator" -> calc.exe, "notepad" -> notepad.exe
            let alt = match app_q.to_lowercase().as_str() {
                "calculator" => Some("calc.exe".to_string()),
                q if !q.is_empty() && !q.ends_with(".exe") && !q.contains('\\') && file.is_empty() && url.is_empty() => Some(format!("{}.exe", q)),
                _ => None,
            };
            match alt.map(|f| shell_execute(&f, &params, show)) {
                Some(Ok(p)) => p,
                _ => {
                    if !file.is_empty() || !url.is_empty() {
                        fail(format!("Failed to open {}: {}", if url.is_empty() { &file } else { &url }, e.trim()));
                    }
                    fail(format!("Failed to open app '{}': {}", app_q, e.trim()));
                }
            }
        }
    };
    if let Some(pid) = launched {
        base.insert("pid".into(), json!(pid));
    }
    let start = std::time::Instant::now();
    let file_stem = std::path::Path::new(&file).file_name().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
    let deadline = wait.unwrap_or(1.5);
    let mut found: Option<Win> = None;
    while start.elapsed().as_secs_f64() < deadline {
        for w in top_windows() {
            if before.contains(&(w.hwnd.0 as isize)) {
                continue;
            }
            let pid_ok = launched.map(|p| p == w.pid).unwrap_or(false);
            let title_ok = !file_stem.is_empty() && w.title.to_lowercase().contains(&file_stem);
            let name_ok = !app_q.is_empty() && find_app(&app_q).map(|a| a.pid == w.pid).unwrap_or(false);
            if pid_ok || title_ok || name_ok || (app_q.is_empty() && file_stem.is_empty()) {
                found = Some(w);
                break;
            }
        }
        if found.is_some() {
            break;
        }
        sleep_ms(100);
    }
    let mut restored = None;
    if mode != Mode::Foreground {
        sleep_ms(150);
        restored = input::restore_foreground_if_changed(prev_fg);
        if mode == Mode::Background && restored == Some(false) {
            fail("Background mode not possible for open: the app brought itself to the front and the previous window could not be restored");
        }
    } else if let Some(w) = &found {
        input::bring_to_front(w.hwnd);
    }
    if let Some(w) = &found {
        let app = apps_list().into_iter().find(|a| a.pid == w.pid);
        if let Some(app) = &app {
            base.insert("app".into(), json!(app.name));
            base.insert("pid".into(), json!(app.pid));
            let wi = app.windows.iter().position(|x| x.hwnd == w.hwnd).unwrap_or(0);
            base.insert("window".into(), json!({"frame": frame_json(&w.frame), "ref": wi.to_string(), "title": w.title}));
        }
    }
    if wait.is_some() {
        if found.is_none() {
            fail(format!("Opened {}, but no window appeared within {}", base.get("opened").and_then(|v| v.as_str()).unwrap_or(""), format_seconds(deadline)));
        }
        base.insert("waited".into(), num(start.elapsed().as_secs_f64()));
    }
    if let Some(r) = restored {
        base.insert("restored".into(), json!({"frontmost": r}));
    }
    base.insert("mode".into(), json!(if mode == Mode::Foreground { "foreground" } else { "background" }));
    print_json(Value::Object(base))
}

pub fn focus(a: &[String]) -> ! {
    let app = resolve_app(&arg(a, 0));
    check_uipi(&app);
    let w = match app.main_window() {
        Some(w) => w.clone(),
        None => fail(format!("{} has no windows", app.name)),
    };
    if !input::bring_to_front(w.hwnd) {
        fail(format!("Could not bring '{}' to the front", app.name));
    }
    print_json(json!({"focused": app.name, "success": true, "mode": "foreground", "pid": app.pid}))
}

const PROTECTED: &[&str] = &["explorer", "dwm", "csrss", "winlogon", "wininit", "lsass", "services", "smss", "svchost", "sihost", "ctfmon", "fontdrvhost", "shellexperiencehost", "startmenuexperiencehost", "searchhost", "textinputhost", "runtimebroker", "applicationframehost", "taskhostw", "conhost", "system"];

pub fn quit(a: &[String]) -> ! {
    let q = arg(a, 0);
    let force = parse_bool("force", &arg(a, 1), false);
    let timeout = parse_duration("timeout", &arg(a, 2), 5.0);
    let qs = q.trim().to_lowercase();
    let qs = qs.strip_suffix(".exe").unwrap_or(&qs).to_string();
    if PROTECTED.contains(&qs.as_str()) {
        fail(format!("Refusing to quit {}: Windows needs it to keep the session working", q.trim()));
    }
    let app = resolve_app(&q);
    let lname = app.name.to_lowercase();
    if app.pid == std::process::id() {
        fail("Refusing to quit this process");
    }
    if PROTECTED.contains(&lname.as_str()) {
        fail(format!("Refusing to quit {} ({}): Windows needs it to keep the session working", app.name, app.path));
    }
    check_uipi(&app);
    let method = if force { "TerminateProcess" } else { "WM_CLOSE" };
    let mut out = json!({"app": app.name, "pid": app.pid, "path": app.path, "force": force, "method": method, "mode": "background"});
    if dry_run() {
        out["dryRun"] = json!(true);
        print_json(out);
    }
    let start = std::time::Instant::now();
    if force {
        unsafe {
            match windows::Win32::System::Threading::OpenProcess(windows::Win32::System::Threading::PROCESS_TERMINATE, false, app.pid) {
                Ok(h) => {
                    let r = windows::Win32::System::Threading::TerminateProcess(h, 1);
                    let _ = windows::Win32::Foundation::CloseHandle(h);
                    if let Err(e) = r {
                        fail(format!("Could not force-quit {}: {}", app.name, e.message()));
                    }
                }
                Err(e) => fail(format!("Could not force-quit {}: {}", app.name, e.message())),
            }
        }
    } else {
        for w in app.windows.iter().filter(|w| w.owner.is_none()) {
            unsafe {
                let _ = PostMessageW(Some(w.hwnd), WM_CLOSE_MSG, WPARAM(0), LPARAM(0));
            }
        }
    }
    loop {
        if !process_running(app.pid) {
            out["terminated"] = json!(true);
            out["waited"] = num(start.elapsed().as_secs_f64());
            print_json(out);
        }
        if !force && start.elapsed().as_secs_f64() > 0.3 {
            if let Some(app2) = find_app(&app.pid.to_string()) {
                if let Some(d) = find_dialog(&app2) {
                    out["terminated"] = json!(false);
                    out["dialog"] = d.json;
                    out["waited"] = num(start.elapsed().as_secs_f64());
                    out["note"] = json!(format!("{} asked before quitting: answer it with 'aux4 computer win dialog --app {} --press <button>', then quit again", app.name, app.name));
                    print_json(out);
                }
            }
        }
        if start.elapsed().as_secs_f64() > timeout {
            out["terminated"] = json!(false);
            out["waited"] = num(start.elapsed().as_secs_f64());
            out["note"] = json!(format!("{} did not exit within {}; use --force true to kill it (unsaved changes are lost)", app.name, format_seconds(timeout)));
            print_json(out);
        }
        sleep_ms(100);
    }
}

pub fn hide(a: &[String], hide: bool) -> ! {
    let app = resolve_app(&arg(a, 0));
    let tops: Vec<Win> = app.windows.iter().filter(|w| w.owner.is_none()).cloned().collect();
    let was_hidden = !tops.is_empty() && tops.iter().all(|w| w.minimized);
    let mut out = json!({"app": app.name, "pid": app.pid, "before": {"hidden": was_hidden}, "hidden": hide, "mode": "background",
        "method": if hide { "ShowWindow(SW_SHOWMINNOACTIVE)" } else { "ShowWindow(SW_SHOWNOACTIVATE)" }});
    if dry_run() {
        out["dryRun"] = json!(true);
        print_json(out);
    }
    check_uipi(&app);
    let prev = input::foreground_window();
    let mut changed = false;
    for w in &tops {
        if hide && !w.minimized {
            unsafe {
                let _ = ShowWindow(w.hwnd, SW_SHOWMINNOACTIVE);
            }
            changed = true;
        } else if !hide && w.minimized {
            unsafe {
                let _ = ShowWindow(w.hwnd, SW_SHOWNOACTIVATE);
            }
            changed = true;
        }
    }
    sleep_ms(250);
    if let Some(r) = input::restore_foreground_if_changed(prev) {
        out["restored"] = json!({"frontmost": r});
    }
    let now = find_app(&app.pid.to_string()).map(|a2| {
        let t: Vec<Win> = a2.windows.into_iter().filter(|w| w.owner.is_none()).collect();
        !t.is_empty() && t.iter().all(|w| w.minimized)
    });
    out["changed"] = json!(changed);
    if let Some(h) = now {
        out["hidden"] = json!(h);
    }
    print_json(out)
}

// ---------------------------------------------------------------------------------------------
// window ops

pub fn window(a: &[String]) -> ! {
    let action = arg(a, 0).to_lowercase();
    if action == "list" {
        let app = resolve_app(&arg(a, 1));
        let ws: Vec<Value> = app.windows.iter().enumerate().map(|(i, w)| win_json(w, i)).collect();
        print_json(json!({"app": app.name, "pid": app.pid, "windows": ws}));
    }
    let (mut base, up, mode, name) = base_and_mode("window", a);
    let app = resolve_app(&arg(a, 1));
    check_uipi(&app);
    base.insert("app".into(), json!(app.name));
    let (wi, w) = pick_window(&app, &arg(a, 2), &arg(a, 3));
    base.insert("before".into(), win_json(&w, wi));
    let h = w.hwnd;
    let fg = Fg { target: Some(h), name: app.name.clone() };
    let after = move || refresh_window(h).map(|x| win_json(&x, wi)).unwrap_or(Value::Null);
    let app_name = app.name.clone();
    let (p5, p6) = (arg(a, 5), arg(a, 6));
    let act = action.clone();
    let bg = Box::new(move || -> Outcome {
        let prev = input::foreground_window();
        let r: Result<&str, String> = unsafe {
            match act.as_str() {
                "move" | "resize" => {
                    let wr = window_rect(h);
                    let fr = window_frame(h);
                    let s = rect_scale(&fr);
                    // visible frame -> window rect offsets (invisible resize borders)
                    let (bl, bt, br, bb) = (fr.left - wr.left, fr.top - wr.top, wr.right - fr.right, wr.bottom - fr.bottom);
                    let (mut x, mut y, mut cw, mut ch) = (wr.left, wr.top, wr.right - wr.left, wr.bottom - wr.top);
                    if act == "move" {
                        let (px, py) = to_physical(p5.trim().parse().unwrap_or(0.0), p6.trim().parse().unwrap_or(0.0));
                        x = px - bl;
                        y = py - bt;
                    } else {
                        if let Ok(wv) = p5.trim().parse::<f64>() {
                            cw = (wv * s).round() as i32 + bl + br;
                        }
                        if let Ok(hv) = p6.trim().parse::<f64>() {
                            ch = (hv * s).round() as i32 + bt + bb;
                        }
                    }
                    let flags = SWP_NOACTIVATE | SWP_NOZORDER | if act == "move" { SWP_NOSIZE } else { SWP_NOMOVE };
                    SetWindowPos(h, None, x, y, cw, ch, flags).map(|_| "SetWindowPos").map_err(|e| e.message())
                }
                "minimize" => {
                    let _ = ShowWindow(h, SW_SHOWMINNOACTIVE);
                    Ok("ShowWindow(SW_SHOWMINNOACTIVE)")
                }
                "restore" => {
                    let _ = ShowWindow(h, SW_SHOWNOACTIVATE);
                    Ok("ShowWindow(SW_SHOWNOACTIVATE)")
                }
                "close" => PostMessageW(Some(h), WM_CLOSE_MSG, WPARAM(0), LPARAM(0)).map(|_| "WM_CLOSE").map_err(|e| e.message()),
                "fullscreen" => {
                    if parse_bool("enabled", &p5, true) {
                        let _ = ShowWindow(h, SW_MAXIMIZE);
                        Ok("ShowWindow(SW_MAXIMIZE)")
                    } else {
                        let _ = ShowWindow(h, SW_SHOWNOACTIVATE);
                        Ok("ShowWindow(SW_SHOWNOACTIVATE)")
                    }
                }
                _ => Err("not a background action".into()),
            }
        };
        let method = match r {
            Ok(m) => m,
            Err(e) => return Outcome::Unavailable(e),
        };
        sleep_ms(250);
        let mut m = Map::new();
        m.insert("method".into(), json!(method));
        if let Some(rr) = input::restore_foreground_if_changed(prev) {
            m.insert("restored".into(), json!({"frontmost": rr}));
        }
        if act == "close" {
            // closed, or a "save changes?" dialog came up
            let start = std::time::Instant::now();
            loop {
                let alive = unsafe { IsWindow(Some(h)).as_bool() } && unsafe { IsWindowVisible(h).as_bool() };
                if !alive {
                    m.insert("closed".into(), json!(true));
                    m.insert("after".into(), Value::Null);
                    break;
                }
                if let Some(app2) = find_app(&app_name) {
                    if let Some(d) = find_dialog(&app2) {
                        m.insert("closed".into(), json!(false));
                        m.insert("dialog".into(), d.json);
                        m.insert("note".into(), json!(format!("{} asked before closing: answer it with 'aux4 computer win dialog --app {} --press <button>'", app_name, app_name)));
                        break;
                    }
                }
                if start.elapsed().as_secs_f64() > 2.0 {
                    m.insert("closed".into(), json!(false));
                    break;
                }
                sleep_ms(100);
            }
            if !m.contains_key("after") {
                m.insert("after".into(), after());
            }
        } else {
            m.insert("after".into(), after());
        }
        Outcome::Done(m)
    });
    let plan = match up {
        Some((true, r)) => Plan::InherentlyForeground(r),
        Some((false, r)) => Plan::Unavailable(r),
        None => Plan::Attempt(bg),
    };
    let act2 = action.clone();
    run_action(&name, mode, base, plan, &fg, Box::new(move || {
        let mut m = Map::new();
        if act2 == "raise" {
            m.insert("method".into(), json!("SetForegroundWindow"));
        } else {
            return Err(format!("Could not {} the window in the background", act2));
        }
        m.insert("after".into(), refresh_window(h).map(|x| win_json(&x, wi)).unwrap_or(Value::Null));
        Ok(m)
    }))
}

// ---------------------------------------------------------------------------------------------
// menus: Win32 menu bars (HMENU) are read and pressed with WM_COMMAND, fully in the background;
// apps without one (UWP/XAML/new Notepad) go through the UI Automation menu bar.

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub title: String,
    pub shortcut: Option<String>,
    pub id: u32,
    pub enabled: bool,
    pub checked: bool,
    pub submenu: Option<HMENU>,
    pub path: Vec<String>,
    pub r#ref: String,
}

pub fn clean_title(raw: &str) -> (String, Option<String>) {
    let mut parts = raw.splitn(2, '\t');
    let label = parts.next().unwrap_or("");
    let sc = parts.next().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let mut t = String::new();
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            if chars.peek() == Some(&'&') {
                t.push('&');
                chars.next();
            }
            continue;
        }
        t.push(c);
    }
    (t.trim().to_string(), sc.map(|s| s.to_lowercase()))
}

pub fn norm_title(s: &str) -> String {
    let (t, _) = clean_title(s);
    t.to_lowercase().replace(['\u{2019}', '\u{2018}'], "'").trim_end_matches('…').trim_end_matches("...").trim().to_string()
}

fn menu_items(m: HMENU, parent_path: &[String], parent_ref: &str) -> Vec<MenuItem> {
    let mut out = Vec::new();
    let n = unsafe { GetMenuItemCount(Some(m)) };
    for i in 0..n.max(0) {
        let mut buf = [0u16; 256];
        let len = unsafe { GetMenuStringW(m, i as u32, Some(&mut buf), MF_BYPOSITION) };
        if len <= 0 {
            continue; // separator
        }
        let raw = String::from_utf16_lossy(&buf[..len as usize]);
        let (title, shortcut) = clean_title(&raw);
        let state = unsafe { GetMenuState(m, i as u32, MF_BYPOSITION) };
        let sub = unsafe { GetSubMenu(m, i) };
        let id = unsafe { GetMenuItemID(m, i) };
        let mut path = parent_path.to_vec();
        path.push(title.clone());
        out.push(MenuItem {
            title,
            shortcut,
            id,
            enabled: state & (MF_GRAYED.0 | MF_DISABLED.0) == 0,
            checked: state & MF_CHECKED.0 != 0,
            submenu: if sub.0.is_null() { None } else { Some(sub) },
            path,
            r#ref: if parent_ref.is_empty() { format!("m{}", i) } else { format!("{}.{}", parent_ref, i) },
        });
    }
    out
}

/// Every item of a window's Win32 menu bar, flattened.
pub fn hmenu_flat(h: HWND) -> Vec<MenuItem> {
    let m = unsafe { GetMenu(h) };
    if m.0.is_null() {
        return vec![];
    }
    let mut out = Vec::new();
    let mut stack = vec![(m, Vec::<String>::new(), String::new())];
    while let Some((menu, path, r)) = stack.pop() {
        for it in menu_items(menu, &path, &r) {
            if let Some(s) = it.submenu {
                stack.push((s, it.path.clone(), it.r#ref.clone()));
            }
            out.push(it);
        }
    }
    out
}

fn menu_tree_json(menu: HMENU, path: &[String], r: &str, depth: usize) -> Vec<Value> {
    menu_items(menu, path, r)
        .into_iter()
        .map(|it| {
            let mut j = json!({"title": it.title, "ref": it.r#ref});
            if let Some(s) = &it.shortcut {
                j["shortcut"] = json!(s);
            }
            if !it.enabled {
                j["enabled"] = json!(false);
            }
            if it.checked {
                j["checked"] = json!(true);
            }
            if let Some(sub) = it.submenu {
                if depth > 1 {
                    j["children"] = json!(menu_tree_json(sub, &it.path, &it.r#ref, depth - 1));
                } else {
                    j["submenu"] = json!(true);
                }
            }
            j
        })
        .collect()
}

fn uia_menubar(app: &App, wi: usize) -> Option<(Vec<Node>, usize)> {
    let nodes = uia::tree(app.windows[wi].hwnd, wi, 8);
    let idx = nodes.iter().position(|n| n.ctype == 50010 && n.name.to_lowercase() != "system" && n.automation_id != "SystemMenuBar")?;
    Some((nodes, idx))
}

pub fn menu(a: &[String]) -> ! {
    let app = resolve_app(&arg(a, 0));
    let path = arg(a, 1);
    let depth: usize = arg(a, 2).trim().parse().unwrap_or(2);
    let (wi, w) = match app.windows.iter().position(|w| w.owner.is_none()) {
        Some(i) => (i, app.windows[i].clone()),
        None => fail(format!("{} has no windows", app.name)),
    };
    let hm = unsafe { GetMenu(w.hwnd) };
    if path.is_empty() {
        if !hm.0.is_null() {
            print_json(json!({"app": app.name, "depth": depth, "source": "win32", "menus": menu_tree_json(hm, &[], "", depth)}));
        }
        match uia_menubar(&app, wi) {
            Some((nodes, mb)) => {
                let prefix = format!("{}.", nodes[mb].r#ref);
                let menus: Vec<Value> = nodes.iter().filter(|n| n.r#ref.starts_with(&prefix) && n.ctype == 50011 && n.parent == Some(mb)).map(|n| json!({"title": n.name, "ref": n.r#ref, "submenu": n.expand})).collect();
                print_json(json!({"app": app.name, "depth": 1, "source": "uia", "menus": menus, "note": "This app's menus are only listed one level deep: their items exist while a menu is open"}));
            }
            None => fail(format!("{} has no menu bar available to UI Automation", app.name)),
        }
    }
    let (mut base, up, mode, name) = base_and_mode("menu", a);
    check_uipi(&app);
    base.insert("app".into(), json!(app.name));
    let parts = menu_path_parts(&path);
    let fg = fg_for(&app);
    if !hm.0.is_null() {
        // walk the HMENU by titles
        let mut cur = hm;
        let mut trail: Vec<String> = Vec::new();
        let mut item: Option<MenuItem> = None;
        for (i, part) in parts.iter().enumerate() {
            let items = menu_items(cur, &trail, "");
            let want = norm_title(part);
            let found = items.iter().find(|it| norm_title(&it.title) == want).cloned();
            let it = match found {
                Some(it) => it,
                None => {
                    let place = if trail.is_empty() { "in the menu bar".to_string() } else { format!("under '{}'", trail.join(" > ")) };
                    let available: Vec<String> = items.iter().map(|x| x.title.clone()).collect();
                    fail(format!("No menu item '{}' {}. Available: {}", part, place, available.join(", ")));
                }
            };
            trail.push(it.title.clone());
            if i + 1 < parts.len() {
                match it.submenu {
                    Some(s) => cur = s,
                    None => fail(format!("'{}' has no submenu", it.title)),
                }
            }
            item = Some(it);
        }
        let it = item.unwrap();
        if it.submenu.is_some() {
            fail(format!("Invalid --path '{}': '{}' is a menu, give one of its items", path, it.title));
        }
        base.insert("pressed".into(), json!(it.title));
        base.insert("ref".into(), json!(format!("menu:{}", it.id)));
        let h = w.hwnd;
        let id = it.id;
        let enabled = it.enabled;
        let bg = Box::new(move || -> Outcome {
            if !enabled {
                return Outcome::Unavailable("the menu item is disabled".into());
            }
            match unsafe { PostMessageW(Some(h), WM_COMMAND, WPARAM(id as usize), LPARAM(0)) } {
                Ok(()) => done(vec![("method", json!("WM_COMMAND"))]),
                Err(e) => Outcome::Unavailable(e.message()),
            }
        });
        let plan = match up {
            Some((false, r)) => Plan::Unavailable(r),
            _ => Plan::Attempt(bg),
        };
        run_action(&name, mode, base, plan, &fg, Box::new(move || {
            unsafe { PostMessageW(Some(h), WM_COMMAND, WPARAM(id as usize), LPARAM(0)) }.map_err(|e| format!("Could not press the menu item: {}", e.message()))?;
            Ok(json!({"method": "WM_COMMAND"}).as_object().unwrap().clone())
        }))
    }
    // UI Automation menu bar: expand each level, invoke the last item
    let app2 = app.clone();
    let parts2 = parts.clone();
    let press = move || -> Result<String, String> {
        let (nodes, mb) = uia_menubar(&app2, wi).ok_or_else(|| format!("{} has no menu bar available to UI Automation", app2.name))?;
        let top: Vec<&Node> = nodes.iter().filter(|n| n.parent == Some(mb) && n.ctype == 50011).collect();
        let want = norm_title(&parts2[0]);
        let first = top.iter().find(|n| norm_title(&n.name) == want).ok_or_else(|| format!("No menu item '{}' in the menu bar. Available: {}", parts2[0], top.iter().map(|n| n.name.clone()).collect::<Vec<_>>().join(", ")))?;
        let mut current: Node = (*first).clone();
        for part in parts2.iter().skip(1) {
            uia::expand(&current).or_else(|_| uia::invoke(&current))?;
            sleep_ms(300);
            // the opened menu: search the app's windows (popups are top-level) and the desktop
            let want = norm_title(part);
            let mut next: Option<Node> = None;
            let mut seen: Vec<String> = Vec::new();
            for _ in 0..10 {
                let a3 = find_app(&app2.pid.to_string()).unwrap_or(app2.clone());
                for (i, ww) in a3.windows.iter().enumerate() {
                    for n in uia::tree(ww.hwnd, i, 12) {
                        if n.ctype == 50011 {
                            if norm_title(&n.name) == want {
                                next = Some(n.clone());
                            }
                            seen.push(n.name.clone());
                        }
                    }
                }
                if next.is_some() {
                    break;
                }
                sleep_ms(100);
            }
            current = next.ok_or_else(|| format!("No menu item '{}'. Available: {}", part, seen.join(", ")))?;
        }
        uia::invoke(&current).or_else(|_| uia::legacy_default_action(&current))?;
        Ok(current.name)
    };
    base.insert("pressed".into(), json!(parts.last().cloned().unwrap_or_default()));
    let press = std::rc::Rc::new(press);
    let p1 = press.clone();
    let bg = Box::new(move || -> Outcome {
        match p1() {
            Ok(n) => done(vec![("method", json!("uia")), ("pressed", json!(n))]),
            Err(e) => Outcome::Unavailable(e),
        }
    });
    run_action(&name, mode, base, Plan::Attempt(bg), &fg, Box::new(move || {
        let n = press()?;
        Ok(json!({"method": "uia", "pressed": n}).as_object().unwrap().clone())
    }))
}

// ---------------------------------------------------------------------------------------------
// dialogs

pub struct Dialog {
    pub json: Value,
    pub hwnd: HWND,
    pub buttons: Vec<Node>,
}

pub fn find_dialog(app: &App) -> Option<Dialog> {
    let (wi, w) = app.windows.iter().enumerate().find(|(_, w)| w.class == "#32770" || (w.owner.is_some() && !w.minimized)).map(|(i, w)| (i, w.clone()))?;
    let nodes = uia::tree(w.hwnd, wi, 16);
    let buttons: Vec<Node> = nodes.iter().filter(|n| matches!(n.ctype, 50000 | 50031) && !n.name.is_empty() && !n.offscreen).cloned().collect();
    let fields: Vec<&Node> = nodes.iter().filter(|n| matches!(n.ctype, 50004 | 50003)).collect();
    let texts: Vec<String> = nodes.iter().filter(|n| n.ctype == 50020 && !n.name.is_empty()).map(|n| n.name.clone()).collect();
    let title = w.title.clone();
    let tl = title.to_lowercase();
    let kind = if !fields.is_empty() && (tl.contains("save") || tl.contains("open")) {
        if tl.contains("open") { "open-panel" } else { "save-panel" }
    } else if fields.is_empty() {
        "alert"
    } else {
        "dialog"
    };
    let bjson: Vec<Value> = buttons
        .iter()
        .map(|b| {
            let mut j = uia::node_json(b, Some(&w.frame));
            let t = clean_title(&b.name).0;
            j["title"] = json!(t);
            let tl = t.to_lowercase();
            j["cancel"] = json!(tl == "cancel");
            j["default"] = json!(b.focused);
            j
        })
        .collect();
    let fjson: Vec<Value> = fields.iter().map(|f| uia::node_json(f, Some(&w.frame))).collect();
    let json = json!({"app": app.name, "kind": kind, "title": title, "text": texts.join("\n"), "buttons": bjson, "fields": fjson, "ref": wi.to_string(), "frame": frame_json(&w.frame)});
    Some(Dialog { json, hwnd: w.hwnd, buttons })
}

pub fn dialog(a: &[String]) -> ! {
    let app_q = arg(a, 0);
    let press = arg(a, 1);
    let app = resolve_app(&app_q);
    let d = find_dialog(&app);
    if press.is_empty() {
        print_json(json!({"app": app.name, "dialog": d.map(|d| d.json).unwrap_or(Value::Null)}));
    }
    let (mut base, up, mode, name) = base_and_mode("dialog", a);
    check_uipi(&app);
    base.insert("app".into(), json!(app.name));
    let d = match d {
        Some(d) => d,
        None => fail(format!("No dialog, sheet or alert open in {}", app.name)),
    };
    let want = norm_title(&press);
    let b = match d.buttons.iter().find(|b| norm_title(&b.name) == want).or_else(|| d.buttons.iter().find(|b| norm_title(&b.name).starts_with(&want))) {
        Some(b) => b.clone(),
        None => {
            let titles: Vec<String> = d.buttons.iter().map(|b| clean_title(&b.name).0).collect();
            let kind = d.json["kind"].as_str().unwrap_or("dialog").to_string();
            fail(format!("No button '{}' in the {}. Buttons: {}", press, kind, titles.join(", ")));
        }
    };
    base.insert("kind".into(), d.json["kind"].clone());
    base.insert("dialog".into(), json!({"kind": d.json["kind"], "title": d.json["title"]}));
    let h = d.hwnd;
    let fg = Fg { target: Some(h), name: app.name.clone() };
    let b2 = b.clone();
    let wait_closed = move || -> bool {
        let start = std::time::Instant::now();
        while start.elapsed().as_secs_f64() < 2.0 {
            if !unsafe { IsWindow(Some(h)).as_bool() } || !unsafe { IsWindowVisible(h).as_bool() } {
                return true;
            }
            sleep_ms(100);
        }
        false
    };
    let wc = std::rc::Rc::new(wait_closed);
    let wc2 = wc.clone();
    let bg = Box::new(move || -> Outcome {
        let r = uia::invoke(&b2).map(|_| "Invoke").or_else(|_| uia::legacy_default_action(&b2).map(|_| "DoDefaultAction"));
        match r {
            Ok(m) => done(vec![("method", json!(m)), ("pressed", json!(clean_title(&b2.name).0)), ("closed", json!(wc2()))]),
            Err(e) => Outcome::Unavailable(format!("the button can't be pressed in the background: {}", e)),
        }
    });
    let plan = match up {
        Some((false, r)) => Plan::Unavailable(r),
        _ => Plan::Attempt(bg),
    };
    let rect = b.rect;
    let bname = clean_title(&b.name).0;
    run_action(&name, mode, base, plan, &fg, Box::new(move || {
        input::click_physical((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2, false, 1)?;
        Ok(json!({"method": "SendInput", "pressed": bname, "closed": wc()}).as_object().unwrap().clone())
    }))
}
