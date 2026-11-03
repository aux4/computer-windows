# aux4 computer win apps

## listing apps

### should give every app a name, pid, path and hidden flag

```execute
aux4 computer win apps | jq -r 'all(.apps[]; (.name | type) == "string" and (.pid | type) == "number" and (.path | type) == "string" and (.hidden | type) == "boolean")'
```

```expect
true
```
