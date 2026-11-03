// Input actions: click, move, drag, scroll, type, shortcut, clipboard copy/paste, set, select.
// Each tries the background (UI Automation patterns, or messages to the app's own controls) and,
// in auto mode, falls back to real input with the cursor and foreground window restored.

use super::cmds::*;
use super::input::{self, Fg};
use super::uia::{self, Node};
use super::*;
use crate::{action_base, action_mode_index, action_name};
use serde_json::Map;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;

fn base_and_mode(command: &str, a: &[String]) -> (Map<String, Value>, Option<(bool, String)>, Mode, String) {
    let (base, plan) = action_base(command, a);
    let mode = action_mode_index(command).map(|i| parse_mode(&arg(a, i))).unwrap_or(Mode::Auto);
    (base, plan, mode, action_name(command, a))
}

fn plan_of<'a>(up: Option<(bool, String)>, bg: Box<dyn FnOnce() -> Outcome + 'a>) -> Plan<'a> {
    match up {
        Some((true, r)) => Plan::InherentlyForeground(r),
        Some((false, r)) => Plan::Unavailable(r),
        None => Plan::Attempt(bg),
    }
}

fn done(pairs: Vec<(&str, Value)>) -> Outcome {
    let mut m = Map::new();
    for (k, v) in pairs {
        m.insert(k.to_string(), v);
    }
    Outcome::Done(m)
}

fn fg_for(app: Option<&App>) -> Fg {
    match app {
        Some(a) => Fg { target: a.main_window().map(|w| w.hwnd), name: a.name.clone() },
        None => Fg { target: None, name: String::new() },
    }
}

fn opt_app(q: &str) -> Option<App> {
    if q.trim().is_empty() {
        None
    } else {
        let a = resolve_app(q);
        check_uipi(&a);
        Some(a)
    }
}

/// Post a click to the native window under the point (client coordinates).
fn post_click(h: HWND, p: POINT, right: bool) -> bool {
    let mut c = p;
    unsafe {
        let _ = ScreenToClient(h, &mut c);
    }
    let lp = ((c.y as u32 & 0xffff) << 16 | (c.x as u32 & 0xffff)) as isize;
    let (down, up, mk) = if right { (0x0204u32, 0x0205u32, 0x0002usize) } else { (0x0201u32, 0x0202u32, 0x0001usize) };
    unsafe { PostMessageW(Some(h), down, WPARAM(mk), LPARAM(lp)).is_ok() && PostMessageW(Some(h), up, WPARAM(0), LPARAM(lp)).is_ok() }
}

/// Press an element in the background through its UI Automation patterns.
fn press_node(n: &Node, nodes: Option<&[Node]>, idx: usize, p: Option<POINT>, right: bool) -> Result<&'static str, String> {
    // A Win32 push button: tell its parent it was clicked. UI Automation's Invoke on these goes
    // through BM_CLICK, which can activate the window.
    if !right && !n.hwnd.0.is_null() && n.ctype == 50000 && class_name(n.hwnd).eq_ignore_ascii_case("button") {
        let style = unsafe { GetWindowLongW(n.hwnd, GWL_STYLE) } as u32 & 0xF;
        if matches!(style, 0x0 | 0x1 | 0xB) {
            unsafe {
                let id = GetDlgCtrlID(n.hwnd) as u32;
                if let Ok(parent) = GetParent(n.hwnd) {
                    if !parent.0.is_null() && PostMessageW(Some(parent), WM_COMMAND, WPARAM((id & 0xffff) as usize), LPARAM(n.hwnd.0 as isize)).is_ok() {
                        return Ok("BN_CLICKED");
                    }
                }
            }
        }
    }
    if !right {
        if n.invoke && uia::invoke(n).is_ok() {
            return Ok("Invoke");
        }
        if n.toggle && uia::toggle(n).is_ok() {
            return Ok("Toggle");
        }
        if n.selection_item && uia::select_item(n).is_ok() {
            return Ok("SelectionItem");
        }
        if n.expand {
            let r = if n.expand_state == 1 { uia::collapse(n) } else { uia::expand(n) };
            if r.is_ok() {
                return Ok("ExpandCollapse");
            }
        }
        if !n.legacy_action.is_empty() && !matches!(n.ctype, 50004 | 50030) && uia::legacy_default_action(n).is_ok() {
            return Ok("DoDefaultAction");
        }
    }
    // A native control: post the click to it (text fields get the caret there).
    let h = if !n.hwnd.0.is_null() { Some(n.hwnd) } else { nodes.and_then(|ns| uia::nearest_hwnd(ns, idx)) };
    if let Some(h) = h {
        let pt = p.unwrap_or(POINT { x: (n.rect.left + n.rect.right) / 2, y: (n.rect.top + n.rect.bottom) / 2 });
        if post_click(h, pt, right) {
            return Ok("PostMessage");
        }
    }
    Err(format!("'{}' ({}) has no UI Automation action and no window to post the click to", n.name, n.role))
}

// ---------------------------------------------------------------------------------------------

pub fn click(a: &[String]) -> ! {
    let (mut base, up, mode, name) = base_and_mode("click", a);
    let right = arg(a, 2).to_lowercase() == "right";
    let count = arg(a, 3).trim().parse::<f64>().unwrap_or(1.0) as u32;
    let element = arg(a, 4);
    let app_q = arg(a, 5);
    if !element.is_empty() {
        let app = resolve_app(&app_q);
        check_uipi(&app);
        let n = uia::resolve_ref(&app, &element);
        base.insert("app".into(), json!(app.name));
        base.insert("element".into(), uia::short_json(&n));
        if n.rect.right > n.rect.left {
            base.insert("screen".into(), center_json(&n.rect));
        }
        let rect = n.rect;
        let fg = fg_for(Some(&app));
        let n2 = n.clone();
        run_action(&name, mode, base, plan_of(up, Box::new(move || match press_node(&n2, None, 0, None, right) {
            Ok(m) => done(vec![("action", json!(m))]),
            Err(e) => Outcome::Unavailable(e),
        })), &fg, Box::new(move || {
            if rect.right <= rect.left {
                return Err(format!("Element '{}' has no on-screen frame to click", element));
            }
            input::click_physical((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2, right, count.max(1))?;
            Ok(Map::new())
        }))
    }
    let (x, y) = (parse_number("x", &arg(a, 0)).unwrap_or(0.0), parse_number("y", &arg(a, 1)).unwrap_or(0.0));
    let app = opt_app(&app_q);
    // the physical point
    let (p, win_index) = match &app {
        Some(app) => {
            let (wi, w) = pick_window(app, "", "");
            let s = rect_scale(&w.frame);
            (POINT { x: w.frame.left + (x * s).round() as i32, y: w.frame.top + (y * s).round() as i32 }, Some(wi))
        }
        None => {
            let (px, py) = to_physical(x, y);
            (POINT { x: px, y: py }, None)
        }
    };
    let (lx, ly) = to_logical(p.x as f64, p.y as f64);
    base.insert("screen".into(), json!({"x": num(lx), "y": num(ly)}));
    // without --app the target is the app whose window is at the point
    let target: Option<(App, usize)> = match (&app, win_index) {
        (Some(a), Some(wi)) => Some((a.clone(), wi)),
        _ => {
            let h = unsafe { GetAncestor(WindowFromPoint(p), GA_ROOT) };
            apps_list().into_iter().find_map(|a| a.windows.iter().position(|w| w.hwnd == h).map(|i| (a.clone(), i)))
        }
    };
    if let Some((t, _)) = &target {
        check_uipi(t);
    }
    let fg = fg_for(app.as_ref());
    let bg = Box::new(move || -> Outcome {
        let (t, wi) = match &target {
            Some(t) => t.clone(),
            None => return Outcome::Unavailable("no app window at that point".into()),
        };
        let w = &t.windows[wi];
        let nodes = uia::tree(w.hwnd, wi, 64);
        let mut best: Option<usize> = None;
        for (i, n) in nodes.iter().enumerate() {
            let r = n.rect;
            if !n.offscreen && p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom && best.map(|b| n.depth >= nodes[b].depth).unwrap_or(true) {
                best = Some(i);
            }
        }
        let idx = match best {
            Some(i) => i,
            None => return Outcome::Unavailable("nothing pressable at that point".into()),
        };
        // climb to the nearest actionable ancestor (a button's text child -> the button)
        let mut k = idx;
        loop {
            let n = &nodes[k];
            if n.invoke || n.toggle || n.selection_item || n.expand || !n.hwnd.0.is_null() || matches!(n.ctype, 50004 | 50030) {
                break;
            }
            match n.parent {
                Some(p) if p != 0 => k = p,
                _ => break,
            }
        }
        let n = &nodes[k];
        match press_node(n, Some(&nodes), k, Some(p), right) {
            Ok(m) => done(vec![("action", json!(m)), ("app", json!(t.name)), ("element", uia::short_json(n))]),
            Err(e) => Outcome::Unavailable(e),
        }
    });
    run_action(&name, mode, base, plan_of(up, bg), &fg, Box::new(move || {
        input::click_physical(p.x, p.y, right, count.max(1))?;
        Ok(Map::new())
    }))
}

pub fn mouse_move(a: &[String]) -> ! {
    let (base, up, mode, name) = base_and_mode("move", a);
    let (px, py) = to_physical(parse_number("x", &arg(a, 0)).unwrap_or(0.0), parse_number("y", &arg(a, 1)).unwrap_or(0.0));
    run_action(&name, mode, base, plan_of(up, Box::new(|| Outcome::Unavailable(String::new()))), &Fg { target: None, name: String::new() }, Box::new(move || {
        input::set_cursor_physical(px, py)?;
        Ok(Map::new())
    }))
}

pub fn drag(a: &[String]) -> ! {
    let (base, up, mode, name) = base_and_mode("drag", a);
    let n = |i: usize| parse_number("x", &arg(a, i)).unwrap_or(0.0);
    let (x, y) = to_physical(n(0), n(1));
    let (tx, ty) = to_physical(n(2), n(3));
    run_action(&name, mode, base, plan_of(up, Box::new(|| Outcome::Unavailable(String::new()))), &Fg { target: None, name: String::new() }, Box::new(move || {
        input::drag_physical(x, y, tx, ty)?;
        Ok(Map::new())
    }))
}

pub fn scroll(a: &[String]) -> ! {
    let (base, up, mode, name) = base_and_mode("scroll", a);
    let dx = parse_number("dx", &arg(a, 2)).unwrap_or(0.0);
    let dy = parse_number("dy", &arg(a, 3)).unwrap_or(0.0);
    let app = opt_app(&arg(a, 4));
    let has_xy = !arg(a, 0).is_empty() && !arg(a, 1).is_empty();
    let (x, y) = (parse_number("x", &arg(a, 0)).unwrap_or(0.0), parse_number("y", &arg(a, 1)).unwrap_or(0.0));
    let target = app.clone().or_else(foreground_app);
    let point: Option<(POINT, usize)> = target.as_ref().and_then(|t| {
        let (wi, w) = (0..t.windows.len()).find(|i| !t.windows[*i].minimized).map(|i| (i, t.windows[i].clone()))?;
        let s = rect_scale(&w.frame);
        let p = if has_xy && app.is_some() {
            POINT { x: w.frame.left + (x * s).round() as i32, y: w.frame.top + (y * s).round() as i32 }
        } else if has_xy {
            let (px, py) = to_physical(x, y);
            POINT { x: px, y: py }
        } else {
            POINT { x: (w.frame.left + w.frame.right) / 2, y: (w.frame.top + w.frame.bottom) / 2 }
        };
        Some((p, wi))
    });
    let mut base = base;
    if let Some((p, _)) = &point {
        let (lx, ly) = to_logical(p.x as f64, p.y as f64);
        base.insert("screen".into(), json!({"x": num(lx), "y": num(ly)}));
    }
    let fg = fg_for(app.as_ref());
    let t2 = target.clone();
    let bg = Box::new(move || -> Outcome {
        let (t, (p, wi)) = match (&t2, point) {
            (Some(t), Some(pw)) => (t.clone(), pw),
            _ => return Outcome::Unavailable("no app window to scroll".into()),
        };
        let nodes = uia::tree(t.windows[wi].hwnd, wi, 64);
        // deepest element at the point, then the nearest ancestor with a scroll pattern
        let mut best: Option<usize> = None;
        for (i, n) in nodes.iter().enumerate() {
            let r = n.rect;
            if p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom && best.map(|b| n.depth >= nodes[b].depth).unwrap_or(true) {
                best = Some(i);
            }
        }
        let mut k = best;
        while let Some(i) = k {
            if nodes[i].scroll {
                if let Some((h0, v0, hs, vs)) = uia::scroll_percent(&nodes[i].el) {
                    // one small increment ~ 40 logical px
                    let steps_y = (dy.abs() / 40.0).round().max(if dy != 0.0 { 1.0 } else { 0.0 }) as i32;
                    let steps_x = (dx.abs() / 40.0).round().max(if dx != 0.0 { 1.0 } else { 0.0 }) as i32;
                    let p: Option<windows::Win32::UI::Accessibility::IUIAutomationScrollPattern> = unsafe { nodes[i].el.GetCurrentPatternAs(windows::Win32::UI::Accessibility::UIA_ScrollPatternId).ok() };
                    if let Some(p) = p {
                        use windows::Win32::UI::Accessibility::*;
                        for _ in 0..steps_y {
                            if !vs {
                                break;
                            }
                            let amt = if dy > 0.0 { ScrollAmount_SmallIncrement } else { ScrollAmount_SmallDecrement };
                            let _ = unsafe { p.Scroll(ScrollAmount_NoAmount, amt) };
                        }
                        for _ in 0..steps_x {
                            if !hs {
                                break;
                            }
                            let amt = if dx > 0.0 { ScrollAmount_SmallIncrement } else { ScrollAmount_SmallDecrement };
                            let _ = unsafe { p.Scroll(amt, ScrollAmount_NoAmount) };
                        }
                    }
                    let (h1, v1, _, _) = uia::scroll_percent(&nodes[i].el).unwrap_or((h0, v0, hs, vs));
                    return done(vec![("method", json!("ScrollPattern")), ("app", json!(t.name)), ("vertical", num(v1 / 100.0)), ("horizontal", num(h1 / 100.0))]);
                }
            }
            k = nodes[i].parent;
        }
        // no scroll pattern: post the wheel to the native window under the point
        if let Some(i) = best {
            if let Some(h) = uia::nearest_hwnd(&nodes, i) {
                let lp = ((p.y as u32 & 0xffff) << 16 | (p.x as u32 & 0xffff)) as isize;
                let mut ok = true;
                if dy != 0.0 {
                    let delta = ((-dy * 3.0).round() as i32 as i16 as u16 as usize) << 16;
                    ok &= unsafe { PostMessageW(Some(h), 0x020A, WPARAM(delta), LPARAM(lp)).is_ok() };
                }
                if dx != 0.0 {
                    let delta = (((dx * 3.0).round() as i32 as i16 as u16) as usize) << 16;
                    ok &= unsafe { PostMessageW(Some(h), 0x020E, WPARAM(delta), LPARAM(lp)).is_ok() };
                }
                if ok {
                    return done(vec![("method", json!("WM_MOUSEWHEEL")), ("app", json!(t.name))]);
                }
            }
        }
        Outcome::Unavailable("no scroll area at the point".into())
    });
    run_action(&name, mode, base, plan_of(up, bg), &fg, Box::new(move || {
        if let Some((p, _)) = point {
            input::set_cursor_physical(p.x, p.y)?;
            sleep_ms(30);
        }
        input::wheel(dx, dy)?;
        Ok(Map::new())
    }))
}

/// Insert text at the app's caret in the background.
fn insert_text_bg(app: &App, text: &str) -> Outcome {
    let (_, w, n) = match app_text_target(app) {
        Some(t) => t,
        None => return Outcome::Unavailable(format!("{} has no focused text field", app.name)),
    };
    let h = if !n.hwnd.0.is_null() { Some(n.hwnd) } else { app_focus_hwnd(&w) };
    if let Some(h) = h {
        let class = class_name(h);
        if is_edit_class(&class) {
            let before = get_text(h);
            let normalized = text.replace("\r\n", "\n").replace('\n', "\r\n");
            if replace_sel(h, &normalized) {
                let after = get_text(h);
                if after != before || text.is_empty() {
                    return done(vec![("method", json!("EM_REPLACESEL")), ("element", uia::short_json(&n))]);
                }
            }
            return Outcome::Unavailable("the text field did not accept the text in the background".into());
        }
        // other native controls: post the characters
        let mut ok = true;
        for u in text.encode_utf16() {
            ok &= unsafe { PostMessageW(Some(h), WM_CHAR, WPARAM(u as usize), LPARAM(1)).is_ok() };
        }
        if ok && class_name(h) != "ApplicationFrameWindow" && !n.hwnd.0.is_null() {
            return done(vec![("method", json!("WM_CHAR")), ("element", uia::short_json(&n))]);
        }
    }
    Outcome::Unavailable("the focused element doesn't accept text in the background".into())
}

pub fn type_text(a: &[String]) -> ! {
    let (base, up, mode, name) = base_and_mode("type", a);
    let text = arg(a, 0);
    let delay: u64 = arg(a, 1).trim().parse().unwrap_or(0);
    if delay > 0 {
        sleep_ms(delay);
    }
    let app = opt_app(&arg(a, 2));
    let fg = fg_for(app.as_ref());
    let (t1, t2) = (text.clone(), text.clone());
    let app2 = app.clone();
    let bg = Box::new(move || match &app2 {
        Some(app) => insert_text_bg(app, &t1),
        None => match foreground_app() {
            Some(f) => insert_text_bg(&f, &t1),
            None => Outcome::Unavailable("no app to type into".into()),
        },
    });
    run_action(&name, mode, base, plan_of(up, bg), &fg, Box::new(move || {
        input::type_unicode(&t2)?;
        Ok(json!({"method": "SendInput"}).as_object().unwrap().clone())
    }))
}

/// A Win32 menu item whose label shows this shortcut (e.g. "&New\tCtrl+N").
fn menu_item_for_combo(h: HWND, c: &Combo) -> Option<(u32, String)> {
    let items = super::appwin::hmenu_flat(h);
    let want = combo_display(c);
    for it in items {
        if let Some(sc) = &it.shortcut {
            if let Ok(parsed) = parse_combo(&sc.replace(' ', "")) {
                if combo_display(&parsed) == want && it.enabled {
                    return Some((it.id, it.title.clone()));
                }
            }
        }
    }
    None
}

pub fn shortcut(a: &[String]) -> ! {
    let (base, up, mode, name) = base_and_mode("shortcut", a);
    let combo = parse_combo(&arg(a, 0)).unwrap_or_else(|e| fail(e));
    let delay: u64 = arg(a, 1).trim().parse().unwrap_or(0);
    if delay > 0 {
        sleep_ms(delay);
    }
    let app = opt_app(&arg(a, 2));
    let fg = fg_for(app.as_ref());
    let app2 = app.clone();
    let c2 = combo.clone();
    let bg = Box::new(move || -> Outcome {
        let app = match &app2 {
            Some(a) => a,
            None => return Outcome::Unavailable("no app".into()),
        };
        let main = match app.main_window() {
            Some(w) => w.clone(),
            None => return Outcome::Unavailable(format!("{} has no windows", app.name)),
        };
        // 1. the menu item for that shortcut
        if let Some((id, title)) = menu_item_for_combo(main.hwnd, &c2) {
            if unsafe { PostMessageW(Some(main.hwnd), WM_COMMAND, WPARAM(id as usize), LPARAM(0)).is_ok() } {
                return done(vec![("method", json!("menu")), ("menuItem", json!(title))]);
            }
        }
        // 2. editing shortcuts on the app's text control
        let plain_ctrl = c2.ctrl && !c2.alt && !c2.shift && !c2.win;
        if plain_ctrl && ["a", "c", "v", "x", "z"].contains(&c2.key.as_str()) {
            if let Some((_, w, n)) = app_text_target(app) {
                let h = if !n.hwnd.0.is_null() { Some(n.hwnd) } else { app_focus_hwnd(&w) };
                if let Some(h) = h.filter(|h| is_edit_class(&class_name(*h))) {
                    let (msg, method) = match c2.key.as_str() {
                        "a" => (EM_SETSEL, "EM_SETSEL"),
                        "c" => (WM_COPY, "WM_COPY"),
                        "v" => (WM_PASTE, "WM_PASTE"),
                        "x" => (WM_CUT, "WM_CUT"),
                        _ => (WM_UNDO, "WM_UNDO"),
                    };
                    let ok = if msg == EM_SETSEL { set_sel(h, 0, -1) } else { send_msg(h, msg, 0, 0).is_some() };
                    if ok {
                        return done(vec![("method", json!(method)), ("element", uia::short_json(&n))]);
                    }
                }
            }
        }
        // 3. the app is already in front: real keys don't change anything for the user
        let fgw = input::foreground_window();
        let fg_root = unsafe { GetAncestor(fgw, GA_ROOTOWNER) };
        if app.windows.iter().any(|w| w.hwnd == fg_root || w.hwnd == fgw) {
            return match input::press_combo(&c2) {
                Ok(()) => done(vec![("method", json!("SendInput"))]),
                Err(e) => Outcome::Unavailable(e),
            };
        }
        Outcome::Unavailable(format!("no menu item or background equivalent for {} in {}", combo_display(&c2), app.name))
    });
    run_action(&name, mode, base, plan_of(up, bg), &fg, Box::new(move || {
        input::press_combo(&combo)?;
        Ok(Map::new())
    }))
}

pub fn clipboard_copy(a: &[String]) -> ! {
    let text = arg(a, 0);
    if !text.is_empty() {
        if dry_run() {
            print_json(json!({"text": text, "dryRun": true}));
        }
        if let Err(e) = super::clip::write(&text) {
            fail(e);
        }
        print_json(json!({"text": text}));
    }
    let (base, up, mode, name) = base_and_mode("clipboard-copy", a);
    let app = opt_app(&arg(a, 1));
    let target = app.clone().or_else(foreground_app);
    let fg = fg_for(app.as_ref());
    let t2 = target.clone();
    let bg = Box::new(move || -> Outcome {
        let t = match &t2 {
            Some(t) => t,
            None => return Outcome::Unavailable("no app".into()),
        };
        let (_, w, n) = match app_text_target(t) {
            Some(x) => x,
            None => return Outcome::Unavailable(format!("{} has no text selection the background can read", t.name)),
        };
        let h = if !n.hwnd.0.is_null() { Some(n.hwnd) } else { app_focus_hwnd(&w) };
        let selected: Option<(String, &str)> = if let Some(h) = h.filter(|h| is_edit_class(&class_name(*h))) {
            let all = get_text(h);
            let (s, e) = get_sel(h);
            let u: Vec<u16> = all.encode_utf16().collect();
            let (s, e) = (s.min(u.len()), e.min(u.len()));
            Some((String::from_utf16_lossy(&u[s.min(e)..e.max(s)]), "EM_GETSEL"))
        } else {
            uia::selected_text(&n.el).map(|s| (s, "TextPattern"))
        };
        match selected {
            Some((s, method)) => {
                if let Err(e) = super::clip::write(&s) {
                    return Outcome::Unavailable(e);
                }
                done(vec![("method", json!(method)), ("text", json!(s)), ("app", json!(t.name))])
            }
            None => Outcome::Unavailable("the app doesn't expose its selection".into()),
        }
    });
    run_action(&name, mode, base, plan_of(up, bg), &fg, Box::new(move || {
        input::press_combo(&parse_combo("ctrl+c").unwrap())?;
        sleep_ms(200);
        let t = super::clip::read()?;
        Ok(json!({"text": t, "method": "SendInput"}).as_object().unwrap().clone())
    }))
}

pub fn clipboard_paste(a: &[String]) -> ! {
    let (mut base, up, mode, name) = base_and_mode("clipboard-paste", a);
    let text = arg(a, 0);
    if !text.is_empty() {
        if let Err(e) = super::clip::write(&text) {
            fail(e);
        }
    }
    let content = super::clip::read().unwrap_or_default();
    base.insert("pasted".into(), json!(content));
    let app = opt_app(&arg(a, 1));
    let target = app.clone().or_else(foreground_app);
    let fg = fg_for(app.as_ref());
    let c2 = content.clone();
    let bg = Box::new(move || -> Outcome {
        let t = match &target {
            Some(t) => t,
            None => return Outcome::Unavailable("no app".into()),
        };
        if let Some((_, w, n)) = app_text_target(t) {
            let h = if !n.hwnd.0.is_null() { Some(n.hwnd) } else { app_focus_hwnd(&w) };
            if let Some(h) = h.filter(|h| is_edit_class(&class_name(*h))) {
                let before = get_text(h);
                if send_msg(h, WM_PASTE, 0, 0).is_some() && (get_text(h) != before || c2.is_empty()) {
                    return done(vec![("method", json!("WM_PASTE")), ("element", uia::short_json(&n))]);
                }
            }
        }
        insert_text_bg(t, &c2)
    });
    run_action(&name, mode, base, plan_of(up, bg), &fg, Box::new(move || {
        input::press_combo(&parse_combo("ctrl+v").unwrap())?;
        Ok(json!({"method": "SendInput"}).as_object().unwrap().clone())
    }))
}

fn element_value(n: &Node) -> Value {
    if !n.hwnd.0.is_null() && is_edit_class(&n.class) {
        return json!(get_text(n.hwnd));
    }
    if let Some(v) = uia::current_value(&n.el) {
        return json!(v);
    }
    if let Some(v) = uia::range_value(&n.el) {
        return num(v);
    }
    if let Some(s) = uia::toggle_state(n) {
        return json!(s == 1);
    }
    json!(n.name)
}

pub fn set(a: &[String]) -> ! {
    let (mut base, up, mode, name) = base_and_mode("set", a);
    let app = resolve_app(&arg(a, 0));
    check_uipi(&app);
    let element = arg(a, 1);
    let (value, checked, option) = (arg(a, 2), arg(a, 3), arg(a, 4));
    let n = uia::resolve_ref(&app, &element);
    base.insert("app".into(), json!(app.name));
    base.insert("element".into(), uia::short_json(&n));
    base.insert("before".into(), element_value(&n));
    let fg = fg_for(Some(&app));
    let n2 = n.clone();
    let (v2, c2, o2) = (value.clone(), checked.clone(), option.clone());
    let bg = Box::new(move || -> Outcome {
        let n = n2;
        if !v2.is_empty() {
            if !n.hwnd.0.is_null() && is_edit_class(&n.class) {
                let normalized = v2.replace("\r\n", "\n").replace('\n', "\r\n");
                if set_sel(n.hwnd, 0, -1) && replace_sel(n.hwnd, &normalized) {
                    return done(vec![("method", json!("EM_REPLACESEL")), ("after", element_value(&n))]);
                }
            }
            if n.has_value && !n.value_readonly && uia::set_value(&n.el, &v2).is_ok() {
                return done(vec![("method", json!("Value")), ("after", element_value(&n))]);
            }
            if n.range {
                if let Ok(f) = v2.trim().parse::<f64>() {
                    if uia::set_range_value(&n.el, f).is_ok() {
                        return done(vec![("method", json!("RangeValue")), ("after", element_value(&n))]);
                    }
                }
            }
            return Outcome::Unavailable(format!("'{}' ({}) has no settable value", n.name, n.role));
        }
        if !c2.is_empty() {
            let want = c2.to_lowercase() == "true";
            if n.toggle {
                for _ in 0..3 {
                    if uia::toggle_state(&n).map(|s| s == 1) == Some(want) {
                        return done(vec![("method", json!("Toggle")), ("after", json!(want))]);
                    }
                    if uia::toggle(&n).is_err() {
                        break;
                    }
                }
                if uia::toggle_state(&n).map(|s| s == 1) == Some(want) {
                    return done(vec![("method", json!("Toggle")), ("after", json!(want))]);
                }
                return Outcome::Unavailable("the checkbox did not change state in the background".into());
            }
            if n.selection_item && want {
                if uia::select_item(&n).is_ok() {
                    return done(vec![("method", json!("SelectionItem")), ("after", json!(true))]);
                }
            }
            return Outcome::Unavailable(format!("'{}' ({}) is not a checkbox or radio button", n.name, n.role));
        }
        // --option: a combo box / list / radio group child by name
        let pick = |nodes: &[Node]| -> Option<Node> {
            let o = o2.to_lowercase();
            nodes.iter().skip(1).find(|c| c.name.to_lowercase() == o && (c.selection_item || c.invoke)).cloned().or_else(|| nodes.iter().skip(1).find(|c| c.name.to_lowercase().contains(&o) && (c.selection_item || c.invoke)).cloned())
        };
        let subtree = |n: &Node| -> Vec<Node> {
            let uia_ = uia::automation();
            let _ = uia_;
            let wi: usize = n.r#ref.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0);
            let app = find_app(&arg(a, 0)).unwrap_or_else(|| fail(format!("App not running: {}", arg(a, 0))));
            let all = uia::tree(app.windows[wi.min(app.windows.len() - 1)].hwnd, wi, 64);
            let prefix = format!("{}.", n.r#ref);
            let mut v = vec![n.clone()];
            v.extend(all.into_iter().filter(|x| x.r#ref.starts_with(&prefix)));
            v
        };
        let mut nodes = subtree(&n);
        let mut opened = false;
        if pick(&nodes).is_none() && n.expand {
            let _ = uia::expand(&n);
            opened = true;
            sleep_ms(250);
            nodes = subtree(&n);
        }
        let r = match pick(&nodes) {
            Some(c) => {
                let ok = (c.selection_item && uia::select_item(&c).is_ok()) || (c.invoke && uia::invoke(&c).is_ok());
                if ok {
                    done(vec![("method", json!("SelectionItem")), ("option", json!(c.name)), ("after", json!(c.name))])
                } else {
                    Outcome::Unavailable(format!("could not pick '{}'", o2))
                }
            }
            None => {
                if n.has_value && !n.value_readonly && uia::set_value(&n.el, &o2).is_ok() {
                    done(vec![("method", json!("Value")), ("after", element_value(&n))])
                } else {
                    let names: Vec<String> = nodes.iter().skip(1).filter(|c| c.selection_item).map(|c| c.name.clone()).collect();
                    if opened {
                        let _ = uia::collapse(&n);
                    }
                    return Outcome::Unavailable(format!("No option '{}'. Options: {}", o2, names.join(", ")));
                }
            }
        };
        if opened && n.expand_state != 1 {
            let _ = uia::collapse(&n);
        }
        r
    });
    let rect = n.rect;
    let fv = value.clone();
    run_action(&name, mode, base, plan_of(up, bg), &fg, Box::new(move || {
        // foreground: click the element, select all, type the value
        if fv.is_empty() {
            return Err("this change can only be made through UI Automation, which refused it".into());
        }
        input::click_physical((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2, false, 1)?;
        sleep_ms(80);
        input::press_combo(&parse_combo("ctrl+a").unwrap())?;
        input::type_unicode(&fv)?;
        Ok(json!({"method": "SendInput"}).as_object().unwrap().clone())
    }))
}

pub fn select(a: &[String]) -> ! {
    let (mut base, up, mode, name) = base_and_mode("select", a);
    let app = resolve_app(&arg(a, 0));
    check_uipi(&app);
    base.insert("app".into(), json!(app.name));
    let all = parse_bool("all", &arg(a, 1), false);
    let text = arg(a, 2);
    let occurrence: usize = arg(a, 3).trim().parse().unwrap_or(1);
    let start_arg = arg(a, 4);
    let length_arg = arg(a, 5);
    let element = arg(a, 6);
    let n = if !element.is_empty() {
        uia::resolve_ref(&app, &element)
    } else {
        match app_text_target(&app) {
            Some((_, _, n)) => n,
            None => fail(format!("{} has no focused element: pass --element <ref> (from 'find' or 'state')", app.name)),
        }
    };
    base.insert("element".into(), uia::short_json(&n));
    let content = if !n.hwnd.0.is_null() && is_edit_class(&n.class) { get_text(n.hwnd) } else { uia::document_text(&n.el).or_else(|| uia::current_value(&n.el)).unwrap_or_default() };
    let total = content.chars().count();
    let (start, len) = if all {
        (0, total)
    } else if !text.is_empty() {
        let lower: Vec<char> = content.to_lowercase().chars().collect();
        let needle: Vec<char> = text.to_lowercase().chars().collect();
        let mut found = 0;
        let mut pos = None;
        let mut i = 0;
        while i + needle.len() <= lower.len() {
            if lower[i..i + needle.len()] == needle[..] {
                found += 1;
                if found == occurrence {
                    pos = Some(i);
                    break;
                }
            }
            i += 1;
        }
        match pos {
            Some(p) => (p, needle.len()),
            None => fail(format!("Text '{}' {}not found in the element", text, if occurrence > 1 { format!("(occurrence {}) ", occurrence) } else { String::new() })),
        }
    } else {
        let s: usize = start_arg.trim().parse().unwrap_or(0);
        let l: usize = length_arg.trim().parse().unwrap_or(0);
        if s + l > total {
            fail(format!("Range {}+{} is past the end of the text ({} characters)", s, l, total));
        }
        (s, l)
    };
    let selected: String = content.chars().skip(start).take(len).collect();
    base.insert("range".into(), json!({"start": start, "length": len}));
    base.insert("selected".into(), json!(selected));
    let fg = fg_for(Some(&app));
    let n2 = n.clone();
    let c2 = content.clone();
    let bg = Box::new(move || -> Outcome {
        if !n2.hwnd.0.is_null() && is_edit_class(&n2.class) {
            let (s16, e16) = (char_to_utf16(&c2, start), char_to_utf16(&c2, start + len));
            if set_sel(n2.hwnd, s16, e16 as isize) {
                return done(vec![("method", json!("EM_SETSEL"))]);
            }
        }
        match uia::text_select_range(&n2.el, start as i32, len as i32) {
            Ok(()) => done(vec![("method", json!("TextPattern"))]),
            Err(e) => Outcome::Unavailable(e),
        }
    });
    let rect = n.rect;
    run_action(&name, mode, base, plan_of(up, bg), &fg, Box::new(move || {
        if !all {
            return Err("selecting a range needs UI Automation or the control's own messages, which refused it".into());
        }
        input::click_physical((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2, false, 1)?;
        input::press_combo(&parse_combo("ctrl+a").unwrap())?;
        Ok(json!({"method": "SendInput"}).as_object().unwrap().clone())
    }))
}
