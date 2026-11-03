#### Description

Scrolls at a point of an app (`--x`/`--y` relative to its window with `--app`; the window's center by default). In the background it uses the UI Automation Scroll pattern of the scrollable element there (one small increment per ~40 px of `--dy`/`--dx`) or posts a wheel message to the control; `vertical`/`horizontal` report the new position (0–1). Positive `--dy` scrolls down. Pass negative values with `=`: `--dy=-200`.

#### Usage

```bash
aux4 computer win scroll [--x <x> --y <y>] [--dx 0] [--dy 0] [--app <name>]
```

--x          X coordinate to scroll at: relative to the app's window with --app, screen logical pixels without
--y          Y coordinate to scroll at: relative to the app's window with --app, screen logical pixels without
--dx         Horizontal scroll delta in pixels (default: 0)
--dy         Vertical scroll delta in pixels (default: 0)
--app        App to scroll (default: the foreground app)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win scroll --app notepad --dy=-400
```

```json
{"app":"notepad","dx":0,"dy":-400,"horizontal":0,"method":"ScrollPattern","mode":"background","screen":{"x":440,"y":330},"vertical":0}
```
