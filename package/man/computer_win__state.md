#### Description

One call to see where things stand: the target `app` (name, pid, path, frontmost, hidden), the `frontmost` app, the app's `windows`, the `focused` element (shaped like a `find` match), the open `dialog` if any, and a compact list of the main window's actionable and text `elements`, each with a `ref`. Layout panes are skipped, `--depth` counts levels of meaningful elements and `--limit` caps the list (`truncated` and a `note` say when there's more).

#### Usage

```bash
aux4 computer win state [--app <name>] [--depth 3] [--limit 120]
```

--app        App to describe: exe name, window title or pid (default: the foreground app)
--depth      Levels of actionable/text elements to include (layout groups don't count; 0 = none) (default: 3)
--limit      Maximum number of elements in the compact tree (default: 120)

#### Example

```bash
aux4 computer win state --app notepad
```

```json
{"app":{"frontmost":false,"hidden":false,"name":"notepad","path":"C:\\Windows\\System32\\notepad.exe","pid":4120},"dialog":null,"elements":[{"frame":{"height":400,"width":624,"x":128,"y":148},"label":"Text Editor","ref":"0.0","role":"Edit"}],"focused":null,"frontmost":{"name":"WindowsTerminal","pid":3312},"windows":[{"class":"Notepad","dialog":false,"focused":false,"frame":{"height":480,"width":640,"x":120,"y":90},"fullscreen":false,"id":262742,"index":0,"maximized":false,"minimized":false,"ref":"0","title":"Untitled - Notepad"}]}
```
