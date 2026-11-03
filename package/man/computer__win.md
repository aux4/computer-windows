#### Description

Operate this Windows desktop: read the UI Automation tree, find and press elements, type, use menus and dialogs, take screenshots of covered windows, read text with OCR, control the clipboard, apps and windows. Actions run in the background by default (UI Automation patterns and the app's own control messages), falling back to real input only when needed. Every command prints JSON; errors are JSON on stderr with exit code 1.

#### Usage

```bash
aux4 computer win <command> [flags]
```

#### Example

```bash
aux4 computer win info
```

```json
{"provider":"win","contract":1,"platform":"windows/amd64","version":"0.1.1","capabilities":{"background":true,"ocr":true,"annotate":true}}
```
