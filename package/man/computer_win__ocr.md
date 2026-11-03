#### Description

Recognizes text with Windows.Media.Ocr (on the device, no network) in the main display, an app's main window (`--app`, even when covered) or an existing PNG (`--image`). Each line is an entry in `matches` with `text`, a `frame` and a `center` in logical screen pixels, plus a `windowCenter` with `--app` (the space `click --app` uses). `--text` keeps only the lines containing that text, ignoring case, whitespace and commas (`1792` matches `1,792`), and adds a `match` with the position of just that text.

Windows OCR reports no confidence and has one recognition level (`--level` is accepted for compatibility). `--languages` picks OCR languages (each needs its Windows OCR language feature); the default is the user's profile languages. `ocr` always stops at `--timeout` with a JSON error.

#### Usage

```bash
aux4 computer win ocr [--app <name>] [--image file.png] [--text <text>] [--languages en-US] [--timeout 60s]
```

--app        Read this app's main window (even when covered by other windows); positions are also given relative to it as windowCenter, the space 'click --app' uses
--image      Recognize an existing PNG instead of capturing the screen
--text       Only return lines containing this text (case-insensitive); each line also gets a 'match' with the position of just that text
--level      accurate (default) or fast; Windows OCR has a single level, both run it (default: accurate)
--languages  Comma-separated OCR languages in priority order, e.g. en-US,pt-BR (default: the user's profile languages; each needs its Windows OCR language feature)
--scale      With --image: image pixels per logical pixel (the 'scale' from 'screenshot'), so positions come out in logical pixels (default: 1)
--timeout    Give up with a JSON error after this long, e.g. 30s, 2m (default 60s) (default: 60s)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win ocr --app Calculator --text 1792
```

```json
{"app":"Calculator","height":322,"language":"en-US","level":"accurate","matches":[{"center":{"x":596,"y":290},"frame":{"height":17,"width":38,"x":577,"y":282},"match":{"center":{"x":596,"y":290},"frame":{"height":17,"width":38,"x":577,"y":282},"windowCenter":{"x":194,"y":67}},"text":"1792","windowCenter":{"x":194,"y":67}}],"method":"wgc","mode":"background","scale":1,"source":"window","text":"1792","width":228}
```
