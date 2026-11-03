# aux4 computer win drag

## dragging

### should require all four coordinates

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win drag --x 1 --y 2 --toX 3
```

```error
{"error":"--x, --y, --toX and --toY are required"}
```

### should report from and to

```execute
AUX4_COMPUTER_DRY_RUN=1 aux4 computer win drag --x 10 --y 20 --toX 30 --toY 40 | jq -c '{from, to, mode}'
```

```expect
{"from":{"x":10,"y":20},"to":{"x":30,"y":40},"mode":"foreground"}
```
