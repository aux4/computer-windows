# aux4 computer win quit

## refusals

### should refuse to quit explorer

```execute
aux4 computer win quit explorer 2>&1 | jq -r '.error | startswith("Refusing to quit explorer")'
```

```expect
true
```

## an app that isn't running

### should fail with a JSON error

```execute
aux4 computer win quit ThisAppDoesNotExist12345
```

```error
{"error":"App not running: ThisAppDoesNotExist12345"}
```
