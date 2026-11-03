# aux4 computer win dialog

## dry run

### should press a button in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win dialog --app notepad --press "Don't Save"
```

```expect:json
{
  "app": "notepad",
  "dryRun": true,
  "mode": "background",
  "pressed": "Don't Save"
}
```
