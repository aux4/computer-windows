## Windows (aux4 computer win)

**Reaching apps.** There is no permission prompt on Windows. What matters is:

- An interactive, unlocked desktop: aux4 must run in a signed-in user's desktop session. From a Windows service, a non-interactive scheduled task or an SSH session, nothing can be seen or driven (`permissions` reports `desktop.granted: false`).
- Elevation (UIPI): a non-elevated process can't send input or UI Automation actions to an app running as administrator. Commands on such an app fail with `"uipi": true` and say so; run aux4 from an elevated terminal to drive it. `permissions` shows whether this process is elevated.

**Naming apps.** `--app` takes the exe name (`notepad`, `notepad.exe`), a friendly name for common apps (`Calculator`), a window title (`Untitled - Notepad`) or a pid. `apps` lists what's running with windows.

**Elements and roles.** `read`/`find`/`state` walk the UI Automation control view. Roles are UI Automation control types (`Button`, `Edit`, `Document`, `Text`, `CheckBox`, `ComboBox`, `MenuItem`, `ListItem`, `Window`...); `--role` also accepts the macOS names (`AXButton`, `AXTextField`, `AXStaticText`...). Refs are child-index paths starting with the window index from `window list` (`0.3.1`); re-read after the UI changes.

**What runs in the background** (no cursor movement, no activation):

- `click --element` / `click --x --y`: Invoke, Toggle, SelectionItem or ExpandCollapse on the element; a native control without a pattern gets a posted click message.
- `type`, `clipboard paste`, `set --value` on Win32 text fields (Notepad, dialogs): written through the control's own messages, so the app sees a real edit (undoable, marks the document modified). Other fields use the UI Automation Value pattern.
- `select`, `clipboard copy`: the text field's selection; TextPattern for rich controls.
- `menu --path` on apps with a classic menu bar: the menu command is sent directly, without opening the menu. Apps without one (modern UWP/XAML apps) are driven through their UI Automation menu, which may open it.
- `shortcut --app`: the menu item showing that shortcut, or ctrl+a/c/v/x/z on the text field; otherwise real keys (the app is brought forward, then the previous window is put back).
- `scroll`: the Scroll pattern, or a wheel message to the control.
- `window move|resize|minimize|restore|close`, `hide`/`unhide` (minimize/restore all windows without activating).
- `screenshot --app` and `ocr --app` capture a covered window (Windows.Graphics.Capture, PrintWindow fallback). A minimized window can't be captured: auto mode restores it without activating it first.

**Always foreground:** `focus`, `move`, `drag`, `window raise`, `shortcut` without `--app`, multi-clicks. `window fullscreen true` maximizes the window (Windows may activate it; the previous foreground window is put back).

**Coordinates** are logical pixels (per-monitor DPI aware): physical pixels divided by the display's `scale` from `screen`. With `--app`, x/y are relative to the app's main window frame (the visible frame, without the invisible resize borders), the same space as `windowCenter` in `find`/`ocr` and a `screenshot --app` image divided by its `scale`.

**OCR** uses Windows.Media.Ocr with the user's languages (each needs its Windows "Optical character recognition" language feature). It reports no per-line confidence. `--level` is accepted, but Windows has a single recognition level.

**Dialogs.** A "save changes?" prompt is an owned window of the app: `window close` and `quit` report it as `dialog` instead of closing; answer it with `dialog --app <name> --press "Don't Save"` (button titles ignore `&` mnemonics).

**Gotchas.**

- `hide` minimizes: Windows has no app-level hide, and a hidden app's windows are still listed by `window list` (as minimized).
- Text field selections are read with 16-bit offsets: selection-based commands are reliable on texts up to 65,535 characters.
- Calculator on Windows Server is the classic Win32 calculator; on Windows 10/11 it's the UWP app (hosted in ApplicationFrameHost, matched by `--app Calculator`).
- Pass negative numbers with `=`: `--dy=-200`, `--x=-100`.
