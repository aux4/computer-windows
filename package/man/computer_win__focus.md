#### Description

Brings an app's main window to the front (always foreground). It restores the window if it is minimized.

#### Usage

```bash
aux4 computer win focus <app>
```

--app        Application: exe name, window title or pid

#### Example

```bash
aux4 computer win focus notepad
```

```json
{"focused":"notepad","mode":"foreground","pid":4120,"success":true}
```
