# aux4 computer win move

## moving the cursor

### should require x and y

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win move --x 10
```

```error
{"error":"--x and --y are required"}
```

### should always be foreground

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win move --x 10 --y 20
```

```expect:json
{
  "dryRun": true,
  "mode": "foreground",
  "x": 10,
  "y": 20
}
```
