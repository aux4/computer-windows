# aux4 computer win screen

## listing displays

### should list displays with index, size, scale and origin

```execute
aux4 computer win screen | jq -c '[(.displays | length > 0), ([.displays[] | (.index | type), (.width | type), (.height | type), (.scale | type), (.x | type), (.y | type), (.main | type)] | unique)]'
```

```expect
[true,["boolean","number"]]
```

### should mark one display as main

```execute
aux4 computer win screen | jq '[.displays[] | select(.main)] | length'
```

```expect
1
```

## without a desktop

### should error with guidance

```execute
AUX4_COMPUTER_DENY_PERMISSIONS=1 aux4 computer win screen
```

```error:partial
Permission not granted: this process has no interactive, unlocked desktop*?
```
