# aux4 computer win log

```beforeAll
rm -rf state state2
```

```afterAll
rm -rf state state2
```

## the action log

### should record a failed action with the contract fields

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win click --x 10 --y 20 --app ThisAppDoesNotExist12345 2>/dev/null; AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win log --limit 1 | jq -c '.entries[0] | {ts: (.ts | test("^20[0-9][0-9]-")), provider, command, app, x: .args.x, result, error}'
```

```expect
{"ts":true,"provider":"win","command":"click","app":"ThisAppDoesNotExist12345","x":"10","result":"error","error":"App not running: ThisAppDoesNotExist12345"}
```

### should truncate typed text to 16 characters plus its length

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win type 'a long and private sentence typed by an agent' --app ThisAppDoesNotExist12345 2>/dev/null; AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win log --limit 1 | jq -r '.entries[0].args.text'
```

```expect
a long and priva… (45 chars)
```

### should not record dry runs or read-only commands

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state AUX4_COMPUTER_DRY_RUN=1 aux4 computer win type hi --app notepad > /dev/null; AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win permissions > /dev/null; AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win log | jq '.entries | length'
```

```expect
2
```

### should reject an invalid limit

```execute
aux4 computer win log --limit 0
```

```error
{"error":"Invalid --limit '0': use a positive number"}
```
