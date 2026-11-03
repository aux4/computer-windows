# aux4 computer win find

## an app that isn't running

### should fail with a JSON error

```execute
aux4 computer win find Save --app ThisAppDoesNotExist12345
```

```error
{"error":"App not running: ThisAppDoesNotExist12345"}
```

## without a desktop

### should error with guidance

```execute
AUX4_COMPUTER_DENY_PERMISSIONS=1 aux4 computer win find Save
```

```error:partial
Permission not granted*?
```
