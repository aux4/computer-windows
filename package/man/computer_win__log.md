#### Description

Shows the most recent entries of the shared action log (`%LOCALAPPDATA%\aux4-computer\actions.jsonl`, also read by `aux4 computer log`). Every command that sends input or changes state appends one line — `ts`, `provider` (`win`), `command`, `app`, `args` (typed text and values cut to 16 characters plus their length), `mode`, `method`, `result` and `error` — including attempts that failed or were refused. Dry runs and read-only commands aren't recorded. The file is rotated at 10MB.

#### Usage

```bash
aux4 computer win log [--limit 20]
```

--limit      Number of recent entries to show (default: 20)

#### Example

```bash
aux4 computer win log --limit 1
```

```json
{"entries":[{"app":"notepad","args":{"app":"notepad","text":"Hello from aux4"},"command":"type","method":"EM_REPLACESEL","mode":"background","ms":38,"provider":"win","result":"ok","ts":"2026-11-03T10:15:02.114Z"}],"path":"C:\\Users\\sally\\AppData\\Local\\aux4-computer\\actions.jsonl","stopped":false}
```
