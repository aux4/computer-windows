# aux4 computer win clipboard copy

## with --text

### should put the text on the clipboard

```execute
aux4 computer win clipboard copy --text 'copied by aux4/computer-windows' && aux4 computer win clipboard read
```

```expect
{"text":"copied by aux4/computer-windows"}
{"text":"copied by aux4/computer-windows"}
```
