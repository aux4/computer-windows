#### Description

Brings a window to the front (SetForegroundWindow); always foreground, so `--mode background` is refused. The window is picked by `--title` (exact, then contained, case-insensitive), `--index` (from `window list`) or the app's main window. It returns the window `before` and `after`.

#### Usage

```bash
aux4 computer win window raise --app <name> [--title <t> | --index <n>]
```

--app        App whose window to use: exe name, window title or pid (default: the foreground app)
--title      Window title (exact, then contained; case-insensitive); default: the app's main window
--index      Window index from 'window list' (instead of --title)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win window raise --app notepad
```

```json
{"action":"raise","after":{"focused":true},"app":"notepad","method":"SetForegroundWindow","mode":"foreground"}
```
