#### Description

Moves a window so its visible top-left corner is at `--x`/`--y` (logical screen pixels), without activating it. The window is picked by `--title` (exact, then contained, case-insensitive), `--index` (from `window list`) or the app's main window. It returns the window `before` and `after`.

#### Usage

```bash
aux4 computer win window move --app <name> [--title <t> | --index <n>] --x <x> --y <y>
```

--app        App whose window to use: exe name, window title or pid (default: the foreground app)
--title      Window title (exact, then contained; case-insensitive); default: the app's main window
--index      Window index from 'window list' (instead of --title)
--x          Screen x of the window's top-left corner, logical pixels (negative: --x=-100)
--y          Screen y of the window's top-left corner
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win window move --app notepad --x 120 --y 90
```

```json
{"action":"move","after":{"frame":{"height":480,"width":640,"x":120,"y":90},"index":0,"title":"Untitled - Notepad"},"app":"notepad","before":{"frame":{"height":480,"width":640,"x":312,"y":160},"index":0,"title":"Untitled - Notepad"},"method":"SetWindowPos","mode":"background","requested":{"x":120,"y":90}}
```
