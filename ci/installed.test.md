# the published aux4/computer-windows

Runs the package as downloaded from the hub (not the build in this checkout).

```timeout
60000
```

```afterAll
aux4 computer win quit notepad --force true > /dev/null 2>&1; true
```

## provider info

### should be the win provider of contract 1

```execute
aux4 computer win info | jq -c '{provider, contract, platform}'
```

```expect:regex
\{"provider":"win","contract":1,"platform":"windows/(amd64|arm64)"\}
```

### should report the published version

```execute
aux4 computer win info | jq -r .version
```

```expect:regex
^\d+\.\d+\.\d+$
```

### should be accepted by aux4/computer

```execute
aux4 computer providers --provider win | jq -r '.providers[] | select(.provider == "win") | .status'
```

```expect
ok
```

## driving Notepad

### should open Notepad without activating it

```execute
aux4 computer win open notepad --wait 15s | jq -r .mode
```

```expect
background
```

### should type in the background

```execute
aux4 computer win type "installed from the hub" --app notepad | jq -r .mode
```

```expect
background
```

### should see the text

```execute
aux4 computer win wait --app notepad --text "installed from the hub" --source ax --timeout 5s | jq '.matches | length > 0'
```

```expect
true
```
