# aux4 computer win clipboard paste

## dry run

### should paste in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win clipboard paste --text hello --app notepad | jq -c '{app, dryRun, mode}'
```

```expect
{"app":"notepad","dryRun":true,"mode":"background"}
```
