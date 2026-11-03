# aux4 computer win shortcut

## dry run

### should run in the background with --app

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win shortcut ctrl+a --app notepad
```

```expect:json
{
  "app": "notepad",
  "dryRun": true,
  "keys": "ctrl+a",
  "mode": "background"
}
```

### should be foreground without --app

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win shortcut alt+tab | jq -c '{mode, fallbackReason}'
```

```expect
{"mode":"foreground","fallbackReason":null}
```

### should refuse --mode background without --app

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win shortcut ctrl+c --mode background
```

```error
{"error":"Background mode not possible for shortcut: without --app it is a system-wide shortcut, which only real input can press"}
```

## validation

### should reject an unknown modifier

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win shortcut hyper+x
```

```error
{"error":"Unknown modifier: hyper"}
```

### should reject an unknown key

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win shortcut ctrl+nokey
```

```error
{"error":"Unknown key: nokey"}
```
