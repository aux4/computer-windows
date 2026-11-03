# aux4 computer win state

## validation

### should reject a zero limit

```execute
aux4 computer win state --limit 0
```

```error
{"error":"Invalid --limit '0': use a positive number"}
```

## without a desktop

### should error with guidance

```execute
AUX4_COMPUTER_DENY_PERMISSIONS=1 aux4 computer win state
```

```error:partial
Permission not granted*?
```
