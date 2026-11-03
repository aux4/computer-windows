# aux4 computer win stop

```beforeAll
rm -rf state state2
```

```afterAll
rm -rf state state2
```

## the stop switch file

### should create the STOP file in the state directory

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win stop | jq -c '{stopped, file: (.file | test("state.STOP$"))}'; test -f state/STOP && echo exists
```

```expect
{"stopped":true,"file":true}
exists
```

### should refuse input commands while it exists

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state AUX4_COMPUTER_DRY_RUN=1 aux4 computer win click --x 1 --y 1 2>&1 | jq -c '{stopped, command, says: (.error | test("^Stopped: .*STOP exists.*resume"))}'
```

```expect
{"stopped":true,"command":"click","says":true}
```

### should exit with code 1 when refused

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win type hi 2>/dev/null; echo "exit=$?"
```

```expect
exit=1
```

### should name the subcommand when refusing

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state AUX4_COMPUTER_DRY_RUN=1 aux4 computer win clipboard paste --app notepad 2>&1 | jq -c '{stopped, command}'
```

```expect
{"stopped":true,"command":"clipboard paste"}
```

### should record refusals in the action log

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win log | jq -c '{stopped, commands: [.entries[].command], results: ([.entries[].result] | unique), provider: ([.entries[].provider] | unique)}'
```

```expect
{"stopped":true,"commands":["stop","click","type","clipboard paste"],"results":["error","ok"],"provider":["win"]}
```

### should keep read-only commands working

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state AUX4_COMPUTER_DENY_PERMISSIONS=1 aux4 computer win permissions | jq -c '[.[] | .granted] | unique'
```

```expect
[false]
```

## the environment switch

### should refuse input with AUX4_COMPUTER_STOP=1

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state2 AUX4_COMPUTER_STOP=1 AUX4_COMPUTER_DRY_RUN=1 aux4 computer win shortcut ctrl+c --app notepad 2>&1 | jq -c '{stopped, says: (.error | test("AUX4_COMPUTER_STOP=1 is set"))}'
```

```expect
{"stopped":true,"says":true}
```
