#### Description

Restores an app's minimized windows without activating them.

#### Usage

```bash
aux4 computer win unhide <app>
```

--app        Application exe name, window title or pid

#### Example

```bash
aux4 computer win unhide notepad
```

```json
{"app":"notepad","before":{"hidden":true},"changed":true,"hidden":false,"method":"ShowWindow(SW_SHOWNOACTIVATE)","mode":"background","pid":4120}
```
