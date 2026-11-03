# aux4 computer win screenshot

## validation

### should reject an invalid display

```execute
aux4 computer win screenshot --display one
```

```error
{"error":"Invalid display index: one"}
```

### should reject invalid roles

```execute
aux4 computer win screenshot --annotate true --roles some
```

```error
{"error":"Invalid --roles 'some': use actionable or all"}
```

## without a desktop

### should error with guidance

```execute
AUX4_COMPUTER_DENY_PERMISSIONS=1 aux4 computer win screenshot
```

```error:partial
Permission not granted*?
```
