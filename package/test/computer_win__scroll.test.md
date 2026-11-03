# aux4 computer win scroll

## dry run

### should scroll in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win scroll --dy=-200 --app notepad
```

```expect:json
{
  "app": "notepad",
  "dryRun": true,
  "dx": 0,
  "dy": -200,
  "mode": "background"
}
```

### should error with --mode background when unavailable

```execute
AUX4_COMPUTER_DRY_RUN=1 AUX4_COMPUTER_SIMULATE_BACKGROUND=unavailable aux4 computer win scroll --dy=-200 --mode background
```

```error:partial
Background mode not possible for scroll*?
```
