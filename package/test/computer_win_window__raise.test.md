# aux4 computer win window raise

## dry run

### should always be foreground

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window raise --app notepad
```

```expect:json
{
  "action": "raise",
  "app": "notepad",
  "dryRun": true,
  "mode": "foreground"
}
```

### should refuse --mode background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window raise --app notepad --mode background
```

```error
{"error":"Background mode not possible for window raise: raising a window brings its app to the front"}
```
