#### Description

Reports the app's open message box or dialog (an owned window, or a `#32770` dialog): `kind` (`alert`, `dialog`, `save-panel`, `open-panel`), `title`, `text`, `buttons` (with `title`, `ref`, `default`, `cancel`, positions) and `fields` — or `null`. `--press` presses a button by title (case-insensitive, `&` mnemonics ignored) with UI Automation Invoke in the background and says whether the dialog `closed`.

#### Usage

```bash
aux4 computer win dialog [--app <name>] [--press <button>]
```

--app        App to check: exe name, window title or pid (default: the foreground app, then system alerts)
--press      Button to press, by title (case-insensitive, '&' mnemonics ignored, e.g. "Don't Save")
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win dialog --app notepad --press "Don't Save"
```

```json
{"app":"notepad","closed":true,"dialog":{"kind":"alert","title":"Notepad"},"kind":"alert","method":"Invoke","mode":"background","pressed":"Don't Save"}
```
