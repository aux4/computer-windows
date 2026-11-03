#### Description

Lists all of an app's top-level windows, including minimized ones and dialogs it owns, in z-order: `index`, `ref` (the first part of element refs), `id`, `title`, `class`, `frame`, `minimized`, `maximized` (`fullscreen`), `dialog` and `focused`.

#### Usage

```bash
aux4 computer win window list [app]
```

--app        App whose windows to list: exe name, window title or pid (default: the foreground app)

#### Example

```bash
aux4 computer win window list notepad
```

```json
{"app":"notepad","pid":4120,"windows":[{"class":"Notepad","dialog":false,"focused":false,"frame":{"height":480,"width":640,"x":120,"y":90},"fullscreen":false,"id":262742,"index":0,"maximized":false,"minimized":false,"ref":"0","title":"Untitled - Notepad"}]}
```
