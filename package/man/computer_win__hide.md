#### Description

Hides an app by minimizing all its top-level windows without activating anything (Windows has no app-level hide). `changed` says whether anything was minimized.

#### Usage

```bash
aux4 computer win hide <app>
```

--app        Application exe name, window title or pid

#### Example

```bash
aux4 computer win hide notepad
```

```json
{"app":"notepad","before":{"hidden":false},"changed":true,"hidden":true,"method":"ShowWindow(SW_SHOWMINNOACTIVE)","mode":"background","pid":4120}
```
