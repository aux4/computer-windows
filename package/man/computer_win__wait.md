#### Description

Blocks until text or an element appears — checking the app's UI Automation tree first, then its window's pixels with OCR when the tree has no match — or, with `--app` alone, until the app has an on-screen window. `--gone true` waits until it's no longer there. It returns as soon as the condition holds with how long it `waited`, the `source` that saw it (`ax`, `ocr` or `app`) and the same `matches` as `find`/`ocr`. Text matching ignores case, whitespace and commas.

On timeout it exits 1 with a JSON error naming the condition and `lastSeen` (the first texts from UI Automation and OCR, and whether the app was running). It always stops at `--timeout` and never moves the cursor or activates anything.

#### Usage

```bash
aux4 computer win wait [--text <text>] [--role Button] [--app <name>] [--gone true] [--source auto|ax|ocr] [--timeout 10s] [--interval 500ms]
```

--text       Text to wait for in element names/values or on the pixels (ignores case, whitespace and commas, so 1792 matches 1,792)
--role       Only elements with this UI Automation control type, e.g. Button; UI Automation only (macOS names like AXButton are accepted)
--app        App to watch (defaults to the foreground app / the main display); alone, wait until it is running with an on-screen window
--gone       true: wait until the text, element or app window is no longer there (default: false)
--source     auto (default): UI Automation first, OCR of the app window (or screen) when it has no match; ax (UI Automation) or ocr: only that one (default: auto)
--timeout    How long to wait before failing, e.g. 500ms, 10s, 2m (default: 10s)
--interval   Pause between checks, e.g. 250ms, 1s (checks never overlap) (default: 500ms)

#### Example

```bash
aux4 computer win wait --app Calculator --text 1792
```

```json
{"app":"Calculator","matches":[{"automationId":"CalculatorResults","center":{"x":516,"y":290},"enabled":true,"focused":false,"frame":{"height":40,"width":220,"x":406,"y":270},"label":"Display is 1,792","ref":"0.2.1","role":"Text","value":"","windowCenter":{"x":114,"y":67}}],"source":"ax","waited":1.62}
```
