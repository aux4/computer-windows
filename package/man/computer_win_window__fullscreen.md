#### Description

Maximizes (`true`, the default) or restores (`false`) a window. Windows has no per-app full screen: a maximized window fills the display's work area. Maximizing may activate the window; the previous foreground window is then put back (`restored`). The window is picked by `--title` (exact, then contained, case-insensitive), `--index` (from `window list`) or the app's main window. It returns the window `before` and `after`.

#### Usage

```bash
aux4 computer win window fullscreen [true|false] --app <name> [--title <t> | --index <n>]
```

--app        App whose window to use: exe name, window title or pid (default: the foreground app)
--title      Window title (exact, then contained; case-insensitive); default: the app's main window
--index      Window index from 'window list' (instead of --title)
--enabled    true to maximize, false to restore (default: true)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win window fullscreen --app notepad
```

```json
{"action":"fullscreen","after":{"maximized":true},"app":"notepad","method":"ShowWindow(SW_MAXIMIZE)","mode":"background","requested":{"fullscreen":true}}
```
