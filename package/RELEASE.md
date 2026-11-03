# Release notes

First release of the Windows provider (`win`) for aux4/computer.

- The full aux4/computer command surface under `aux4 computer win`: `info`, `notes`, `permissions`, `screenshot` (with `--annotate`), `ocr`, `wait`, `read`, `find`, `state`, `click`, `move`, `drag`, `type`, `shortcut`, `scroll`, `cursor`, `screen`, `apps`, `windows`, `open`, `focus`, `quit`, `hide`, `unhide`, `menu`, `select`, `set`, `dialog`, `window list|move|resize|minimize|restore|close|fullscreen|raise`, `clipboard copy|paste|read`, `log`, `stop`, `resume`.
- Background by default: UI Automation patterns (Invoke, Toggle, SelectionItem, ExpandCollapse, Value, RangeValue, Text, Scroll) and the apps' own control messages, without moving the cursor or changing the foreground window; real input only as a fallback, with the cursor and foreground window put back.
- Screenshots of covered windows (Windows.Graphics.Capture, PrintWindow fallback) and OCR with Windows.Media.Ocr, in DPI-aware logical pixels.
- Clear errors for a missing interactive desktop and for elevated apps (UIPI).
- The shared stop switch and action log in `%LOCALAPPDATA%\aux4-computer`.
- Native binaries for windows/amd64 and windows/arm64.
