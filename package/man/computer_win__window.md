#### Description

Acts on one of an app's windows: `list`, `move`, `resize`, `minimize`, `restore`, `close`, `fullscreen` (maximize) and `raise`. Everything but `raise` runs in the background.

#### Usage

```bash
aux4 computer win window <list|move|resize|minimize|restore|close|fullscreen|raise> [flags]
```

#### Example

```bash
aux4 computer win window list notepad
```

```json
{"app":"notepad","pid":4120,"windows":[{"class":"Notepad","dialog":false,"focused":false,"frame":{"height":480,"width":640,"x":120,"y":90},"fullscreen":false,"id":262742,"index":0,"maximized":false,"minimized":false,"ref":"0","title":"Untitled - Notepad"}]}
```
