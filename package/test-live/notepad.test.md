# live: Notepad

```timeout
60000
```

Types, selects, sets, copies, pastes, moves and resizes in Notepad in the background, then answers its unsaved-changes dialog (CI only: it discards the text).

```beforeAll
aux4 computer win open notepad --wait 15s > /dev/null; sleep 1
```

```afterAll
aux4 computer win quit notepad --force true > /dev/null 2>&1; true
```

## editing

```timeout
60000
```

### should type in the background

```execute
aux4 computer win type "Hello from aux4" --app notepad | jq -c '{mode, typed}'
```

```expect
{"mode":"background","typed":15}
```

### should show the typed text in the UI Automation tree

```execute
aux4 computer win wait --app notepad --text "Hello from aux4" --source ax --timeout 5s | jq '.matches | length > 0'
```

```expect
true
```

### should select text in the background

```execute
aux4 computer win select --app notepad --text aux4 | jq -c '{mode, selected, range}'
```

```expect
{"mode":"background","selected":"aux4","range":{"length":4,"start":11}}
```

### should copy the selection in the background

```execute
aux4 computer win clipboard copy --app notepad | jq -c '{mode, text}'
```

```expect
{"mode":"background","text":"aux4"}
```

### should read the clipboard

```execute
aux4 computer win clipboard read
```

```expect
{"text":"aux4"}
```

### should paste in the background

```execute
aux4 computer win clipboard paste --text " pasted" --app notepad | jq -r .mode
```

```expect
background
```

### should set the text field's value

```execute
REF=$(aux4 computer win state --app notepad | jq -r '[.elements[] | select(.role == "Edit" or .role == "Document")][0].ref'); aux4 computer win set --app notepad --element "$REF" --value "Set by aux4/computer-windows CI" | jq -c '{mode, after}'
```

```expect
{"mode":"background","after":"Set by aux4/computer-windows CI"}
```

### should select all with ctrl+a in the background

```execute
aux4 computer win shortcut ctrl+a --app notepad | jq -r .mode
```

```expect
background
```

### should list the menu bar

```execute
aux4 computer win menu --app notepad 2>/dev/null | jq -r '[.menus[].title] | length > 0' || echo "no menu bar"
```

```expect:partial
true
```

## windows

```timeout
60000
```

### should move the window in the background

```execute
aux4 computer win window move --app notepad --x 120 --y 90 | jq -c '{mode, x: .after.frame.x, y: .after.frame.y}'
```

```expect
{"mode":"background","x":120,"y":90}
```

### should resize the window

```execute
aux4 computer win window resize --app notepad --width 640 --height 480 | jq -c '{width: .after.frame.width, height: .after.frame.height}'
```

```expect
{"width":640,"height":480}
```

### should read back the new frame

```execute
aux4 computer win window list notepad | jq -c '[.windows[] | select(.dialog | not) | .frame][0]'
```

```expect
{"height":480,"width":640,"x":120,"y":90}
```

### should minimize without activating

```execute
aux4 computer win window minimize --app notepad | jq -c '{mode, minimized: .after.minimized}'
```

```expect
{"mode":"background","minimized":true}
```

### should restore

```execute
aux4 computer win window restore --app notepad | jq -c '{minimized: .after.minimized}'
```

```expect
{"minimized":false}
```

### should hide and unhide

```execute
aux4 computer win hide notepad | jq -c '{hidden, changed}'; aux4 computer win unhide notepad | jq -c '{hidden, changed}'
```

```expect
{"hidden":true,"changed":true}
{"hidden":false,"changed":true}
```

## the unsaved-changes dialog

```timeout
60000
```

### should report the dialog instead of closing (or close if this Notepad keeps unsaved tabs)

```execute
aux4 computer win window close --app notepad | tee close.json | jq -r 'if .dialog != null then "dialog" else "closed" end'
```

```expect:regex
(dialog|closed)
```

### should list Don't Save

```execute
jq -e '.dialog == null' close.json > /dev/null && echo yes || aux4 computer win dialog --app notepad | jq -r '[.dialog.buttons[].title] | index("Don'"'"'t Save") != null | if . then "yes" else "no" end'
```

```expect
yes
```

### should press Don't Save in the background

```execute
jq -e '.dialog == null' close.json > /dev/null && echo '{"closed":true}' || aux4 computer win dialog --app notepad --press "Don't Save" | jq -c '{closed}'; rm -f close.json
```

```expect
{"closed":true}
```
