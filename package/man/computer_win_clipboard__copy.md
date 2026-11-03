#### Description

With `--text`, puts that text on the clipboard. Without it, copies the app's current selection — read in the background from the text field (`EM_GETSEL`) or the UI Automation Text pattern, falling back to a real ctrl+c — and returns what was copied.

#### Usage

```bash
aux4 computer win clipboard copy [--text <text>] [--app <name>]
```

--text       Text to put on the clipboard; omit to copy the current selection (ctrl+c)
--app        App whose selection to copy (default: the foreground app)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win clipboard copy --app notepad
```

```json
{"app":"notepad","method":"EM_GETSEL","mode":"background","text":"aux4"}
```
