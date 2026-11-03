# aux4 computer win type

## dry run

### should type in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win type hello --app notepad
```

```expect:json
{
  "app": "notepad",
  "dryRun": true,
  "mode": "background",
  "typed": 5
}
```

### should count characters, not bytes

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win type 'olá 👋' --app notepad | jq .typed
```

```expect
5
```

### should fall back to real input

```execute
AUX4_COMPUTER_DRY_RUN=1 AUX4_COMPUTER_SIMULATE_BACKGROUND=unavailable aux4 computer win type hello | jq -c '{mode, fallbackReason}'
```

```expect
{"mode":"foreground","fallbackReason":"background simulated as unavailable"}
```

## validation

### should reject a negative delay

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win type hi --delay=-5
```

```error
{"error":"Invalid --delay '-5': use 0 or more"}
```
