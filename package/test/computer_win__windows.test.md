# aux4 computer win windows

## listing windows

### should list windows with app, title, pid and frame

```execute
aux4 computer win windows | jq -r '(.windows | type) == "array" and all(.windows[]; (.app | type) == "string" and (.title | type) == "string" and (.pid | type) == "number" and ([.frame.x, .frame.y, .frame.width, .frame.height] | map(type) | unique) == ["number"])'
```

```expect
true
```
