# aux4 computer win window close

## dry run

### should close in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window close --app notepad
```

```expect:json
{
  "action": "close",
  "app": "notepad",
  "dryRun": true,
  "mode": "background"
}
```
