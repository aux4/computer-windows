#### Description

Types unicode text (emoji and non-ASCII included) into the app's focused text field. In the background it writes through the control's own messages for Win32 text fields (Notepad, dialog fields: `method: EM_REPLACESEL`, a real, undoable edit that marks the document modified) or posts the characters to other native controls (`WM_CHAR`); otherwise it falls back to real keystrokes (`SendInput`). `--delay` waits before typing.

#### Usage

```bash
aux4 computer win type <text> [--app <name>] [--delay 0]
```

--text       Text to type
--delay      Milliseconds to wait before typing (e.g. for a panel to appear) (default: 0)
--app        App to type into (default: the foreground app)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win type "Hello from aux4" --app notepad
```

```json
{"app":"notepad","element":{"label":"Text Editor","ref":"0.0","role":"Edit"},"method":"EM_REPLACESEL","mode":"background","typed":15}
```
