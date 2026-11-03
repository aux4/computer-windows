# aux4 computer win window resize

## validation

### should need a size

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window resize --app notepad
```

```error
{"error":"--width or --height is required"}
```

### should need a positive size

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window resize --app notepad --width 0
```

```error
{"error":"--width and --height must be positive"}
```

## dry run

### should resize in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window resize --app notepad --width 640 | jq -c '{action, mode, requested}'
```

```expect
{"action":"resize","mode":"background","requested":{"width":640}}
```
