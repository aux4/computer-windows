#### Description

Selects text in the app's focused text field (or `--element`): all of it, the `--occurrence`-th match of `--text` (case-insensitive), or `--start`/`--length` characters. It works in the background through the control's own selection (`EM_SETSEL`) or the UI Automation Text pattern, and returns the `selected` text and its `range`; `clipboard copy --app` then copies it.

#### Usage

```bash
aux4 computer win select --app <name> (--all true | --text <text> [--occurrence 1] | --start <n> --length <n>) [--element <ref>]
```

--app        App to select in: exe name, window title or pid (default: the foreground app)
--all        Select all the element's text (default: false)
--text       Select this text (case-insensitive)
--occurrence Which match of --text to select (1 = first)
--start      Start of the range to select (character offset from 0)
--length     Length of the range to select (characters)
--element    Text element ref from 'find' or 'state' (default: the app's focused element)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win select --app notepad --text aux4
```

```json
{"app":"notepad","element":{"label":"Text Editor","ref":"0.0","role":"Edit"},"method":"EM_SETSEL","mode":"background","range":{"length":4,"start":11},"selected":"aux4"}
```
