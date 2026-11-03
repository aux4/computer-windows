# aux4 computer win menu

## validation

### should need an item, not just a menu

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win menu --app notepad --path File
```

```error
{"error":"Invalid --path 'File': give a menu item, not just a menu, e.g. 'File > ...'"}
```

### should reject an empty title

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win menu --app notepad --path 'File > > New'
```

```error
{"error":"Invalid --path 'File > > New': separate menu titles with '>', e.g. 'Format > Font > Bold'"}
```

## dry run

### should press in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win menu --app notepad --path 'File > New'
```

```expect:json
{
  "app": "notepad",
  "dryRun": true,
  "mode": "background",
  "path": [
    "File",
    "New"
  ]
}
```
