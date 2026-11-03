# aux4 computer win window minimize

## dry run

### should minimize in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window minimize --app notepad
```

```expect:json
{
  "action": "minimize",
  "app": "notepad",
  "dryRun": true,
  "mode": "background"
}
```
