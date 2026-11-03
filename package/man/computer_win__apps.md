#### Description

Lists the running apps that have windows: `name` (exe name, the value `--app` takes), `pid`, `path`, `active` (it owns the foreground window), `hidden` (all its windows minimized) and the number of `windows`.

#### Usage

```bash
aux4 computer win apps
```

#### Example

```bash
aux4 computer win apps
```

```json
{"apps":[{"active":false,"hidden":false,"name":"notepad","path":"C:\\Windows\\System32\\notepad.exe","pid":4120,"windows":1}]}
```
