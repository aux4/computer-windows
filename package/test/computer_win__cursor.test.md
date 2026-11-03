# aux4 computer win cursor

## reading the cursor

### should report x and y as numbers

```execute
aux4 computer win cursor | jq -c '[(.x | type), (.y | type)]'
```

```expect
["number","number"]
```
