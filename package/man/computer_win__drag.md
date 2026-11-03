#### Description

Drags the mouse with the left button from one screen point to another (logical pixels), in small steps. It is always real input.

#### Usage

```bash
aux4 computer win drag --x <x> --y <y> --toX <x> --toY <y>
```

--x          Start X coordinate
--y          Start Y coordinate
--toX        End X coordinate
--toY        End Y coordinate

#### Example

```bash
aux4 computer win drag --x 100 --y 200 --toX 400 --toY 200
```

```json
{"from":{"x":100,"y":200},"mode":"foreground","to":{"x":400,"y":200}}
```
