#### Description

Prints the Windows specifics an agent needs, as markdown: what runs in the background and what is always foreground, elevation (UIPI) limits, app names, roles and refs, coordinates, OCR and dialog behavior, and known gotchas. The generic computer-use skill of `aux4/computer` appends it for this OS.

#### Usage

```bash
aux4 computer win notes
```

#### Example

```bash
aux4 computer win notes | head -3
```

```text
## Windows (aux4 computer win)

**Reaching apps.** There is no permission prompt on Windows. What matters is:
```
