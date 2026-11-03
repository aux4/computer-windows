#### Description

Lists the on-screen (not minimized) top-level windows of every app, in z-order (topmost first): `app`, `pid`, `id` (window handle), `title`, `class` and `frame` in logical pixels.

#### Usage

```bash
aux4 computer win windows
```

#### Example

```bash
aux4 computer win windows
```

```json
{"windows":[{"app":"notepad","class":"Notepad","frame":{"height":480,"width":640,"x":120,"y":90},"id":262742,"pid":4120,"title":"Untitled - Notepad"}]}
```
