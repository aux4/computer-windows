# aux4 computer win read

## validation

### should reject a negative depth

```execute
aux4 computer win read --depth=-1
```

```error
{"error":"Invalid --depth '-1': use 0 or more"}
```

## an app that isn't running

### should fail with a JSON error

```execute
aux4 computer win read --app ThisAppDoesNotExist12345
```

```error
{"error":"App not running: ThisAppDoesNotExist12345"}
```
