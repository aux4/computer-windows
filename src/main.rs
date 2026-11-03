// aux4-computer-windows: computer-use for AI agents on Windows (aux4/computer provider "win").
//
// main.rs validates every command's arguments and handles the platform-independent parts
// (stop switch, action log, dry runs, info). The Windows work (UI Automation, input, capture,
// OCR, clipboard, windows) lives in src/win and is only compiled for Windows targets.

mod common;
#[cfg(windows)]
mod win;

use common::*;
use serde_json::{json, Map, Value};

#[cfg(not(windows))]
mod plat {
    use super::common::*;
    use serde_json::json;
    pub fn exec(command: &str, _args: &[String]) -> ! {
        if command == "permissions" {
            print_json(permissions_json());
        }
        fail_json(json!({
            "error": "aux4/computer-windows only runs on Windows (this is a different OS); use the provider for this OS, or 'aux4 computer' to pick it",
            "unsupported": true
        }))
    }
    pub fn permissions_json() -> serde_json::Value {
        json!({"elevated": {"granted": false, "guidance": "Not running on Windows"}, "uiAccess": {"granted": false, "guidance": "Not running on Windows"}, "desktop": {"granted": false, "guidance": "Not running on Windows"}})
    }
    pub fn require_desktop() {
        if permissions_denied() {
            super::desktop_denied();
        }
    }
}
#[cfg(windows)]
use win as plat;

pub fn desktop_denied() -> ! {
    fail_json(json!({
        "error": "Permission not granted: this process has no interactive, unlocked desktop, so UI Automation, input and screen capture can't reach any app. Run aux4 from a signed-in user's desktop session (not a Windows service, a non-interactive scheduled task or an SSH session) and unlock the screen, then re-run.",
        "permission": "desktop"
    }))
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "internal error".to_string()
        };
        let loc = info.location().map(|l| format!(" at {}:{}", l.file(), l.line())).unwrap_or_default();
        eprintln!("{}", to_json(&json!({"error": format!("Internal error: {}{}", msg, loc)})));
        std::process::exit(1);
    }));

    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.is_empty() {
        fail("Usage: aux4-computer-windows <command> [args...]");
    }
    let command = argv[0].clone();
    let args: Vec<String> = argv[1..].to_vec();

    match command.as_str() {
        "stop" => cmd_stop(),
        "resume" => cmd_resume(),
        "log" => cmd_log(&args),
        "info" => cmd_info(&args),
        "notes" => cmd_notes(&args),
        _ => {}
    }

    if is_action_invocation(&command, &args) {
        action_begin(&command, &args);
        guard_stop_switch(&command, &args);
    }

    validate(&command, &args);

    // OCR of an existing image needs no desktop at all.
    let needs_desktop = !(command == "ocr" && !arg(&args, 1).is_empty()) && command != "permissions";
    if needs_desktop && (permissions_denied() || !(dry_run() && dry_run_is_platform_free(&command, &args))) {
        plat::require_desktop();
    }

    if dry_run() && dry_run_is_platform_free(&command, &args) {
        dry_run_action(&command, &args);
    }
    plat::exec(&command, &args)
}

const NOTES: &str = include_str!("../package/instructions/notes.md");

fn cmd_notes(args: &[String]) -> ! {
    let dir = arg(args, 0);
    let text = std::fs::read_to_string(std::path::Path::new(&dir).join("instructions").join("notes.md")).unwrap_or_else(|_| NOTES.to_string());
    print!("{}", text);
    std::process::exit(0)
}

fn cmd_info(args: &[String]) -> ! {
    let package_dir = arg(args, 0);
    let mut version = json!(env!("CARGO_PKG_VERSION"));
    if !package_dir.is_empty() {
        let p = std::path::Path::new(&package_dir).join(".aux4");
        if let Ok(text) = std::fs::read_to_string(p) {
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                if let Some(s) = v.get("version").and_then(|x| x.as_str()) {
                    version = json!(s);
                }
            }
        }
    }
    let os = if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "darwin" } else { std::env::consts::OS };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "x86" => "386",
        a => a,
    };
    print_json(json!({
        "provider": PROVIDER,
        "contract": CONTRACT,
        "platform": format!("{}/{}", os, arch),
        "version": version,
        "binaryVersion": env!("CARGO_PKG_VERSION"),
        "capabilities": {
            "background": true,
            "foregroundFallback": true,
            "ocr": true,
            "annotate": true,
            "screenshotCoveredWindow": true,
            "menus": true,
            "dialogs": true,
            "clipboardText": true,
            "windowOps": true,
            "fullscreen": false,
            "hide": false,
            "logicalCoordinates": true
        },
        "permissions": plat::permissions_json(),
        "stateDir": path_str(&state_dir()),
        "logFile": path_str(&log_file()),
        "stopFile": path_str(&stop_file())
    }))
}

// ---------------------------------------------------------------------------------------------
// argument validation (identical messages to computer-mac where the flag exists there)

fn validate(command: &str, a: &[String]) {
    match command {
        "screenshot" => {
            // output, display, app, maxWidth, annotate, roles, limit
            let d = arg(a, 1);
            if !d.trim().is_empty() && d.trim().parse::<usize>().is_err() {
                fail(format!("Invalid display index: {}", d));
            }
            if let Some(w) = parse_number("maxWidth", &arg(a, 3)) {
                if w <= 0.0 {
                    fail(format!("Invalid --maxWidth '{}': use a positive number", arg(a, 3)));
                }
            }
            parse_bool("annotate", &arg(a, 4), false);
            let roles = arg(a, 5).to_lowercase();
            if !roles.is_empty() && roles != "actionable" && roles != "all" {
                fail(format!("Invalid --roles '{}': use actionable or all", arg(a, 5)));
            }
            positive_int("limit", &arg(a, 6));
        }
        "ocr" => {
            // app, image, text, level, languages, mode, scale, timeout
            if !arg(a, 0).is_empty() && !arg(a, 1).is_empty() {
                fail("Use either --app or --image, not both");
            }
            let level = arg(a, 3).to_lowercase();
            if !level.is_empty() && level != "accurate" && level != "fast" {
                fail(format!("Invalid --level '{}': use accurate or fast", arg(a, 3)));
            }
            parse_mode(&arg(a, 5));
            parse_number("scale", &arg(a, 6));
            parse_duration("timeout", &arg(a, 7), 60.0);
            let img = arg(a, 1);
            if !img.is_empty() && !std::path::Path::new(&img).exists() {
                fail(format!("Image not found: {}", img));
            }
        }
        "wait" => {
            // text, role, app, gone, source, timeout, interval
            let gone = arg(a, 3);
            if !matches!(gone.to_lowercase().as_str(), "" | "true" | "false") {
                fail(format!("Invalid --gone '{}': use true or false", gone));
            }
            let source = arg(a, 4).to_lowercase();
            if !matches!(source.as_str(), "" | "auto" | "ax" | "uia" | "ocr") {
                fail(format!("Invalid --source '{}': use auto, ax or ocr", arg(a, 4)));
            }
            if source == "ocr" && !arg(a, 1).is_empty() {
                fail("--role needs the accessibility tree: use --source auto or ax with --role");
            }
            if arg(a, 0).is_empty() && arg(a, 1).is_empty() && arg(a, 2).is_empty() {
                fail("Nothing to wait for: give --text (optionally with --role), --role, or --app");
            }
            parse_duration("timeout", &arg(a, 5), 10.0);
            parse_duration("interval", &arg(a, 6), 0.5);
        }
        "read" => {
            nonneg_int("depth", &arg(a, 1));
        }
        "state" => {
            nonneg_int("depth", &arg(a, 1));
            positive_int("limit", &arg(a, 2));
        }
        "click" => {
            // x, y, button, count, element, app, mode
            let b = arg(a, 2).to_lowercase();
            if !matches!(b.as_str(), "" | "left" | "right") {
                fail(format!("Invalid --button '{}': use left or right", arg(a, 2)));
            }
            if let Some(c) = parse_number("count", &arg(a, 3)) {
                if c < 1.0 || c.fract() != 0.0 {
                    fail(format!("Invalid --count '{}': use 1 or more", arg(a, 3)));
                }
            }
            parse_number("x", &arg(a, 0));
            parse_number("y", &arg(a, 1));
            if arg(a, 4).is_empty() && (arg(a, 0).is_empty() || arg(a, 1).is_empty()) {
                fail("Provide --x/--y or --element");
            }
            parse_mode(&arg(a, 6));
        }
        "move" => {
            if arg(a, 0).is_empty() || arg(a, 1).is_empty() {
                fail("--x and --y are required");
            }
            parse_number("x", &arg(a, 0));
            parse_number("y", &arg(a, 1));
        }
        "drag" => {
            for (i, n) in ["x", "y", "toX", "toY"].iter().enumerate() {
                if arg(a, i).is_empty() {
                    fail("--x, --y, --toX and --toY are required");
                }
                parse_number(n, &arg(a, i));
            }
        }
        "scroll" => {
            parse_number("x", &arg(a, 0));
            parse_number("y", &arg(a, 1));
            parse_number("dx", &arg(a, 2));
            parse_number("dy", &arg(a, 3));
            parse_mode(&arg(a, 5));
        }
        "type" => {
            if arg(a, 0).is_empty() {
                fail("--text is required");
            }
            nonneg_int("delay", &arg(a, 1));
            parse_mode(&arg(a, 3));
        }
        "shortcut" => {
            if let Err(e) = parse_combo(&arg(a, 0)) {
                fail(e);
            }
            nonneg_int("delay", &arg(a, 1));
            parse_mode(&arg(a, 3));
        }
        "clipboard-copy" | "clipboard-paste" => {
            parse_mode(&arg(a, 2));
        }
        "open" => {
            // app, mode, file, url, wait, hidden
            let mode = parse_mode(&arg(a, 1));
            let hidden = parse_bool("hidden", &arg(a, 5), false);
            if hidden && mode == Mode::Foreground {
                fail("--hidden true opens the app without showing it; it can't be combined with --mode foreground");
            }
            let (file, url) = (arg(a, 2), arg(a, 3));
            if !file.is_empty() && !url.is_empty() {
                fail("Give either --file or --url, not both");
            }
            if arg(a, 0).is_empty() && file.is_empty() && url.is_empty() {
                fail("--app is required (or give --file or --url)");
            }
            if !url.is_empty() && !url.contains("://") && !url.starts_with("mailto:") {
                fail(format!("Invalid --url '{}': include the scheme, e.g. https://aux4.io", url));
            }
            if !file.is_empty() && !dry_run() && !std::path::Path::new(&file).exists() {
                fail(format!("File not found: {}", file));
            }
            parse_duration("wait", &arg(a, 4), 10.0);
        }
        "focus" | "hide" | "unhide" => {
            if arg(a, 0).is_empty() {
                fail("--app is required");
            }
        }
        "quit" => {
            if arg(a, 0).is_empty() {
                fail("--app is required");
            }
            parse_bool("force", &arg(a, 1), false);
            parse_duration("timeout", &arg(a, 2), 5.0);
        }
        "menu" => {
            // app, path, depth, mode
            let depth = arg(a, 2);
            if !depth.is_empty() {
                match depth.trim().parse::<i64>() {
                    Ok(d) if d >= 1 => {}
                    _ => fail(format!("Invalid --depth '{}': use 1 (menu titles), 2 (their items) or more for submenus", depth)),
                }
            }
            let path = arg(a, 1);
            if !path.is_empty() {
                let parts = menu_path_parts(&path);
                if parts.iter().any(|p| p.is_empty()) {
                    fail(format!("Invalid --path '{}': separate menu titles with '>', e.g. 'Format > Font > Bold'", path));
                }
                if parts.len() < 2 {
                    fail(format!("Invalid --path '{}': give a menu item, not just a menu, e.g. '{} > ...'", path, parts.first().cloned().unwrap_or_default()));
                }
            }
            parse_mode(&arg(a, 3));
        }
        "select" => {
            // app, all, text, occurrence, start, length, element, mode
            let all = parse_bool("all", &arg(a, 1), false);
            let text = arg(a, 2);
            let has_range = !arg(a, 4).is_empty() || !arg(a, 5).is_empty();
            let given = [all, !text.is_empty(), has_range].iter().filter(|b| **b).count();
            if given == 0 {
                fail("Say what to select: --all true, --text <text> [--occurrence N], or --start N --length N");
            }
            if given > 1 {
                fail("Give only one of --all, --text, or --start/--length");
            }
            let occ = arg(a, 3);
            if !occ.is_empty() {
                if text.is_empty() {
                    fail("--occurrence goes with --text");
                }
                match occ.trim().parse::<i64>() {
                    Ok(n) if n >= 1 => {}
                    _ => fail(format!("Invalid --occurrence '{}': use 1 for the first match, 2 for the second...", occ)),
                }
            }
            if has_range {
                let s = arg(a, 4);
                match s.trim().parse::<i64>() {
                    Ok(n) if n >= 0 => {}
                    _ => fail(format!("Invalid --start '{}': use a character offset (0 or more)", s)),
                }
                let l = arg(a, 5);
                if !l.is_empty() {
                    match l.trim().parse::<i64>() {
                        Ok(n) if n >= 0 => {}
                        _ => fail(format!("Invalid --length '{}': use 0 or more characters", l)),
                    }
                }
            }
            parse_mode(&arg(a, 7));
        }
        "set" => {
            // app, element, value, checked, option, mode
            if arg(a, 1).is_empty() {
                fail("--element is required: the element's ref from 'find' or 'state'");
            }
            let given = [!arg(a, 2).is_empty(), !arg(a, 3).is_empty(), !arg(a, 4).is_empty()].iter().filter(|b| **b).count();
            if given == 0 {
                fail("Say what to set: --value <text|number>, --checked true|false, or --option <title>");
            }
            if given > 1 {
                fail("Give only one of --value, --checked, or --option");
            }
            let c = arg(a, 3);
            if !matches!(c.to_lowercase().as_str(), "" | "true" | "false") {
                fail(format!("Invalid --checked '{}': use true or false", c));
            }
            parse_mode(&arg(a, 5));
        }
        "dialog" => {
            parse_mode(&arg(a, 2));
        }
        "window" => {
            // action, app, title, index, mode, [x y | width height | enabled]
            let action = arg(a, 0).to_lowercase();
            let actions = ["list", "move", "resize", "minimize", "restore", "close", "fullscreen", "raise"];
            if !actions.contains(&action.as_str()) {
                fail(format!("Invalid window action '{}': use {}", arg(a, 0), actions.join(", ")));
            }
            if action == "list" {
                return;
            }
            if !arg(a, 2).is_empty() && !arg(a, 3).is_empty() {
                fail("Give either --title or --index, not both");
            }
            nonneg_int("index", &arg(a, 3));
            parse_mode(&arg(a, 4));
            match action.as_str() {
                "move" => {
                    if arg(a, 5).is_empty() || arg(a, 6).is_empty() {
                        fail("--x and --y are required (screen points of the window's top-left corner)");
                    }
                    parse_number("x", &arg(a, 5));
                    parse_number("y", &arg(a, 6));
                }
                "resize" => {
                    let w = parse_number("width", &arg(a, 5));
                    let h = parse_number("height", &arg(a, 6));
                    if w.is_none() && h.is_none() {
                        fail("--width or --height is required");
                    }
                    if w.map(|v| v <= 0.0).unwrap_or(false) || h.map(|v| v <= 0.0).unwrap_or(false) {
                        fail("--width and --height must be positive");
                    }
                }
                "fullscreen" => {
                    parse_bool("enabled", &arg(a, 5), true);
                }
                "raise" => {
                    if parse_mode(&arg(a, 4)) == Mode::Background {
                        fail("Background mode not possible for window raise: raising a window brings its app to the front");
                    }
                }
                _ => {}
            }
        }
        "find" | "permissions" | "cursor" | "screen" | "apps" | "windows" | "clipboard-read" => {}
        _ => fail(format!("Unknown command: {}", command)),
    }
}

fn positive_int(name: &str, raw: &str) {
    if raw.trim().is_empty() {
        return;
    }
    match raw.trim().parse::<i64>() {
        Ok(n) if n > 0 => {}
        _ => fail(format!("Invalid --{} '{}': use a positive number", name, raw)),
    }
}

fn nonneg_int(name: &str, raw: &str) {
    if raw.trim().is_empty() {
        return;
    }
    match raw.trim().parse::<i64>() {
        Ok(n) if n >= 0 => {}
        _ => fail(format!("Invalid --{} '{}': use 0 or more", name, raw)),
    }
}

pub fn menu_path_parts(path: &str) -> Vec<String> {
    path.split('>').map(|p| p.trim().to_string()).collect()
}

// ---------------------------------------------------------------------------------------------
// dry runs that don't need the desktop: the decision and the JSON shape, never touching anything

fn dry_run_is_platform_free(command: &str, args: &[String]) -> bool {
    match command {
        "click" | "move" | "drag" | "scroll" | "type" | "shortcut" | "clipboard-paste" | "open" | "focus" | "set" | "select" => true,
        "clipboard-copy" => arg(args, 0).is_empty(),
        "menu" | "dialog" => is_action_invocation(command, args),
        "window" => arg(args, 0).to_lowercase() != "list",
        _ => false,
    }
}

pub fn opt_insert(m: &mut Map<String, Value>, key: &str, v: &str) {
    if !v.is_empty() {
        m.insert(key.into(), json!(v));
    }
}

pub fn num_arg(raw: &str) -> Value {
    num(raw.trim().parse::<f64>().unwrap_or(0.0))
}

/// The base JSON and the up-front plan for an action, shared by dry runs and real runs.
pub fn action_base(command: &str, a: &[String]) -> (Map<String, Value>, Option<(bool, String)>) {
    // Some((true, reason)) = inherently foreground; Some((false, reason)) = background known unavailable.
    let mut m = Map::new();
    let mut plan = None;
    match command {
        "click" => {
            let count = arg(a, 3).trim().parse::<f64>().unwrap_or(1.0) as i64;
            if !arg(a, 4).is_empty() {
                m.insert("clicked".into(), json!(arg(a, 4)));
            } else {
                m.insert("button".into(), json!(if arg(a, 2).is_empty() { "left".to_string() } else { arg(a, 2).to_lowercase() }));
                m.insert("count".into(), json!(count));
                m.insert("x".into(), num_arg(&arg(a, 0)));
                m.insert("y".into(), num_arg(&arg(a, 1)));
                if count > 1 {
                    plan = Some((false, "multi-click has no UI Automation equivalent".to_string()));
                }
            }
            opt_insert(&mut m, "app", &arg(a, 5));
        }
        "move" => {
            m.insert("x".into(), num_arg(&arg(a, 0)));
            m.insert("y".into(), num_arg(&arg(a, 1)));
            plan = Some((true, "moving the mouse cursor is real input".into()));
        }
        "drag" => {
            m.insert("from".into(), json!({"x": num_arg(&arg(a, 0)), "y": num_arg(&arg(a, 1))}));
            m.insert("to".into(), json!({"x": num_arg(&arg(a, 2)), "y": num_arg(&arg(a, 3))}));
            plan = Some((true, "dragging is real mouse input".into()));
        }
        "scroll" => {
            m.insert("dx".into(), num_arg(&arg(a, 2)));
            m.insert("dy".into(), num_arg(&arg(a, 3)));
            if !arg(a, 0).is_empty() {
                m.insert("x".into(), num_arg(&arg(a, 0)));
            }
            if !arg(a, 1).is_empty() {
                m.insert("y".into(), num_arg(&arg(a, 1)));
            }
            opt_insert(&mut m, "app", &arg(a, 4));
        }
        "type" => {
            m.insert("typed".into(), json!(arg(a, 0).chars().count()));
            opt_insert(&mut m, "app", &arg(a, 2));
        }
        "shortcut" => {
            m.insert("keys".into(), json!(arg(a, 0)));
            opt_insert(&mut m, "app", &arg(a, 2));
            if arg(a, 2).is_empty() {
                plan = Some((true, "without --app it is a system-wide shortcut, which only real input can press".into()));
            }
        }
        "clipboard-copy" => {
            opt_insert(&mut m, "app", &arg(a, 1));
        }
        "clipboard-paste" => {
            opt_insert(&mut m, "app", &arg(a, 1));
            if !arg(a, 0).is_empty() {
                m.insert("pasted".into(), json!(arg(a, 0)));
            }
        }
        "open" => {
            let (app, file, url) = (arg(a, 0), arg(a, 2), arg(a, 3));
            let opened = if !app.is_empty() { app.clone() } else if !url.is_empty() { url.clone() } else { file.clone() };
            m.insert("opened".into(), json!(opened));
            if !file.is_empty() {
                let abs = std::path::Path::new(&file);
                let abs = if abs.is_absolute() { abs.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(abs) };
                m.insert("file".into(), json!(path_str(&abs)));
            }
            opt_insert(&mut m, "url", &url);
            if parse_bool("hidden", &arg(a, 5), false) {
                m.insert("hidden".into(), json!(true));
            }
        }
        "focus" => {
            m.insert("focused".into(), json!(arg(a, 0)));
            m.insert("success".into(), json!(true));
            plan = Some((true, "focusing an app brings it to the front".into()));
        }
        "set" => {
            opt_insert(&mut m, "app", &arg(a, 0));
            m.insert("element".into(), json!({"ref": arg(a, 1)}));
        }
        "select" => {
            opt_insert(&mut m, "app", &arg(a, 0));
        }
        "menu" => {
            opt_insert(&mut m, "app", &arg(a, 0));
            m.insert("path".into(), json!(menu_path_parts(&arg(a, 1))));
        }
        "dialog" => {
            opt_insert(&mut m, "app", &arg(a, 0));
            m.insert("pressed".into(), json!(arg(a, 1)));
        }
        "window" => {
            let action = arg(a, 0).to_lowercase();
            m.insert("action".into(), json!(action));
            opt_insert(&mut m, "app", &arg(a, 1));
            match action.as_str() {
                "move" => {
                    m.insert("requested".into(), json!({"x": num_arg(&arg(a, 5)), "y": num_arg(&arg(a, 6))}));
                }
                "resize" => {
                    let mut r = Map::new();
                    if !arg(a, 5).is_empty() {
                        r.insert("width".into(), num_arg(&arg(a, 5)));
                    }
                    if !arg(a, 6).is_empty() {
                        r.insert("height".into(), num_arg(&arg(a, 6)));
                    }
                    m.insert("requested".into(), Value::Object(r));
                }
                "fullscreen" => {
                    m.insert("requested".into(), json!({"fullscreen": parse_bool("enabled", &arg(a, 5), true)}));
                }
                "raise" => {
                    plan = Some((true, "raising a window brings its app to the front".into()));
                }
                _ => {}
            }
        }
        _ => {}
    }
    (m, plan)
}

pub fn action_mode_index(command: &str) -> Option<usize> {
    match command {
        "click" => Some(6),
        "scroll" => Some(5),
        "type" | "shortcut" => Some(3),
        "clipboard-copy" | "clipboard-paste" => Some(2),
        "open" => Some(1),
        "set" => Some(5),
        "select" => Some(7),
        "menu" => Some(3),
        "dialog" => Some(2),
        "window" => Some(4),
        _ => None,
    }
}

pub fn action_name(command: &str, a: &[String]) -> String {
    command_display(command, a)
}

fn dry_run_action(command: &str, a: &[String]) -> ! {
    let (mut base, plan) = action_base(command, a);
    let mode = action_mode_index(command).map(|i| parse_mode(&arg(a, i))).unwrap_or(Mode::Auto);
    let name = action_name(command, a);
    if command == "open" {
        // open is background unless asked for the foreground (it never falls back).
        base.insert("dryRun".into(), json!(true));
        base.insert("mode".into(), json!(if mode == Mode::Foreground { "foreground" } else { "background" }));
        print_json(Value::Object(base));
    }
    let p = match plan {
        Some((true, reason)) => Plan::InherentlyForeground(reason),
        Some((false, reason)) => Plan::Unavailable(reason),
        None => Plan::Attempt(Box::new(|| Outcome::Done(Map::new()))),
    };
    run_action(&name, mode, base, p, &NoForeground, Box::new(|| Ok(Map::new())))
}
