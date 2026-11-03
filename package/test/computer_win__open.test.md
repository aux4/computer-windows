# aux4 computer win open

## validation

### should need an app, file or URL

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win open
```

```error
{"error":"--app is required (or give --file or --url)"}
```

### should take a file or a URL, not both

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win open --file a.txt --url https://aux4.io
```

```error
{"error":"Give either --file or --url, not both"}
```

### should need a URL scheme

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win open --url aux4.io
```

```error
{"error":"Invalid --url 'aux4.io': include the scheme, e.g. https://aux4.io"}
```

### should not combine --hidden with the foreground

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win open notepad --hidden true --mode foreground
```

```error
{"error":"--hidden true opens the app without showing it; it can't be combined with --mode foreground"}
```

## dry run

### should open in the background

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win open notepad
```

```expect:json
{
  "dryRun": true,
  "mode": "background",
  "opened": "notepad"
}
```

### should open a URL

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win open --url https://aux4.io | jq -c '{opened, url, mode}'
```

```expect
{"opened":"https://aux4.io","url":"https://aux4.io","mode":"background"}
```
