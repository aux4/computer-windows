# aux4 computer win window restore

## dry run

### should restore in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window restore --app notepad
```

```expect:json
{
  "action": "restore",
  "app": "notepad",
  "dryRun": true,
  "mode": "background"
}
```
