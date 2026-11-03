#### Description

Presses an element by `--element` (a `ref` from `find`/`state`/`read`/`screenshot --annotate`) or whatever is at `--x`/`--y`. In the background it uses the element's UI Automation pattern — Invoke (buttons), Toggle (checkboxes), SelectionItem (list items, radio buttons, tabs), ExpandCollapse (combo boxes, menus) — or posts the click to the native control under the point (text fields get the caret there). The cursor doesn't move and nothing is activated; `action` says how it was pressed.

With `--app`, `--x`/`--y` are relative to the app's main window (logical pixels), the same space as `windowCenter` and a `screenshot --app` image divided by its `scale`; without it they're screen coordinates. Double-clicks (`--count 2`) always use real input.

#### Usage

```bash
aux4 computer win click --element <ref> [--app <name>]
aux4 computer win click --x <x> --y <y> [--app <name>] [--button left|right] [--count 1]
```

--x          X coordinate: relative to the app's window with --app (same as 'screenshot --app'), screen logical pixels without
--y          Y coordinate: relative to the app's window with --app (same as 'screenshot --app'), screen logical pixels without
--button     left or right (default: left)
--count      Click count (2 for double-click) (default: 1)
--element    Element ref from 'read'/'find'/'state' to press instead of x/y
--app        Target app; x/y become relative to its window (default: the foreground app, screen logical pixels)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win click --element 0.2.3.7 --app Calculator
```

```json
{"action":"Invoke","app":"Calculator","element":{"label":"Seven","ref":"0.2.3.7","role":"Button"},"mode":"background","screen":{"x":436,"y":464}}
```
