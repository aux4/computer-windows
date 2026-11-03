#### Description

Asks an app to quit by closing its windows (WM_CLOSE, without activating it) and waits up to `--timeout` for it to exit. An app with unsaved changes may answer with a dialog: `quit` then returns it as `dialog` with `terminated: false` for you to answer with `dialog --press`. `--force true` kills the process (TerminateProcess; unsaved changes are lost). Quitting Explorer and other session processes, or this process, is refused.

#### Usage

```bash
aux4 computer win quit <app> [--force true] [--timeout 5s]
```

--app        Application exe name, window title or pid
--force      Force-quit with TerminateProcess (unsaved changes are lost) (default: false)
--timeout    How long to wait for the app to exit, e.g. 5s (default: 5s)

#### Example

```bash
aux4 computer win quit notepad
```

```json
{"app":"notepad","force":false,"method":"WM_CLOSE","mode":"background","path":"C:\\Windows\\System32\\notepad.exe","pid":4120,"terminated":true,"waited":0.21}
```
