#### Description

Launches an app, or opens a file or URL (in `--app` or its default app), without activating it: the window is shown without taking the focus, and if the app brings itself to the front the previous foreground window is put back (`restored`). An app that is already running is left as it is (no second instance). `--wait` returns once a new window appears — the file's window with `--file` — with its `frame`; `--hidden true` launches it minimized. `--mode foreground` opens and activates it.

`--app` takes an exe name (`notepad`), a path, or a friendly name (`Calculator`).

#### Usage

```bash
aux4 computer win open [app] [--file <path>] [--url <url>] [--wait 10s] [--hidden true]
```

--app        Application exe name, window title or pid (com.apple.TextEdit); optional with --file/--url (their default app is used)
--file       File to open (in --app, or its default app)
--url        URL to open (in --app, or the default handler), e.g. https://aux4.io
--wait       Wait up to this long for the app's window (the file's window with --file) and return its frame, e.g. 10s
--hidden     Launch the app minimized, without activating it (default: false)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win open notepad --file notes.txt --wait 10s
```

```json
{"app":"notepad","file":"C:\\Users\\sally\\notes.txt","mode":"background","opened":"notepad","pid":5288,"waited":0.42,"window":{"frame":{"height":480,"width":640,"x":120,"y":90},"ref":"0","title":"notes.txt - Notepad"}}
```
