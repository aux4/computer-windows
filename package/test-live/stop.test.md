# live: stop switch

```timeout
60000
```

```beforeAll
aux4 computer win open notepad --wait 15s > /dev/null
```

```afterAll
aux4 computer win quit notepad --force true > /dev/null 2>&1; rm -rf state; true
```

## stopping a real app

```timeout
60000
```

### should stop

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win stop | jq -c '{stopped}'
```

```expect
{"stopped":true}
```

### should refuse to type into Notepad

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win type nope --app notepad 2>&1 | jq -c '{stopped}'
```

```expect
{"stopped":true}
```

### should leave Notepad untouched

```execute
aux4 computer win find nope --app notepad | jq '.matches | length'
```

```expect
0
```

### should resume and type again

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win resume > /dev/null; AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win type yes --app notepad | jq -r .mode
```

```expect
background
```

### should log all of it

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win log | jq -c '[.entries[] | {command, result}]'
```

```expect
[{"command":"stop","result":"ok"},{"command":"type","result":"error"},{"command":"resume","result":"ok"},{"command":"type","result":"ok"}]
```
