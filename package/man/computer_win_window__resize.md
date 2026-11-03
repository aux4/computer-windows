#### Description

Resizes a window's visible frame to `--width`/`--height` (logical pixels; give one to keep the other), without activating it. Apps may clamp the size. The window is picked by `--title` (exact, then contained, case-insensitive), `--index` (from `window list`) or the app's main window. It returns the window `before` and `after`.

#### Usage

```bash
aux4 computer win window resize --app <name> [--title <t> | --index <n>] [--width <w>] [--height <h>]
```

--app        App whose window to use: exe name, window title or pid (default: the foreground app)
--title      Window title (exact, then contained; case-insensitive); default: the app's main window
--index      Window index from 'window list' (instead of --title)
--width      New width in logical pixels (default: keep)
--height     New height in logical pixels (default: keep)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win window resize --app notepad --width 640 --height 480
```

```json
{"action":"resize","after":{"frame":{"height":480,"width":640,"x":120,"y":90}},"app":"notepad","method":"SetWindowPos","mode":"background","requested":{"height":480,"width":640}}
```
