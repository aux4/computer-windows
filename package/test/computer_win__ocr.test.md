# aux4 computer win ocr

## validation

### should take --app or --image, not both

```execute
aux4 computer win ocr --app notepad --image x.png
```

```error
{"error":"Use either --app or --image, not both"}
```

### should reject an invalid level

```execute
aux4 computer win ocr --level best
```

```error
{"error":"Invalid --level 'best': use accurate or fast"}
```

### should report a missing image

```execute
aux4 computer win ocr --image missing.png
```

```error
{"error":"Image not found: missing.png"}
```

### should reject a zero timeout

```execute
aux4 computer win ocr --timeout 0s
```

```error
{"error":"Invalid --timeout '0s': must be more than 0"}
```
