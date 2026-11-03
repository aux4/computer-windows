# aux4 computer win window fullscreen

## dry run

### should maximize by default

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window fullscreen --app notepad | jq -c '{action, mode, requested}'
```

```expect
{"action":"fullscreen","mode":"background","requested":{"fullscreen":true}}
```

### should restore with false

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window fullscreen false --app notepad | jq -c .requested
```

```expect
{"fullscreen":false}
```
