#### Description

Describes this provider for `aux4/computer`: the provider name (`win`), the contract version it implements (`1`), the platform (`windows/amd64` or `windows/arm64`), the package version, a `capabilities` map of booleans and the `permissions` report. `aux4 computer providers` uses it, and refuses a provider whose contract differs from the core's. It also shows the shared stop file and action log paths.

#### Usage

```bash
aux4 computer win info
```

#### Example

```bash
aux4 computer win info
```

```json
{"capabilities":{"annotate":true,"background":true,"clipboardText":true,"dialogs":true,"foregroundFallback":true,"fullscreen":false,"hide":false,"logicalCoordinates":true,"menus":true,"ocr":true,"screenshotCoveredWindow":true,"windowOps":true},"contract":1,"logFile":"C:\\Users\\sally\\AppData\\Local\\aux4-computer\\actions.jsonl","permissions":{"desktop":{"granted":true,"guidance":"..."},"elevated":{"granted":false,"guidance":"..."},"uiAccess":{"granted":false,"guidance":"..."}},"platform":"windows/amd64","provider":"win","stopFile":"C:\\Users\\sally\\AppData\\Local\\aux4-computer\\STOP","version":"0.1.1"}
```
