#### Description

Turns on the shared stop switch: a `STOP` file in `%LOCALAPPDATA%\aux4-computer` (or in `AUX4_COMPUTER_STATE_DIR`). While it exists every command that sends input or changes state refuses with `{"stopped": true, "error": ...}` and exit code 1, here and in every other aux4/computer provider; read-only commands keep working. `AUX4_COMPUTER_STOP=1` in an agent's environment stops it the same way. `aux4 computer stop` does the same.

#### Usage

```bash
aux4 computer win stop
```

#### Example

```bash
aux4 computer win stop
```

```json
{"file":"C:\\Users\\sally\\AppData\\Local\\aux4-computer\\STOP","stopped":true}
```
