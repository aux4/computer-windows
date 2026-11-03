#### Description

Lists an app's menu bar as a tree (`title`, `ref`, `shortcut`, `checked`, `enabled: false`, `children` or `submenu`), or presses an item by `--path` (titles separated by `>`, case-insensitive; `&` mnemonics and a trailing `...` don't matter). For apps with a classic menu bar (Notepad, Paint, most Win32 apps) the item's command is sent directly — no menu opens, nothing is activated (`method: WM_COMMAND`); a disabled item falls back to the foreground. Apps without one (UWP/XAML) are driven through their UI Automation menu (`source: uia`), which opens it. When a title doesn't match, the error lists what's available at that level.

#### Usage

```bash
aux4 computer win menu [--app <name>] [--depth 2]
aux4 computer win menu --app <name> --path "File > New"
```

--app        App whose menu bar to use: exe name, window title or pid (default: the foreground app)
--path       Menu item to press, titles separated by '>' (case-insensitive, '&' mnemonics and trailing '...' optional); omit to list the menu bar
--depth      Menu levels to list: 1 = menu titles, 2 = their items (default), 3+ = submenus (default: 2)
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win menu --app notepad --path "Format > Word Wrap"
```

```json
{"app":"notepad","method":"WM_COMMAND","mode":"background","path":["Format","Word Wrap"],"pressed":"Word Wrap","ref":"menu:32"}
```
