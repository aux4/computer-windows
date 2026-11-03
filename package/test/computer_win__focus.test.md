# aux4 computer win focus

## dry run

### should always be foreground

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win focus notepad
```

```expect:json
{
  "dryRun": true,
  "focused": "notepad",
  "mode": "foreground",
  "success": true
}
```
