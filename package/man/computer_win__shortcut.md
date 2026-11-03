#### Description

Presses a key combo such as `ctrl+s`, `ctrl+shift+n`, `alt+f4`, `enter` or `escape` (modifiers: `ctrl`/`control` — `cmd` is read as ctrl —, `shift`, `alt`, `win`; keys: letters, digits, punctuation, `f1`–`f24`, `enter`, `tab`, `space`, `escape`, `backspace`, `delete`, arrows, `home`, `end`, `pageup`, `pagedown`).

With `--app` it runs in the background when it can: the app's menu item that shows that shortcut is sent directly (`method: menu`), and `ctrl+a`/`c`/`v`/`x`/`z` on a text field use the field's own commands; if the app is already in front the keys are sent as they are. Otherwise the app is brought forward, the keys are pressed and the previous window is put back. Without `--app` it is real input to whatever is in front.

#### Usage

```bash
aux4 computer win shortcut <keys> [--app <name>] [--delay 0]
```

--keys       Key combo, e.g. 'ctrl+c', 'ctrl+shift+s', 'alt+f4', 'enter', 'escape' (cmd is read as ctrl)
--delay      Milliseconds to wait before pressing (default: 0)
--app        Send the shortcut to this app (background when possible: its menu item, or the text field's own command); without it the shortcut is real input to the foreground window
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win shortcut ctrl+n --app notepad
```

```json
{"app":"notepad","keys":"ctrl+n","menuItem":"New","method":"menu","mode":"background"}
```
