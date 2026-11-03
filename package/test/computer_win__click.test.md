# aux4 computer win click

## validation

### should need a target

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click
```

```error
{"error":"Provide --x/--y or --element"}
```

### should reject an invalid mode

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click --x 1 --y 1 --mode bogus
```

```error
{"error":"Invalid --mode 'bogus': use auto, background or foreground"}
```

### should reject an invalid button

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click --x 1 --y 1 --button middle
```

```error
{"error":"Invalid --button 'middle': use left or right"}
```

## mode decision (dry run, never touches the screen)

### should act in the background by default

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click --x 100 --y 200
```

```expect:json
{
  "button": "left",
  "count": 1,
  "dryRun": true,
  "mode": "background",
  "x": 100,
  "y": 200
}
```

### should fall back to the foreground and say why

```execute
AUX4_COMPUTER_DRY_RUN=1 AUX4_COMPUTER_SIMULATE_BACKGROUND=unavailable aux4 computer win click --x 100 --y 200 --app notepad
```

```expect:json
{
  "app": "notepad",
  "button": "left",
  "count": 1,
  "dryRun": true,
  "fallbackReason": "background simulated as unavailable",
  "mode": "foreground",
  "x": 100,
  "y": 200
}
```

### should error instead of falling back with --mode background

```execute
AUX4_COMPUTER_DRY_RUN=1 AUX4_COMPUTER_SIMULATE_BACKGROUND=unavailable aux4 computer win click --x 100 --y 200 --mode background
```

```error
{"error":"Background mode not possible for click: background simulated as unavailable"}
```

### should use real input with --mode foreground, without a fallback reason

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click --x 100 --y 200 --mode foreground | jq -c '{mode, fallbackReason}'
```

```expect
{"mode":"foreground","fallbackReason":null}
```

### should fall back for a double-click

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click --x 100 --y 200 --count 2 | jq -c '{mode, fallbackReason}'
```

```expect
{"mode":"foreground","fallbackReason":"multi-click has no UI Automation equivalent"}
```

### should press an element by ref

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click --element 0.1.2 --app notepad
```

```expect:json
{
  "app": "notepad",
  "clicked": "0.1.2",
  "dryRun": true,
  "mode": "background"
}
```

## without a desktop

### should refuse even a dry run

```execute
AUX4_COMPUTER_DENY_PERMISSIONS=1 AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click --x 1 --y 1
```

```error:partial
Permission not granted*?
```
