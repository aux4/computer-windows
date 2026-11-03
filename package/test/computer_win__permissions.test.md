# aux4 computer win permissions

## reporting what this process can reach

### should report each entry as granted + guidance

```execute
aux4 computer win permissions | jq -c '[.[] | (.granted | type), (.guidance | type)] | unique'
```

```expect
["boolean","string"]
```

### should report the desktop, elevation and uiAccess

```execute
aux4 computer win permissions | jq -c 'keys'
```

```expect
["desktop","elevated","uiAccess"]
```

### should report nothing granted when denied for testing

```execute
AUX4_COMPUTER_DENY_PERMISSIONS=1 aux4 computer win permissions | jq -c '[.[] | .granted] | unique'
```

```expect
[false]
```
