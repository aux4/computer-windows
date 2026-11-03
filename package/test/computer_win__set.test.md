# aux4 computer win set

## validation

### should need an element

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win set --value x
```

```error
{"error":"--element is required: the element's ref from 'find' or 'state'"}
```

### should need one change

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win set --element 0.1
```

```error
{"error":"Say what to set: --value <text|number>, --checked true|false, or --option <title>"}
```

### should take only one change

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win set --element 0.1 --value a --option b
```

```error
{"error":"Give only one of --value, --checked, or --option"}
```

## dry run

### should set in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win set --app notepad --element 0.0 --value hello
```

```expect:json
{
  "app": "notepad",
  "dryRun": true,
  "element": {
    "ref": "0.0"
  },
  "mode": "background"
}
```
