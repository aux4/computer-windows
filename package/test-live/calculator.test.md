# live: Calculator

```timeout
60000
```

Drives Calculator in the background: the classic Win32 Calculator on Windows Server, the UWP one on Windows 11. Run on a desktop you don't mind being used (CI runners).

```beforeAll
aux4 computer win open calc --wait 20s > /dev/null; aux4 computer win wait --app Calculator --timeout 20s > /dev/null; sleep 2; aux4 computer win cursor > cursor-before.json; aux4 computer win apps | jq -c '[.apps[] | select(.active) | .name]' > foreground-before.json
```

```afterAll
aux4 computer win quit Calculator --force true > /dev/null; rm -f cursor-before.json foreground-before.json calc-annotated.png mark.txt
```

## 256 x 7 with background clicks

```timeout
60000
```

### should clear

```execute
REF=$(aux4 computer win find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "clearButton" or .label == "Clear" or .label == "C")][0].ref // empty') && aux4 computer win click --element "$REF" --app Calculator --mode background | jq -r .mode
```

```expect
background
```

### should press 2 in the background

```execute
REF=$(aux4 computer win find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "num2Button" or .label == "2" or .label == "Two")][0].ref // empty') && aux4 computer win click --element "$REF" --app Calculator --mode background | jq -r .mode
```

```expect
background
```

### should press 5 in the background

```execute
REF=$(aux4 computer win find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "num5Button" or .label == "5" or .label == "Five")][0].ref // empty') && aux4 computer win click --element "$REF" --app Calculator --mode background | jq -r .mode
```

```expect
background
```

### should press 6 in the background

```execute
REF=$(aux4 computer win find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "num6Button" or .label == "6" or .label == "Six")][0].ref // empty') && aux4 computer win click --element "$REF" --app Calculator --mode background | jq -r .mode
```

```expect
background
```

### should press multiply in the background

```execute
REF=$(aux4 computer win find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "multiplyButton" or .label == "Multiply" or .label == "Multiply by")][0].ref // empty') && aux4 computer win click --element "$REF" --app Calculator --mode background | jq -r .mode
```

```expect
background
```

### should press 7 in the background

```execute
REF=$(aux4 computer win find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "num7Button" or .label == "7" or .label == "Seven")][0].ref // empty') && aux4 computer win click --element "$REF" --app Calculator --mode background | jq -r .mode
```

```expect
background
```

### should see the result appear with wait (started before equals)

```execute
(sleep 1.5; REF=$(aux4 computer win find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "equalButton" or .label == "Equals")][0].ref // empty') && aux4 computer win click --element "$REF" --app Calculator --mode background | jq -r .mode > /dev/null) & aux4 computer win wait --app Calculator --text 1792 --timeout 10s | jq -c '{found: (.matches | length > 0), waitedForIt: (.waited >= 1), source}'
```

```expect:regex
\{"found":true,"waitedForIt":true,"source":"(ax|ocr)"\}
```

### should not have moved the cursor

```execute
aux4 computer win cursor | diff - cursor-before.json && echo unchanged
```

```expect
unchanged
```

### should not have changed the foreground app

```execute
aux4 computer win apps | jq -c '[.apps[] | select(.active) | .name]' | diff - foreground-before.json && echo unchanged
```

```expect
unchanged
```

## pixels and UI Automation agree

```timeout
60000
```

### should find the result in the UI Automation tree

```execute
aux4 computer win find 1792 --app Calculator | jq '.matches | length' | grep -q -v '^0$' && echo found || (aux4 computer win find 1,792 --app Calculator | jq -e '.matches | length > 0' > /dev/null && echo found)
```

```expect
found
```

### should read the result with OCR inside the element's frame

```execute
F=$(aux4 computer win find 1,792 --app Calculator); [ "$(echo "$F" | jq '.matches | length')" = 0 ] && F=$(aux4 computer win find 1792 --app Calculator); O=$(aux4 computer win ocr --app Calculator --text 1792); jq -n --argjson f "$F" --argjson o "$O" '($f.matches | map(select(.frame.width > 0)) | min_by(.frame.width * .frame.height)) as $e | ($o.matches[0].match // $o.matches[0]) as $m | $m.center.x >= $e.frame.x and $m.center.x <= $e.frame.x + $e.frame.width and $m.center.y >= $e.frame.y and $m.center.y <= $e.frame.y + $e.frame.height'
```

```expect
true
```

### should see the result with OCR only

```execute
aux4 computer win wait --app Calculator --text 1792 --source ocr --timeout 15s | jq -c '{source, found: (.matches | length > 0)}'
```

```expect
{"source":"ocr","found":true}
```

### should not report the shown result as gone

```execute
aux4 computer win wait --app Calculator --text 1792 --gone true --timeout 1s 2>&1 | jq -r '.error | startswith("Timed out")'
```

```expect
true
```

## set-of-marks

```timeout
60000
```

### should clear before annotating (refs follow the tree as it is)

```execute
REF=$(aux4 computer win find "" --role Button --app Calculator | jq -r '[.matches[] | select(.automationId == "clearButton" or .label == "Clear" or .label == "C")][0].ref // empty') && aux4 computer win click --element "$REF" --app Calculator --mode background | jq -r .mode; sleep 0.5
```

```expect
background
```

### should annotate the window

```execute
aux4 computer win screenshot --app Calculator --annotate true --output calc-annotated.png | tee mark.json | jq -c '{annotated, many: (.marks | length > 5)}'; jq -r '[.marks[] | select(.label == "7" or .label == "Seven")][0].ref' mark.json > mark.txt; rm -f mark.json
```

```expect
{"annotated":true,"many":true}
```

### should press a mark by its ref

```execute
aux4 computer win click --element "$(cat mark.txt)" --app Calculator | jq -r .mode
```

```expect
background
```

### should show the pressed digit

```execute
sleep 0.5; aux4 computer win find 7 --app Calculator | jq 'any(.matches[]; .role != "Button")'
```

```expect
true
```

### should click by window coordinates in the background

```execute
P=$(aux4 computer win find "" --role Button --app Calculator | jq -c '[.matches[] | select(.automationId == "num8Button" or .label == "8" or .label == "Eight")][0].windowCenter'); aux4 computer win click --app Calculator --x "$(echo "$P" | jq .x)" --y "$(echo "$P" | jq .y)" | jq -r .mode
```

```expect
background
```

### should show the digits pressed so far

```execute
sleep 0.5; aux4 computer win find 78 --app Calculator | jq 'any(.matches[]; .role != "Button")'
```

```expect
true
```

## quitting

```timeout
60000
```

### should quit in the background

```execute
aux4 computer win quit Calculator --timeout 10s | jq -c '{terminated, mode}'
```

```expect
{"terminated":true,"mode":"background"}
```
