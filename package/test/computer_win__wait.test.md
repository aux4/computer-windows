# aux4 computer win wait

## validation

### should need something to wait for

```execute
aux4 computer win wait
```

```error
{"error":"Nothing to wait for: give --text (optionally with --role), --role, or --app"}
```

### should reject --role with --source ocr

```execute
aux4 computer win wait --role Button --source ocr
```

```error
{"error":"--role needs the accessibility tree: use --source auto or ax with --role"}
```

### should reject an invalid timeout

```execute
aux4 computer win wait --text x --timeout soon
```

```error
{"error":"Invalid --timeout 'soon': use e.g. 500ms, 10s or 2m"}
```

## an app that never shows up

### should be gone at once

```execute
aux4 computer win wait --app ThisAppDoesNotExist12345 --gone true | jq -c '{matches, gone}'
```

```expect
{"matches":[],"gone":true}
```

### should time out with a JSON error

```execute
aux4 computer win wait --app ThisAppDoesNotExist12345 --timeout 1s 2>&1 | jq -c '{error: (.error | startswith("Timed out after 1s")), running: .lastSeen.running}'
```

```expect
{"error":true,"running":false}
```
