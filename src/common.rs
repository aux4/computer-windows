// Cross-platform pieces: JSON output and errors, argument parsing, the shared stop switch and
// action log, text matching, and the background/foreground decision. Everything here also
// builds on macOS/Linux so it can be unit-tested and exercised by the package's .test.md files
// without a Windows desktop.

use serde_json::{json, Map, Value};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const PROVIDER: &str = "win";
pub const CONTRACT: i64 = 1;

pub fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

pub fn arg(args: &[String], i: usize) -> String {
    args.get(i).cloned().unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// test seams (contract names, plus AUX4_COMPUTER_WINDOWS_* aliases)

fn seam(name: &str) -> String {
    let v = env(&format!("AUX4_COMPUTER_{}", name));
    if !v.is_empty() {
        return v;
    }
    env(&format!("AUX4_COMPUTER_WINDOWS_{}", name))
}

pub fn dry_run() -> bool {
    seam("DRY_RUN") == "1"
}

pub fn permissions_denied() -> bool {
    seam("DENY_PERMISSIONS") == "1"
}

pub fn simulate_background_unavailable() -> bool {
    seam("SIMULATE_BACKGROUND") == "unavailable"
}

// ---------------------------------------------------------------------------------------------
// output

pub fn to_json(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string())
}

pub fn print_json(v: Value) -> ! {
    action_finish(Some(&v), None);
    let mut out = std::io::stdout();
    let _ = writeln!(out, "{}", to_json(&v));
    let _ = out.flush();
    std::process::exit(0);
}

pub fn fail_json(v: Value) -> ! {
    action_finish(None, Some(&v));
    let mut err = std::io::stderr();
    let _ = writeln!(err, "{}", to_json(&v));
    let _ = err.flush();
    std::process::exit(1);
}

pub fn fail(msg: impl Into<String>) -> ! {
    fail_json(json!({ "error": msg.into() }))
}

pub fn obj(v: Value) -> Map<String, Value> {
    match v {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

// ---------------------------------------------------------------------------------------------
// argument parsing

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Auto,
    Background,
    Foreground,
}

pub fn parse_mode(s: &str) -> Mode {
    match s.trim().to_lowercase().as_str() {
        "" | "auto" => Mode::Auto,
        "background" => Mode::Background,
        "foreground" => Mode::Foreground,
        _ => fail(format!("Invalid --mode '{}': use auto, background or foreground", s)),
    }
}

pub fn parse_number(name: &str, raw: &str) -> Option<f64> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    match t.parse::<f64>() {
        Ok(v) if v.is_finite() => Some(v),
        _ => fail(format!("Invalid --{} '{}': use a number", name, raw)),
    }
}

pub fn parse_bool(name: &str, raw: &str, default: bool) -> bool {
    match raw.trim().to_lowercase().as_str() {
        "" => default,
        "true" | "yes" | "1" => true,
        "false" | "no" | "0" => false,
        _ => fail(format!("Invalid --{} '{}': use true or false", name, raw)),
    }
}

/// Parses 500ms / 10s / 2m / 1.5 (seconds). None for empty.
pub fn parse_duration_value(raw: &str) -> Result<Option<f64>, ()> {
    let t = raw.trim().to_lowercase();
    if t.is_empty() {
        return Ok(None);
    }
    let (num, mult) = if let Some(n) = t.strip_suffix("ms") {
        (n, 0.001)
    } else if let Some(n) = t.strip_suffix('s') {
        (n, 1.0)
    } else if let Some(n) = t.strip_suffix('m') {
        (n, 60.0)
    } else if let Some(n) = t.strip_suffix('h') {
        (n, 3600.0)
    } else {
        (t.as_str(), 1.0)
    };
    match num.trim().parse::<f64>() {
        Ok(v) if v.is_finite() && v >= 0.0 => Ok(Some(v * mult)),
        _ => Err(()),
    }
}

pub fn parse_duration(name: &str, raw: &str, default: f64) -> f64 {
    match parse_duration_value(raw) {
        Ok(Some(v)) => {
            if v <= 0.0 {
                fail(format!("Invalid --{} '{}': must be more than 0", name, raw));
            }
            v
        }
        Ok(None) => default,
        Err(()) => fail(format!("Invalid --{} '{}': use e.g. 500ms, 10s or 2m", name, raw)),
    }
}

pub fn format_seconds(s: f64) -> String {
    if s < 1.0 {
        format!("{}ms", (s * 1000.0).round() as i64)
    } else if (s - s.round()).abs() < 1e-9 {
        format!("{}s", s.round() as i64)
    } else {
        format!("{}s", s)
    }
}

pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub fn num(v: f64) -> Value {
    let r = round2(v);
    if (r - r.round()).abs() < 1e-9 && r.abs() < 9.0e15 {
        json!(r.round() as i64)
    } else {
        json!(r)
    }
}

// ---------------------------------------------------------------------------------------------
// text matching (contract): wait / ocr --text ignore case, whitespace and commas; find is a plain
// case-insensitive substring.

pub fn normalize_loose(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && *c != ',')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn loose_contains(haystack: &str, needle: &str) -> bool {
    let n = normalize_loose(needle);
    if n.is_empty() {
        return true;
    }
    normalize_loose(haystack).contains(&n)
}

pub fn plain_contains(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

// ---------------------------------------------------------------------------------------------
// roles: UIA control type names, with the macOS AX names accepted by --role

pub fn role_matches(native: &str, wanted: &str) -> bool {
    let w = wanted.trim();
    if w.is_empty() {
        return true;
    }
    let n = native.to_lowercase();
    let mut wl = w.to_lowercase();
    if let Some(rest) = wl.strip_prefix("controltype.") {
        wl = rest.to_string();
    }
    if n == wl {
        return true;
    }
    let mapped: &[&str] = match wl.as_str() {
        "axbutton" => &["button", "splitbutton"],
        "axtextfield" => &["edit"],
        "axtextarea" => &["edit", "document"],
        "axstatictext" => &["text"],
        "axcheckbox" => &["checkbox"],
        "axradiobutton" => &["radiobutton"],
        "axmenuitem" => &["menuitem"],
        "axmenu" => &["menu"],
        "axmenubar" | "axmenubaritem" => &["menubar", "menuitem"],
        "axwindow" | "axsheet" | "axdialog" => &["window"],
        "axpopupbutton" | "axcombobox" => &["combobox"],
        "axlink" => &["hyperlink"],
        "aximage" => &["image"],
        "axlist" => &["list"],
        "axrow" | "axcell" => &["listitem", "dataitem", "treeitem"],
        "axoutline" => &["tree"],
        "axtable" => &["table", "datagrid"],
        "axslider" => &["slider"],
        "axtabgroup" => &["tab"],
        "axtab" => &["tabitem"],
        "axgroup" => &["group", "pane"],
        "axscrollarea" => &["pane", "scrollbar"],
        "axscrollbar" => &["scrollbar"],
        "axtoolbar" => &["toolbar"],
        "axprogressindicator" => &["progressbar"],
        "axheading" => &["header", "headeritem"],
        "axsplitgroup" => &["pane"],
        "axincrementor" => &["spinner"],
        _ => &[],
    };
    mapped.iter().any(|m| *m == n)
}

// ---------------------------------------------------------------------------------------------
// stop switch + action log, shared with aux4/computer (core) and the other providers

pub fn state_dir() -> PathBuf {
    let custom = env("AUX4_COMPUTER_STATE_DIR");
    if !custom.is_empty() {
        return PathBuf::from(custom);
    }
    if cfg!(windows) {
        let base = env("LOCALAPPDATA");
        if !base.is_empty() {
            return PathBuf::from(base).join("aux4-computer");
        }
        return PathBuf::from(env("USERPROFILE")).join("AppData").join("Local").join("aux4-computer");
    }
    let home = PathBuf::from(env("HOME"));
    if cfg!(target_os = "macos") {
        return home.join("Library").join("Application Support").join("aux4-computer");
    }
    let xdg = env("XDG_STATE_HOME");
    if !xdg.is_empty() {
        return PathBuf::from(xdg).join("aux4-computer");
    }
    home.join(".local").join("state").join("aux4-computer")
}

pub fn stop_file() -> PathBuf {
    let custom = env("AUX4_COMPUTER_STOP_FILE");
    if !custom.is_empty() {
        return PathBuf::from(custom);
    }
    state_dir().join("STOP")
}

pub fn log_file() -> PathBuf {
    let custom = env("AUX4_COMPUTER_LOG_FILE");
    if !custom.is_empty() {
        return PathBuf::from(custom);
    }
    state_dir().join("actions.jsonl")
}

fn log_max_bytes() -> u64 {
    env("AUX4_COMPUTER_LOG_MAX_BYTES").parse::<u64>().ok().filter(|n| *n > 0).unwrap_or(10 * 1024 * 1024)
}

pub fn path_str(p: &PathBuf) -> String {
    p.to_string_lossy().to_string()
}

pub fn stop_reason() -> Option<String> {
    if env("AUX4_COMPUTER_STOP") == "1" {
        return Some("AUX4_COMPUTER_STOP=1 is set".to_string());
    }
    let f = stop_file();
    if f.exists() {
        return Some(format!("{} exists", path_str(&f)));
    }
    None
}

/// Commands that send input or change state, with the names of their positional arguments.
pub fn action_arg_names(command: &str, args: &[String]) -> Option<Vec<&'static str>> {
    let names: &[&str] = match command {
        "click" => &["x", "y", "button", "count", "element", "app", "mode"],
        "move" => &["x", "y"],
        "drag" => &["x", "y", "toX", "toY"],
        "scroll" => &["x", "y", "dx", "dy", "app", "mode"],
        "type" => &["text", "delay", "app", "mode"],
        "shortcut" => &["keys", "delay", "app", "mode"],
        "clipboard-copy" => &["text", "app", "mode"],
        "clipboard-paste" => &["text", "app", "mode"],
        "open" => &["app", "mode", "file", "url", "wait", "hidden"],
        "focus" => &["app"],
        "set" => &["app", "element", "value", "checked", "option", "mode"],
        "select" => &["app", "all", "text", "occurrence", "start", "length", "element", "mode"],
        "menu" => &["app", "path", "depth", "mode"],
        "dialog" => &["app", "press", "mode"],
        "quit" => &["app", "force", "timeout"],
        "hide" => &["app"],
        "unhide" => &["app"],
        "window" => match arg(args, 0).to_lowercase().as_str() {
            "move" => &["action", "app", "title", "index", "mode", "x", "y"],
            "resize" => &["action", "app", "title", "index", "mode", "width", "height"],
            "fullscreen" => &["action", "app", "title", "index", "mode", "enabled"],
            _ => &["action", "app", "title", "index", "mode"],
        },
        _ => return None,
    };
    Some(names.to_vec())
}

pub fn is_action_invocation(command: &str, args: &[String]) -> bool {
    if action_arg_names(command, args).is_none() {
        return false;
    }
    match command {
        "menu" => !arg(args, 1).is_empty(),
        "dialog" => !arg(args, 1).is_empty(),
        "window" => arg(args, 0).to_lowercase() != "list",
        _ => true,
    }
}

pub fn truncated_for_log(s: &str, limit: usize) -> String {
    let n = s.chars().count();
    if n <= limit {
        return s.to_string();
    }
    let prefix: String = s.chars().take(limit).collect();
    format!("{}… ({} chars)", prefix, n)
}

struct ActionCtx {
    command: String,
    args: Map<String, Value>,
    start: Instant,
    ts: String,
}

static ACTION: Mutex<Option<ActionCtx>> = Mutex::new(None);

pub fn command_display(command: &str, args: &[String]) -> String {
    let mut c = command.replace('-', " ");
    if command == "window" {
        c = format!("window {}", arg(args, 0).to_lowercase());
    }
    c
}

pub fn action_begin(command: &str, args: &[String]) {
    let names = match action_arg_names(command, args) {
        Some(n) => n,
        None => return,
    };
    let mut named = Map::new();
    for (i, name) in names.iter().enumerate() {
        if *name == "action" {
            continue;
        }
        let v = arg(args, i);
        if v.is_empty() {
            continue;
        }
        let v = if *name == "text" || *name == "value" { truncated_for_log(&v, 16) } else { v };
        named.insert(name.to_string(), Value::String(v));
    }
    *ACTION.lock().unwrap() = Some(ActionCtx {
        command: command_display(command, args),
        args: named,
        start: Instant::now(),
        ts: iso_now(),
    });
}

fn action_finish(result: Option<&Value>, error: Option<&Value>) {
    let ctx = match ACTION.lock().unwrap().take() {
        Some(c) => c,
        None => return,
    };
    let out = result.or(error).cloned().unwrap_or(json!({}));
    // A dry run never touches anything: nothing to record.
    if out.get("dryRun").and_then(|v| v.as_bool()) == Some(true) {
        return;
    }
    let mut e = Map::new();
    e.insert("ts".into(), json!(ctx.ts));
    e.insert("provider".into(), json!(PROVIDER));
    e.insert("command".into(), json!(ctx.command));
    e.insert("ms".into(), json!(ctx.start.elapsed().as_millis() as u64));
    let app = out.get("app").and_then(|v| v.as_str()).map(|s| s.to_string()).or_else(|| ctx.args.get("app").and_then(|v| v.as_str()).map(|s| s.to_string()));
    if let Some(a) = app {
        e.insert("app".into(), json!(a));
    }
    e.insert("args".into(), Value::Object(ctx.args));
    if let Some(m) = out.get("mode").and_then(|v| v.as_str()) {
        e.insert("mode".into(), json!(m));
    }
    if result.is_some() {
        e.insert("result".into(), json!("ok"));
        if let Some(m) = out.get("method").or_else(|| out.get("action")).and_then(|v| v.as_str()) {
            e.insert("method".into(), json!(m));
        }
        if let Some(r) = out.get("fallbackReason").and_then(|v| v.as_str()) {
            e.insert("fallbackReason".into(), json!(r));
        }
    } else {
        e.insert("result".into(), json!("error"));
        let msg = out.get("error").and_then(|v| v.as_str()).unwrap_or("unknown error");
        e.insert("error".into(), json!(truncated_for_log(msg, 300)));
    }
    append_action_log(Value::Object(e));
}

pub fn append_action_log(entry: Value) {
    let line = to_json(&entry);
    let path = log_file();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() + line.len() as u64 + 1 > log_max_bytes() {
            let rotated = PathBuf::from(format!("{}.1", path_str(&path)));
            let _ = std::fs::remove_file(&rotated);
            let _ = std::fs::rename(&path, &rotated);
        }
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    if let Ok(mut f) = opts.open(&path) {
        let _ = f.write_all(format!("{}\n", line).as_bytes());
    }
}

pub fn guard_stop_switch(command: &str, args: &[String]) {
    if let Some(reason) = stop_reason() {
        fail_json(json!({
            "error": format!("Stopped: aux4 computer win is not sending input or changing state ({}). Run 'aux4 computer win resume' (or 'aux4 computer resume') to allow it again.", reason),
            "stopped": true,
            "command": command_display(command, args),
        }));
    }
}

pub fn cmd_stop() -> ! {
    let path = stop_file();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let content = format!("stopped at {} by 'aux4 computer win stop'; 'aux4 computer win resume' removes this file\n", iso_now());
    if std::fs::write(&path, content).is_err() {
        fail(format!("Could not create the stop file at {}", path_str(&path)));
    }
    append_action_log(json!({"ts": iso_now(), "provider": PROVIDER, "command": "stop", "args": {}, "result": "ok"}));
    print_json(json!({"stopped": true, "file": path_str(&path)}))
}

pub fn cmd_resume() -> ! {
    let path = stop_file();
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            fail(format!("Could not remove the stop file at {}: {}", path_str(&path), e));
        }
    }
    append_action_log(json!({"ts": iso_now(), "provider": PROVIDER, "command": "resume", "args": {}, "result": "ok"}));
    let mut out = json!({"stopped": false, "file": path_str(&path)});
    if env("AUX4_COMPUTER_STOP") == "1" {
        out["stopped"] = json!(true);
        out["stopReason"] = json!("AUX4_COMPUTER_STOP=1 is set; unset it to allow input");
    }
    print_json(out)
}

pub fn cmd_log(args: &[String]) -> ! {
    let raw = arg(args, 0);
    let limit: i64 = if raw.trim().is_empty() { 20 } else { raw.trim().parse().unwrap_or(-1) };
    if limit <= 0 {
        fail(format!("Invalid --limit '{}': use a positive number", raw));
    }
    let limit = limit as usize;
    let path = log_file();
    let mut lines: Vec<String> = Vec::new();
    for file in [path.clone(), PathBuf::from(format!("{}.1", path_str(&path)))] {
        if lines.len() >= limit {
            break;
        }
        if let Ok(text) = std::fs::read_to_string(&file) {
            let fl: Vec<String> = text.lines().filter(|l| !l.trim().is_empty()).map(|s| s.to_string()).collect();
            let take = limit - lines.len();
            let start = fl.len().saturating_sub(take);
            let mut chunk = fl[start..].to_vec();
            chunk.extend(lines);
            lines = chunk;
        }
    }
    let entries: Vec<Value> = lines.iter().filter_map(|l| serde_json::from_str(l).ok()).collect();
    let mut out = json!({"path": path_str(&path), "entries": entries, "stopped": stop_reason().is_some()});
    if let Some(r) = stop_reason() {
        out["stopReason"] = json!(r);
    }
    print_json(out)
}

// ---------------------------------------------------------------------------------------------
// time

pub fn iso_now() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    iso_from_unix_ms(d.as_millis() as i64)
}

pub fn iso_from_unix_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let millis = ms.rem_euclid(1000);
    let days = secs.div_euclid(86400);
    let sod = secs.rem_euclid(86400);
    // civil-from-days (Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z", y, m, d, sod / 3600, (sod % 3600) / 60, sod % 60, millis)
}

// ---------------------------------------------------------------------------------------------
// the background/foreground decision (shared by every action)

pub enum Outcome {
    Done(Map<String, Value>),
    Unavailable(String),
}

pub enum Plan<'a> {
    /// Try this in the background; on Unavailable, auto falls back to real input.
    Attempt(Box<dyn FnOnce() -> Outcome + 'a>),
    /// Known up front that background can't do it.
    Unavailable(String),
    /// Foreground by nature (e.g. a system-wide hotkey): no fallback reason, no restore.
    InherentlyForeground(String),
}

/// Hooks the platform layer provides for the foreground path.
pub trait Foreground {
    /// Save cursor + foreground window.
    fn save(&self) -> Option<Box<dyn std::any::Any>>;
    /// Bring the target forward (Err = could not).
    fn bring_forward(&self) -> Result<(), String>;
    /// Put the saved state back; returns {"cursor":bool,"frontmost":bool}.
    fn restore(&self, saved: Box<dyn std::any::Any>) -> Value;
}

pub struct NoForeground;
impl Foreground for NoForeground {
    fn save(&self) -> Option<Box<dyn std::any::Any>> {
        None
    }
    fn bring_forward(&self) -> Result<(), String> {
        Ok(())
    }
    fn restore(&self, _saved: Box<dyn std::any::Any>) -> Value {
        json!({"cursor": true, "frontmost": true})
    }
}

pub fn run_action<'a>(
    name: &str,
    mode: Mode,
    base: Map<String, Value>,
    plan: Plan<'a>,
    fg: &dyn Foreground,
    foreground: Box<dyn FnOnce() -> Result<Map<String, Value>, String> + 'a>,
) -> ! {
    let mut out = base;
    let dry = dry_run();
    if dry {
        out.insert("dryRun".into(), json!(true));
    }
    let mut fallback: Option<String> = None;
    match plan {
        Plan::Attempt(bg) => {
            if mode != Mode::Foreground {
                let outcome = if dry {
                    if simulate_background_unavailable() {
                        Outcome::Unavailable("background simulated as unavailable".into())
                    } else {
                        Outcome::Done(Map::new())
                    }
                } else if simulate_background_unavailable() {
                    Outcome::Unavailable("background simulated as unavailable".into())
                } else {
                    bg()
                };
                match outcome {
                    Outcome::Done(extra) => {
                        for (k, v) in extra {
                            out.insert(k, v);
                        }
                        out.insert("mode".into(), json!("background"));
                        print_json(Value::Object(out));
                    }
                    Outcome::Unavailable(reason) => {
                        if mode == Mode::Background {
                            fail(format!("Background mode not possible for {}: {}", name, reason));
                        }
                        fallback = Some(reason);
                    }
                }
            }
        }
        Plan::Unavailable(reason) => {
            if mode == Mode::Background {
                fail(format!("Background mode not possible for {}: {}", name, reason));
            }
            if mode == Mode::Auto {
                fallback = Some(reason);
            }
        }
        Plan::InherentlyForeground(reason) => {
            if mode == Mode::Background {
                fail(format!("Background mode not possible for {}: {}", name, reason));
            }
        }
    }
    out.insert("mode".into(), json!("foreground"));
    if let Some(r) = &fallback {
        out.insert("fallbackReason".into(), json!(r));
    }
    if dry {
        print_json(Value::Object(out));
    }
    let saved = if fallback.is_some() { fg.save() } else { None };
    if let Err(e) = fg.bring_forward() {
        fail(e);
    }
    match foreground() {
        Ok(extra) => {
            for (k, v) in extra {
                out.insert(k, v);
            }
        }
        Err(e) => fail(e),
    }
    if let Some(s) = saved {
        std::thread::sleep(Duration::from_millis(120));
        out.insert("restored".into(), fg.restore(s));
    }
    print_json(Value::Object(out))
}

// ---------------------------------------------------------------------------------------------
// keys: parse a combo like ctrl+shift+s

#[derive(Debug, Clone, PartialEq)]
pub struct Combo {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
    pub key: String, // normalized key name (lowercase) or single character
}

pub fn parse_combo(combo: &str) -> Result<Combo, String> {
    let c = combo.trim();
    if c.is_empty() {
        return Err("--keys is required".into());
    }
    // "ctrl++" means ctrl and the plus key
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = c.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '+' && !cur.is_empty() {
            parts.push(cur.clone());
            cur.clear();
        } else {
            cur.push(ch);
        }
        i += 1;
    }
    if !cur.is_empty() {
        parts.push(cur);
    }
    if parts.is_empty() {
        return Err(format!("Invalid key combo: {}", combo));
    }
    let key = parts.pop().unwrap();
    let mut out = Combo { ctrl: false, shift: false, alt: false, win: false, key: String::new() };
    for m in parts {
        match m.trim().to_lowercase().as_str() {
            "ctrl" | "control" | "cmd" | "command" => out.ctrl = true,
            "shift" => out.shift = true,
            "alt" | "option" | "opt" => out.alt = true,
            "win" | "windows" | "meta" | "super" => out.win = true,
            other => return Err(format!("Unknown modifier: {}", other)),
        }
    }
    let k = key.trim().to_lowercase();
    let k = if k.is_empty() { key.clone() } else { k };
    if !is_known_key(&k) {
        return Err(format!("Unknown key: {}", key));
    }
    out.key = k;
    Ok(out)
}

pub fn is_known_key(k: &str) -> bool {
    if k.chars().count() == 1 {
        return true;
    }
    if let Some(n) = k.strip_prefix('f') {
        if let Ok(v) = n.parse::<u32>() {
            return (1..=24).contains(&v);
        }
    }
    matches!(
        k,
        "enter" | "return" | "tab" | "space" | "escape" | "esc" | "backspace" | "delete" | "del" | "forwarddelete"
            | "insert" | "home" | "end" | "pageup" | "pagedown" | "up" | "down" | "left" | "right" | "capslock"
            | "printscreen" | "menu" | "apps" | "plus" | "minus"
    )
}

pub fn combo_display(c: &Combo) -> String {
    let mut s = Vec::new();
    if c.ctrl {
        s.push("ctrl".to_string());
    }
    if c.alt {
        s.push("alt".to_string());
    }
    if c.shift {
        s.push("shift".to_string());
    }
    if c.win {
        s.push("win".to_string());
    }
    s.push(c.key.clone());
    s.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loose_matching_ignores_case_space_commas() {
        assert!(loose_contains("Display is 1,792", "1792"));
        assert!(loose_contains("HELLO  World", "hello world"));
        assert!(!loose_contains("1,793", "1792"));
        assert!(loose_contains("anything", ""));
    }

    #[test]
    fn plain_matching_is_substring() {
        assert!(plain_contains("Save As", "save"));
        assert!(!plain_contains("1,792", "1792"));
    }

    #[test]
    fn durations() {
        assert_eq!(parse_duration_value("500ms"), Ok(Some(0.5)));
        assert_eq!(parse_duration_value("10s"), Ok(Some(10.0)));
        assert_eq!(parse_duration_value("2m"), Ok(Some(120.0)));
        assert_eq!(parse_duration_value("1.5"), Ok(Some(1.5)));
        assert_eq!(parse_duration_value(""), Ok(None));
        assert_eq!(parse_duration_value("soon"), Err(()));
        assert_eq!(format_seconds(0.5), "500ms");
        assert_eq!(format_seconds(10.0), "10s");
    }

    #[test]
    fn roles_accept_mac_names() {
        assert!(role_matches("Button", "AXButton"));
        assert!(role_matches("Edit", "AXTextField"));
        assert!(role_matches("Document", "AXTextArea"));
        assert!(role_matches("Text", "AXStaticText"));
        assert!(role_matches("Button", "button"));
        assert!(role_matches("Button", "ControlType.Button"));
        assert!(!role_matches("Edit", "AXButton"));
    }

    #[test]
    fn combos() {
        let c = parse_combo("ctrl+shift+s").unwrap();
        assert!(c.ctrl && c.shift && !c.alt);
        assert_eq!(c.key, "s");
        assert_eq!(parse_combo("cmd+c").unwrap().ctrl, true);
        assert_eq!(parse_combo("ctrl++").unwrap().key, "+");
        assert_eq!(parse_combo("enter").unwrap().key, "enter");
        assert!(parse_combo("hyper+x").is_err());
        assert!(parse_combo("ctrl+nokey").is_err());
        assert_eq!(combo_display(&parse_combo("shift+ctrl+F4").unwrap()), "ctrl+shift+f4");
    }

    #[test]
    fn log_truncation() {
        assert_eq!(truncated_for_log("short", 16), "short");
        assert_eq!(truncated_for_log("this text is longer than sixteen", 16), "this text is lon… (32 chars)");
    }

    #[test]
    fn iso_time() {
        assert_eq!(iso_from_unix_ms(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso_from_unix_ms(1_793_664_000_123), "2026-11-03T00:00:00.123Z");
    }

    #[test]
    fn action_classification() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(is_action_invocation("click", &a(&["1", "2"])));
        assert!(!is_action_invocation("menu", &a(&["Notepad", ""])));
        assert!(is_action_invocation("menu", &a(&["Notepad", "File > New"])));
        assert!(!is_action_invocation("window", &a(&["list", "Notepad"])));
        assert!(is_action_invocation("window", &a(&["move", "Notepad"])));
        assert!(!is_action_invocation("find", &a(&[])));
        assert_eq!(command_display("clipboard-paste", &[]), "clipboard paste");
    }
}
