# aux4 computer win window move

## validation

### should need x and y

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window move --app notepad --x 10
```

```error
{"error":"--x and --y are required (screen points of the window's top-left corner)"}
```

### should take a title or an index, not both

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window move --app notepad --title a --index 0 --x 1 --y 2
```

```error
{"error":"Give either --title or --index, not both"}
```

## dry run

### should move in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window move --app notepad --x 100 --y 80
```

```expect:json
{
  "action": "move",
  "app": "notepad",
  "dryRun": true,
  "mode": "background",
  "requested": {
    "x": 100,
    "y": 80
  }
}
```

### should take negative coordinates

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win window move --app notepad --x=-200 --y 80 | jq -c .requested
```

```expect
{"x":-200,"y":80}
```
