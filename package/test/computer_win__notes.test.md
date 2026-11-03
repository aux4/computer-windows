# aux4 computer win notes

## printing the notes

### should start with the Windows heading

```execute
aux4 computer win notes | head -1
```

```expect
## Windows (aux4 computer win)
```

### should explain UIPI

```execute
aux4 computer win notes | grep -c 'Elevation (UIPI)'
```

```expect
1
```
