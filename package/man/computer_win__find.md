#### Description

Filters the app's UI Automation tree by text (a case-insensitive substring of the element's name or value, or an exact automation id) and/or `--role`. `--role` takes UI Automation control types (`Button`, `Edit`, `Document`, `Text`, `CheckBox`, `ComboBox`, `MenuItem`, `ListItem`...) and also the macOS names (`AXButton`, `AXTextField`, `AXStaticText`...). Each match has a `center` in logical screen pixels and a `windowCenter` relative to its window, ready for `click`.

#### Usage

```bash
aux4 computer win find [text] [--role Button] [--app <name>]
```

--text       Text to search for in element label/value
--role       Filter by UI Automation control type, e.g. Button (macOS names like AXButton are accepted)
--app        App to search (defaults to the foreground app)

#### Example

```bash
aux4 computer win find Seven --role Button --app Calculator
```

```json
{"app":"Calculator","matches":[{"automationId":"num7Button","center":{"x":436,"y":464},"enabled":true,"focused":false,"frame":{"height":44,"width":56,"x":408,"y":442},"label":"Seven","ref":"0.2.3.7","role":"Button","value":"","windowCenter":{"x":34,"y":241}}]}
```
