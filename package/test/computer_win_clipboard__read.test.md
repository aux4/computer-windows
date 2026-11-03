# aux4 computer win clipboard read

## reading

### should return the clipboard text

```execute
aux4 computer win clipboard copy --text 'read me' > /dev/null && aux4 computer win clipboard read | jq -r .text
```

```expect
read me
```
