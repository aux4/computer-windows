# aux4 computer win window list

## an app that isn't running

### should fail with a JSON error

```execute
aux4 computer win window list ThisAppDoesNotExist12345
```

```error
{"error":"App not running: ThisAppDoesNotExist12345"}
```
