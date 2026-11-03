#### Description

Pastes the clipboard into the app's focused text field: in the background with the field's own paste (`WM_PASTE`) or by inserting the text, falling back to a real ctrl+v. With `--text`, that text is put on the clipboard first.

#### Usage

```bash
aux4 computer win clipboard paste [--text <text>] [--app <name>]
```

--text       Text to put on the clipboard before pasting; omit to paste the current clipboard contents
--app        App to paste into (default: the foreground app)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win clipboard paste --text " pasted" --app notepad
```

```json
{"app":"notepad","element":{"label":"Text Editor","ref":"0.0","role":"Edit"},"method":"WM_PASTE","mode":"background","pasted":" pasted"}
```
