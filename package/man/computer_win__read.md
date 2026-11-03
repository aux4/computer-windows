#### Description

Walks the UI Automation control view of every window of the foreground app (or `--app`) and returns a flat list of `elements` with `ref`, `role` (UI Automation control type), `label` (Name), `value`, `frame`, `center`, `windowCenter`, `enabled`, `focused`, `depth` and, when present, `automationId`, `checked` and `selected`. A `ref` is a child-index path starting with the window index (`window list`); it's only valid for the UI as it is now.

#### Usage

```bash
aux4 computer win read [--app <name>] [--depth 10]
```

--app        App name to read (defaults to the foreground app)
--depth      Maximum tree depth to walk (default: 10)

#### Example

```bash
aux4 computer win read --app notepad --depth 2
```

```json
{"app":"notepad","elements":[{"depth":0,"enabled":true,"focused":false,"frame":{"height":480,"width":640,"x":120,"y":90},"label":"Untitled - Notepad","ref":"0","role":"Window","value":""},{"depth":1,"enabled":true,"focused":true,"frame":{"height":400,"width":624,"x":128,"y":148},"label":"Text Editor","ref":"0.0","role":"Edit","value":""}],"pid":4120}
```
