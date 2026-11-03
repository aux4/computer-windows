#### Description

Minimizes a window to the taskbar without activating anything. The window is picked by `--title` (exact, then contained, case-insensitive), `--index` (from `window list`) or the app's main window. It returns the window `before` and `after`.

#### Usage

```bash
aux4 computer win window minimize --app <name> [--title <t> | --index <n>]
```

--app        App whose window to use: exe name, window title or pid (default: the foreground app)
--title      Window title (exact, then contained; case-insensitive); default: the app's main window
--index      Window index from 'window list' (instead of --title)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win window minimize --app notepad
```

```json
{"action":"minimize","after":{"minimized":true},"app":"notepad","method":"ShowWindow(SW_SHOWMINNOACTIVE)","mode":"background"}
```
