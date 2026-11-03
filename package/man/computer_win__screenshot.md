#### Description

Captures a display (`--display`, index from `screen`) or an app's main window (`--app`) as a PNG. A window is captured with Windows.Graphics.Capture, so it works when other windows cover it (PrintWindow is the fallback; `method` says which was used); a minimized window is restored without activating it first. The image is resized down to `--maxWidth`, and `scale` is image pixels per logical pixel: divide a point in the image by `scale` to get `click --app` coordinates.

`--annotate true` draws a numbered box on each actionable element (`--roles all` adds text, images and headers) and returns `marks`: `n`, `ref`, `role`, `label`, `frame`, `center`, `windowCenter` and the `image` box. Press a mark with `click --element <ref> --app <name>`; past `--limit` marks the output says `truncated`.

#### Usage

```bash
aux4 computer win screenshot [--output file.png] [--display 0] [--app <name>] [--maxWidth 1568] [--annotate true] [--roles actionable|all] [--limit 150]
```

--output     File path to save the PNG to (defaults to a temp file)
--display    Display index to capture (from 'computer win screen') (default: 0)
--app        Capture this app's main window (even when covered by other windows) instead of a display
--maxWidth   Resize down if wider than this (image pixels), preserving aspect ratio (default: 1568)
--annotate   Draw a numbered box on each element (set-of-marks) and return the marks: number, ref, role, label, frame, center, windowCenter (default: false)
--roles      With --annotate: actionable (default: buttons, links, fields, checkboxes, menu items, combo boxes, tabs, list items) or all (also text, images, headers) (default: actionable)
--limit      With --annotate: most marks to draw (the rest are reported as truncated) (default: 150)

#### Example

```bash
aux4 computer win screenshot --app Calculator --output calc.png
```

```json
{"app":"Calculator","frame":{"height":322,"width":228,"x":402,"y":223},"height":322,"method":"wgc","path":"calc.png","scale":1,"width":228,"window":{"frame":{"height":322,"width":228,"x":402,"y":223},"id":131512,"ref":"0","title":"Calculator"}}
```
