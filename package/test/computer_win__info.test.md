# aux4 computer win info

```beforeAll
rm -rf state state2
```

```afterAll
rm -rf state state2
```

## describing the provider

### should report the contract fields

```execute
aux4 computer win info | jq -c '{provider, contract, platform: (.platform | test("^(windows|darwin|linux)/(amd64|arm64)$")), version: (.version | type), capabilities: ([.capabilities[] | type] | unique), permissions: (.permissions | type)}'
```

```expect
{"provider":"win","contract":1,"platform":true,"version":"string","capabilities":["boolean"],"permissions":"object"}
```

### should keep the stop file and the log in AUX4_COMPUTER_STATE_DIR

```execute
AUX4_COMPUTER_STATE_DIR=$PWD/state aux4 computer win info | jq -c '{stop: (.stopFile | test("state.STOP$")), log: (.logFile | test("state.actions.jsonl$"))}'
```

```expect
{"stop":true,"log":true}
```
