# aux4 computer win resume

```beforeAll
rm -rf state state2
```

```afterAll
rm -rf state state2
```

## turning the stop switch off

### should remove the STOP file and allow input again

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win stop > /dev/null; AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win resume | jq -c '{stopped}'; test -f state/STOP || echo removed; AUX4_COMPUTER_STATE_DIR=$PWD/state AUX4_COMPUTER_DRY_RUN=1 aux4 computer win type hi --app notepad | jq -c '{mode}'
```

```expect
{"stopped":false}
removed
{"mode":"background"}
```

### should say the environment switch is still on

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state AUX4_COMPUTER_STOP=1 aux4 computer win resume | jq -c '{stopped, reason: (.stopReason | test("unset it"))}'
```

```expect
{"stopped":true,"reason":true}
```
