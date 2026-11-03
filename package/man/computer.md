#### Description

The `computer` group holds computer-use commands for AI agents. `aux4/computer` provides the generic commands that pick the provider for this OS; `aux4/computer-windows` adds the Windows provider under `aux4 computer win`.

#### Usage

```bash
aux4 computer win <command>
```

#### Example

```bash
aux4 computer win state --app notepad
```

```json
{"app":{"name":"notepad","pid":4120,"frontmost":false,"hidden":false},"windows":[{"index":0,"title":"Untitled - Notepad"}]}
```
