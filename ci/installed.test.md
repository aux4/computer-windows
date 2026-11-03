# the published aux4/computer-windows

Runs the package as downloaded from the hub (not the build in this checkout).

```timeout
60000
```

```afterAll
aux4 computer win quit notepad --force true > /dev/null 2>&1; aux4 computer win quit Calculator --force true > /dev/null 2>&1; true
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
aux4 computer providers | jq -c '.providers[] | select(.provider == "win") | {status, default}'
```

```expect
{"status":"ok","default":true}
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

## 256 x 7 through the core's dispatch (aux4 computer ...)

### should open Calculator

```execute
aux4 computer open calc --wait 20s > /dev/null && aux4 computer wait --app Calculator --timeout 20s | jq -r .source
```

```expect
app
```

### should press clear

```execute
REF=$(aux4 computer find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "clearButton" or .label == "Clear")][0].ref') && aux4 computer click --element "$REF" --app Calculator | jq -r .mode
```

```expect
background
```

### should press 2

```execute
REF=$(aux4 computer find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "num2Button" or .label == "2" or .label == "Two")][0].ref') && aux4 computer click --element "$REF" --app Calculator | jq -r .mode
```

```expect
background
```

### should press 5

```execute
REF=$(aux4 computer find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "num5Button" or .label == "5" or .label == "Five")][0].ref') && aux4 computer click --element "$REF" --app Calculator | jq -r .mode
```

```expect
background
```

### should press 6

```execute
REF=$(aux4 computer find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "num6Button" or .label == "6" or .label == "Six")][0].ref') && aux4 computer click --element "$REF" --app Calculator | jq -r .mode
```

```expect
background
```

### should press multiply

```execute
REF=$(aux4 computer find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "multiplyButton" or .label == "Multiply" or .label == "Multiply by")][0].ref') && aux4 computer click --element "$REF" --app Calculator | jq -r .mode
```

```expect
background
```

### should press 7

```execute
REF=$(aux4 computer find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "num7Button" or .label == "7" or .label == "Seven")][0].ref') && aux4 computer click --element "$REF" --app Calculator | jq -r .mode
```

```expect
background
```

### should press equals

```execute
REF=$(aux4 computer find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "equalButton" or .label == "Equals")][0].ref') && aux4 computer click --element "$REF" --app Calculator | jq -r .mode
```

```expect
background
```

### should show 1792

```execute
aux4 computer wait --app Calculator --text 1792 --timeout 10s | jq '.matches | length > 0'
```

```expect
true
```
