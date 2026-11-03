# aux4 computer win select

## validation

### should say what to select

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win select --app notepad
```

```error
{"error":"Say what to select: --all true, --text <text> [--occurrence N], or --start N --length N"}
```

### should take only one way

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win select --app notepad --all true --text hi
```

```error
{"error":"Give only one of --all, --text, or --start/--length"}
```

### should pair --occurrence with --text

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win select --app notepad --start 0 --occurrence 2
```

```error
{"error":"--occurrence goes with --text"}
```

## dry run

### should select in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win select --app notepad --all true
```

```expect:json
{
  "app": "notepad",
  "dryRun": true,
  "mode": "background"
}
```
