# aux4/computer-windows

Computer-use for AI agents on Windows. A single native executable (no Python, .NET runtime or
PowerShell needed) that lets an agent see the screen and act on it: read the UI Automation tree, find
and press elements, type, use menus and dialogs, select text, set fields, scroll, take screenshots of
covered windows, read text with OCR, control the clipboard, and manage apps and windows. Every command
prints JSON, and every action is recorded in an action log that a stop switch can halt at once.

It is the Windows provider (`win`) of [aux4/computer](https://hub.aux4.io/r/public/packages/aux4/computer):
`aux4 computer <command>` runs these commands on Windows, and `aux4 computer win <command>` calls them
directly. It is built to **work alongside the person using the PC**: actions run in the background by
default — on an app that isn't in front, without moving the mouse cursor or changing the foreground
window — and fall back to real input only when the background isn't possible.

Ships native binaries for `windows/amd64` and `windows/arm64`.

## Installation

```bash
aux4 aux4 pkger install aux4/computer-windows
```

> **Known issue:** on Windows, `pkger install` can currently fail while pkger replaces its own executable, and
> aux4 needs `SHELL=bash` (Git Bash). Both are being fixed in aux4 itself; until then this package may not
> install or run on a stock Windows setup.

## What it can reach

```bash
aux4 computer win permissions
```

```json
{
  "desktop": { "granted": true, "guidance": "An interactive, unlocked desktop is available: UI Automation, input and screen capture can reach apps." },
  "elevated": { "granted": false, "integrity": "medium", "guidance": "This process is not elevated: Windows (UIPI) blocks UI Automation actions and input into apps that run as administrator; normal apps work. To drive an elevated app, run aux4 from an elevated terminal (Run as administrator)." },
  "uiAccess": { "granted": false, "guidance": "uiAccess is not set (it needs a signed binary installed under Program Files); only needed to drive elevated apps from a non-elevated process." }
}
```

Windows has no permission prompts for UI Automation or screen capture. Two things limit an agent:

- **An interactive desktop.** aux4 must run in a signed-in user's desktop session; from a Windows
  service, a non-interactive scheduled task or an SSH session, nothing can be seen or driven, and every UI
  command says so.
- **Elevation (UIPI).** A non-elevated process can't send input or UI Automation actions into an app that
  runs as administrator. Commands on such an app fail with `"uipi": true` and explain it; run aux4 from an
  elevated terminal to drive elevated apps.

## Background mode (default)

Every action takes `--mode`:

| Mode | Behavior |
|------|----------|
| `auto` (default) | Act in the background; if that isn't possible, fall back to real input, then put back the cursor and the previous foreground window |
| `background` | Act in the background only; error instead of falling back |
| `foreground` | Always use real mouse/keyboard input |

Every action's JSON says what happened: `"mode":"background"` or `"mode":"foreground"`, plus a
`fallbackReason` (and what was `restored`) when `auto` fell back, and the `method`/`action` used.

| Command | In the background | Falls back to real input when |
|---------|-------------------|-------------------------------|
| `click --element` / `--x --y` | UI Automation Invoke, Toggle, SelectionItem or ExpandCollapse on the element; a posted click for native controls | The element has no pattern and no native window; `--count 2` |
| `type` | Written through the text field's own messages (a real, undoable edit) or posted characters | The focused element isn't a native text control (e.g. a UWP field) |
| `clipboard copy` / `paste` | The text field's selection / its own paste command | The field doesn't expose its selection or paste |
| `shortcut --app` | The menu item showing that shortcut; ctrl+a/c/v/x/z on the text field; or the keys, if the app is already in front | No menu item or field command for it |
| `menu --path` | The menu command is sent directly (classic menu bars), without opening the menu | The item is disabled |
| `select` | The text field's selection, or the UI Automation Text pattern | The element takes neither |
| `set` | The field's own edit, or the Value / RangeValue / Toggle / SelectionItem pattern | The app refuses the change |
| `dialog --press` | UI Automation Invoke on the button | The button can't be invoked |
| `scroll` | The UI Automation Scroll pattern, or a wheel message to the control | Nothing scrollable at the point |
| `window move/resize/minimize/restore/close`, `hide`, `unhide` | Window messages that don't activate | — |
| `open` | Launched without activating; the foreground window is put back if the app takes it | — |
| `screenshot --app`, `ocr --app` | Captures the window even when covered | — (a minimized window is restored without activation) |

**Always foreground:** `focus`, `move`, `drag`, `window raise`, and `shortcut` without `--app`.

### Coordinates

Coordinates are **logical pixels** (per-monitor DPI aware: physical pixels divided by the display's
`scale` from `screen`). With `--app`, `--x`/`--y` are relative to the app's main window — the same space
as `windowCenter` in `find`, `ocr` and `screenshot --annotate`, and a `screenshot --app` image divided by
its `scale`. Without `--app`, they're screen coordinates.

```bash
aux4 computer win find Seven --role Button --app Calculator
aux4 computer win click --app Calculator --x 34 --y 241
```

```json
{"action":"Invoke","app":"Calculator","element":{"label":"Seven","ref":"0.2.3.7","role":"Button"},"mode":"background","screen":{"x":436,"y":464}}
```

## Commands

### Seeing the screen

```bash
aux4 computer win screenshot [--output file.png] [--display 0] [--app Calculator] [--maxWidth 1568]
aux4 computer win screenshot --app Calculator --annotate true [--roles actionable|all] [--limit 150]
aux4 computer win ocr [--app Calculator] [--image file.png] [--text 1792] [--languages en-US] [--timeout 60s]
aux4 computer win screen
aux4 computer win cursor
```

`screenshot` captures a display or an app's window even when it's covered (`method` is `wgc` for
Windows.Graphics.Capture or `printwindow`), resized down to `--maxWidth`. `--annotate true` draws a
numbered box on each actionable element and returns `marks` (`n`, `ref`, `role`, `label`, `frame`,
`center`, `windowCenter`, `image`): act on a mark with `click --element <ref> --app <name>`.

`ocr` reads text on the device with Windows.Media.Ocr: each line in `matches` has `text`, `frame` and
`center` (plus `windowCenter` with `--app`); `--text` keeps the lines containing it (ignoring case,
whitespace and commas, so `1792` matches `1,792`) and adds the position of just that text as `match`.

### Reading the UI

```bash
aux4 computer win state [--app notepad] [--depth 3] [--limit 120]
aux4 computer win read [--app notepad] [--depth 10]
aux4 computer win find "Save" [--role Button] [--app notepad]
aux4 computer win wait [--text 1792] [--role Button] [--app Calculator] [--gone true] [--source auto|ax|ocr] [--timeout 10s]
```

`read`, `find` and `state` walk the UI Automation control view. Roles are UI Automation control types
(`Button`, `Edit`, `Document`, `Text`, `CheckBox`, `ComboBox`, `MenuItem`, `ListItem`, `Window`...);
`--role` also accepts the macOS names (`AXButton`, `AXTextField`, `AXStaticText`...). `find` matches a
case-insensitive substring of an element's name or value, or its exact automation id. A `ref` is a
child-index path starting with the window index from `window list` (`0.2.3.7`): it's valid for the UI as
it is now, so read again after it changes.

`state` returns the app, the foreground app, the app's windows, the focused element, the open dialog
and a compact list of actionable and text elements. `wait` returns as soon as text, an element or an
app window appears (UI Automation first, then OCR), or exits 1 with what it saw at `--timeout`.

### Acting on the UI

```bash
aux4 computer win click --element 0.2.3.7 --app Calculator
aux4 computer win click --x 100 --y 200 [--app Calculator] [--button left|right] [--count 2]
aux4 computer win type "Hello from aux4" [--app notepad] [--delay 300]
aux4 computer win shortcut ctrl+s [--app notepad]
aux4 computer win scroll [--x 100 --y 200] --dy=-200 [--app notepad]
aux4 computer win move --x 100 --y 200
aux4 computer win drag --x 100 --y 200 --toX 400 --toY 200
```

### Menus, selections, fields and dialogs

```bash
aux4 computer win menu [--app notepad] [--depth 2]
aux4 computer win menu --app notepad --path "Format > Word Wrap"
aux4 computer win select --app notepad --all true
aux4 computer win select --app notepad --text aux4 [--occurrence 2]
aux4 computer win set --app notepad --element 0.0 --value "new text"
aux4 computer win set --app myapp --element 0.4 --checked true
aux4 computer win set --app myapp --element 0.6 --option "Large"
aux4 computer win dialog [--app notepad]
aux4 computer win dialog --app notepad --press "Don't Save"
```

- `menu` lists the menu bar (`title`, `ref`, `shortcut`, `checked`, `children`) or presses an item by path;
  titles match case-insensitively, ignoring `&` mnemonics and a trailing `...`.
- `select` returns the `selected` text and its `range`; `clipboard copy --app` then copies it.
- `set` returns the value `before` and `after`.
- `dialog` reports the open message box or dialog — `kind`, `title`, `text`, `buttons`, `fields` — or
  `null`; `--press` presses a button and says whether the dialog `closed`.

Text written to Win32 text fields (`type`, `set --value`, `clipboard paste`) goes through the field's own
editing messages, so the app sees a real edit: it can be undone and it marks the document as modified.

### Clipboard

```bash
aux4 computer win clipboard copy [--text "..."] [--app notepad]
aux4 computer win clipboard paste [--text "..."] [--app notepad]
aux4 computer win clipboard read
```

Text only.

### Apps and windows

```bash
aux4 computer win apps
aux4 computer win windows
aux4 computer win open notepad
aux4 computer win open notepad --file notes.txt --wait 10s
aux4 computer win open --url https://aux4.io
aux4 computer win focus notepad
aux4 computer win window list notepad
aux4 computer win window move --app notepad [--title notes | --index 0] --x 200 --y 150
aux4 computer win window resize --app notepad --width 800 [--height 600]
aux4 computer win window minimize|restore|close|raise --app notepad
aux4 computer win window fullscreen true|false --app notepad
aux4 computer win hide notepad
aux4 computer win unhide notepad
aux4 computer win quit notepad [--force true] [--timeout 5s]
```

`--app` takes the exe name (`notepad`, `notepad.exe`), a friendly name (`Calculator`), a window title or
a pid. `open` launches without activating and leaves an already-running app as it is. `window close` and
`quit` report a "save changes?" prompt as `dialog` instead of closing, for you to answer with
`dialog --press`; `quit --force true` kills the process. Quitting Explorer and other session processes is
refused.

## Action log and stop switch

```bash
aux4 computer win log [--limit 20]
aux4 computer win stop
aux4 computer win resume
```

The stop switch and the action log are shared with `aux4/computer` and its other providers, in
`%LOCALAPPDATA%\aux4-computer` (or `AUX4_COMPUTER_STATE_DIR`). Every command that sends input or changes
state appends one JSON line to `actions.jsonl` — time, provider, command, app, arguments (typed text cut to
16 characters plus its length), mode, method and result or error — including refused attempts; the file is
rotated at 10MB. `stop` (or `aux4 computer stop`) creates a `STOP` file: every input or state command then
refuses with a JSON error until `resume`, while read-only commands keep working. Setting
`AUX4_COMPUTER_STOP=1` in an agent's environment stops it the same way.

```json
{"command":"click","error":"Stopped: aux4 computer win is not sending input or changing state (C:\\Users\\sally\\AppData\\Local\\aux4-computer\\STOP exists). Run 'aux4 computer win resume' (or 'aux4 computer resume') to allow it again.","stopped":true}
```

## Provider info and notes

```bash
aux4 computer win info
aux4 computer win notes
```

`info` describes the provider for `aux4/computer` (name `win`, contract `1`, platform, version,
capabilities, permissions). `notes` prints the Windows specifics for agents; the generic computer-use
skill of `aux4/computer` includes them.

## Platform differences

Compared with the macOS provider (`aux4/computer-mac`):

- **Roles** are UI Automation control types (`Button`, `Edit`, `Text`...); macOS role names are accepted by `--role`.
- **Coordinates** are logical pixels (DPI-scaled) rather than points; the meaning is the same.
- **`--app`** takes exe names, friendly names, window titles or pids instead of bundle ids. `apps` reports a `path` instead of a `bundleId`.
- **`hide`** minimizes the app's windows: Windows has no app-level hide.
- **`window fullscreen`** maximizes: Windows has no per-app full screen.
- **`ocr`** has no per-line `confidence`, and one recognition level (`--level` is accepted).
- **Text edits** through `type`, `set --value` and `clipboard paste` on Win32 text fields are real edits that mark the document modified, so there is no silent data-loss warning.
- **`menu`** on apps without a classic menu bar (UWP/XAML) lists only the top level, and pressing an item opens the menu through UI Automation.
- **Permissions** are the interactive desktop, elevation (UIPI) and uiAccess, instead of Accessibility and Screen Recording.

## Output and errors

Every command prints a single JSON object to stdout. On failure (bad arguments, no desktop, an elevated
target, element not found, app not running) it prints `{"error": "..."}` to stderr and exits 1.

## Development notes

- **Language: Rust** with the `windows` crate. It covers everything the provider needs in one place —
  UI Automation (COM), Windows.Graphics.Capture and Direct3D, Windows.Media.Ocr (WinRT), SendInput,
  the clipboard and window messages — and builds a single self-contained `.exe` per architecture
  (static CRT, no runtime to install). C# with NativeAOT was the alternative: UI Automation is as easy
  there, but WinRT interop in trimmed NativeAOT builds is fragile and the toolchain can't build Windows
  binaries outside Windows. Go was ruled out: UI Automation through raw COM vtables has no maintained
  bindings. The platform-independent parts (argument validation, dry runs, the stop switch and log)
  also compile on macOS and Linux, so `cargo test` and most `.test.md` files run anywhere, and
  `cargo check --target x86_64-pc-windows-msvc` type-checks the Windows code from any OS.
- Build (on Windows, MSVC): `aux4 builder windows` (amd64 + arm64 into `package/dist/windows/`).
  Elsewhere: `aux4 builder local` links a host build into `package/` for the platform-independent tests.
- CI (`.github/workflows/windows.yml`) builds on `windows-latest`, runs the unit tests, the package tests
  and the aux4/computer conformance suite on `windows-latest` (amd64) and `windows-11-arm` (arm64), and a
  live UI test on both: Calculator and Notepad driven in the background. Hosted Windows runners have an
  interactive, elevated desktop session where UI Automation, SendInput, screen capture and OCR work.
