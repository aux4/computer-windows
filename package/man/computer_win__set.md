#### Description

Sets an element by `ref`: a text field's or slider's `--value`, a checkbox's or radio button's `--checked` state, or the `--option` of a combo box, list or radio group. It returns the value `before` and `after`. Win32 text fields are written as a real edit (`EM_REPLACESEL`: undoable, marks the document modified); other fields use the UI Automation Value or RangeValue pattern; checkboxes Toggle until they reach the state; options are picked with SelectionItem (a collapsed combo box is expanded and collapsed again).

#### Usage

```bash
aux4 computer win set --app <name> --element <ref> (--value <text|number> | --checked true|false | --option <title>)
```

--app        App the element belongs to: exe name, window title or pid (default: the foreground app)
--element    Element ref from 'find' or 'state'
--value      New value for a text field, text area, combo box or slider
--checked    true or false: press the checkbox/radio button until it is in that state
--option     Option to pick in a combo box, list or radio group
--mode     auto (default): act in the background, fall back to real input only if that isn't possible (then put the cursor and the foreground window back); background: never fall back (error instead); foreground: always use real input

#### Example

```bash
aux4 computer win set --app notepad --element 0.0 --value "Replaced by aux4"
```

```json
{"after":"Replaced by aux4","app":"notepad","before":"Hello from aux4","element":{"label":"Text Editor","ref":"0.0","role":"Edit"},"method":"EM_REPLACESEL","mode":"background"}
```
