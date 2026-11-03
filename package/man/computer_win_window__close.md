#### Description

Closes a window (WM_CLOSE) without activating it, and reports whether it `closed`. If the app asks to save changes, the dialog is returned as `dialog` with a `note`: answer it with `dialog --app <name> --press <button>`. The window is picked by `--title` (exact, then contained, case-insensitive), `--index` (from `window list`) or the app's main window. It returns the window `before` and `after`.

#### Usage

```bash
aux4 computer win window close --app <name> [--title <t> | --index <n>]
```

--app        App whose window to use: exe name, window title or pid (default: the foreground app)
--title      Window title (exact, then contained; case-insensitive); default: the app's main window
--index      Window index from 'window list' (instead of --title)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win window close --app notepad
```

```json
{"action":"close","app":"notepad","closed":false,"dialog":{"buttons":[{"title":"Save"},{"title":"Don't Save"},{"title":"Cancel"}],"kind":"alert","title":"Notepad"},"method":"WM_CLOSE","mode":"background","note":"notepad asked before closing: answer it with 'aux4 computer win dialog --app notepad --press <button>'"}
```
