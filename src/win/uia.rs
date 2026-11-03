// UI Automation: one cached snapshot of a window's control-view tree (a single cross-process
// call), refs as child-index paths ("<window>.<child>.<child>..."), and pattern helpers.

use super::*;
use windows::core::{Interface, BSTR};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::*;

thread_local! {
    static UIA: std::cell::RefCell<Option<IUIAutomation>> = const { std::cell::RefCell::new(None) };
}

pub fn automation() -> IUIAutomation {
    UIA.with(|u| {
        if let Some(a) = u.borrow().as_ref() {
            return a.clone();
        }
        let a: IUIAutomation = match unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) } {
            Ok(a) => a,
            Err(e) => fail(format!("UI Automation is not available: {}", e.message())),
        };
        *u.borrow_mut() = Some(a.clone());
        a
    })
}

#[derive(Clone)]
pub struct Node {
    pub el: IUIAutomationElement,
    pub r#ref: String,
    pub ctype: i32,
    pub role: String,
    pub name: String,
    pub value: String,
    pub has_value: bool,
    pub value_readonly: bool,
    pub automation_id: String,
    pub class: String,
    pub rect: RECT,
    pub enabled: bool,
    pub focused: bool,
    pub offscreen: bool,
    pub hwnd: HWND,
    pub depth: usize,
    pub parent: Option<usize>,
    pub invoke: bool,
    pub toggle: bool,
    pub toggle_state: i32,
    pub selection_item: bool,
    pub selected: bool,
    pub expand: bool,
    pub expand_state: i32,
    pub text: bool,
    pub scroll: bool,
    pub range: bool,
    pub legacy_action: String,
}

pub fn role_name(ct: i32) -> &'static str {
    match ct {
        50000 => "Button",
        50001 => "Calendar",
        50002 => "CheckBox",
        50003 => "ComboBox",
        50004 => "Edit",
        50005 => "Hyperlink",
        50006 => "Image",
        50007 => "ListItem",
        50008 => "List",
        50009 => "Menu",
        50010 => "MenuBar",
        50011 => "MenuItem",
        50012 => "ProgressBar",
        50013 => "RadioButton",
        50014 => "ScrollBar",
        50015 => "Slider",
        50016 => "Spinner",
        50017 => "StatusBar",
        50018 => "Tab",
        50019 => "TabItem",
        50020 => "Text",
        50021 => "ToolBar",
        50022 => "ToolTip",
        50023 => "Tree",
        50024 => "TreeItem",
        50025 => "Custom",
        50026 => "Group",
        50027 => "Thumb",
        50028 => "DataGrid",
        50029 => "DataItem",
        50030 => "Document",
        50031 => "SplitButton",
        50032 => "Window",
        50033 => "Pane",
        50034 => "Header",
        50035 => "HeaderItem",
        50036 => "Table",
        50037 => "TitleBar",
        50038 => "Separator",
        50039 => "SemanticZoom",
        50040 => "AppBar",
        _ => "Unknown",
    }
}

pub fn is_actionable(n: &Node) -> bool {
    matches!(
        n.ctype,
        50000 | 50002 | 50003 | 50004 | 50005 | 50007 | 50011 | 50013 | 50015 | 50016 | 50019 | 50024 | 50029 | 50031 | 50030
    ) || n.invoke
        || n.toggle
}

pub fn is_text_role(n: &Node) -> bool {
    matches!(n.ctype, 50020 | 50006 | 50034 | 50035 | 50030 | 50004)
}

fn prop_bool(el: &IUIAutomationElement, id: UIA_PROPERTY_ID) -> bool {
    unsafe { el.GetCachedPropertyValue(id) }.ok().and_then(|v: VARIANT| bool::try_from(&v).ok()).unwrap_or(false)
}

fn prop_i32(el: &IUIAutomationElement, id: UIA_PROPERTY_ID) -> i32 {
    unsafe { el.GetCachedPropertyValue(id) }.ok().and_then(|v: VARIANT| i32::try_from(&v).ok()).unwrap_or(0)
}

fn prop_str(el: &IUIAutomationElement, id: UIA_PROPERTY_ID) -> String {
    unsafe { el.GetCachedPropertyValue(id) }.ok().and_then(|v: VARIANT| BSTR::try_from(&v).ok()).map(|b| b.to_string()).unwrap_or_default()
}

const PROPS: &[UIA_PROPERTY_ID] = &[
    UIA_NamePropertyId,
    UIA_ControlTypePropertyId,
    UIA_BoundingRectanglePropertyId,
    UIA_IsEnabledPropertyId,
    UIA_HasKeyboardFocusPropertyId,
    UIA_AutomationIdPropertyId,
    UIA_ClassNamePropertyId,
    UIA_IsOffscreenPropertyId,
    UIA_NativeWindowHandlePropertyId,
    UIA_ValueValuePropertyId,
    UIA_ValueIsReadOnlyPropertyId,
    UIA_IsValuePatternAvailablePropertyId,
    UIA_IsInvokePatternAvailablePropertyId,
    UIA_IsTogglePatternAvailablePropertyId,
    UIA_ToggleToggleStatePropertyId,
    UIA_IsSelectionItemPatternAvailablePropertyId,
    UIA_SelectionItemIsSelectedPropertyId,
    UIA_IsExpandCollapsePatternAvailablePropertyId,
    UIA_ExpandCollapseExpandCollapseStatePropertyId,
    UIA_IsTextPatternAvailablePropertyId,
    UIA_IsScrollPatternAvailablePropertyId,
    UIA_IsRangeValuePatternAvailablePropertyId,
    UIA_LegacyIAccessibleDefaultActionPropertyId,
];

fn node_from(el: IUIAutomationElement, r#ref: String, depth: usize, parent: Option<usize>) -> Node {
    unsafe {
        let ctype = el.CachedControlType().map(|c| c.0).unwrap_or(0);
        let rect = el.CachedBoundingRectangle().unwrap_or_default();
        let name = el.CachedName().map(|b| b.to_string()).unwrap_or_default();
        let enabled = el.CachedIsEnabled().map(|b| b.as_bool()).unwrap_or(false);
        let focused = el.CachedHasKeyboardFocus().map(|b| b.as_bool()).unwrap_or(false);
        let automation_id = el.CachedAutomationId().map(|b| b.to_string()).unwrap_or_default();
        let class = el.CachedClassName().map(|b| b.to_string()).unwrap_or_default();
        let offscreen = el.CachedIsOffscreen().map(|b| b.as_bool()).unwrap_or(false);
        let hwnd = el.CachedNativeWindowHandle().unwrap_or_default();
        let has_value = prop_bool(&el, UIA_IsValuePatternAvailablePropertyId);
        let value = if has_value { prop_str(&el, UIA_ValueValuePropertyId) } else { String::new() };
        Node {
            r#ref,
            ctype,
            role: role_name(ctype).to_string(),
            name,
            value,
            has_value,
            value_readonly: prop_bool(&el, UIA_ValueIsReadOnlyPropertyId),
            automation_id,
            class,
            rect,
            enabled,
            focused,
            offscreen,
            hwnd,
            depth,
            parent,
            invoke: prop_bool(&el, UIA_IsInvokePatternAvailablePropertyId),
            toggle: prop_bool(&el, UIA_IsTogglePatternAvailablePropertyId),
            toggle_state: prop_i32(&el, UIA_ToggleToggleStatePropertyId),
            selection_item: prop_bool(&el, UIA_IsSelectionItemPatternAvailablePropertyId),
            selected: prop_bool(&el, UIA_SelectionItemIsSelectedPropertyId),
            expand: prop_bool(&el, UIA_IsExpandCollapsePatternAvailablePropertyId),
            expand_state: prop_i32(&el, UIA_ExpandCollapseExpandCollapseStatePropertyId),
            text: prop_bool(&el, UIA_IsTextPatternAvailablePropertyId),
            scroll: prop_bool(&el, UIA_IsScrollPatternAvailablePropertyId),
            range: prop_bool(&el, UIA_IsRangeValuePatternAvailablePropertyId),
            legacy_action: prop_str(&el, UIA_LegacyIAccessibleDefaultActionPropertyId),
            el,
        }
    }
}

fn cache_request(scope: TreeScope) -> IUIAutomationCacheRequest {
    let uia = automation();
    unsafe {
        let cr = uia.CreateCacheRequest().unwrap_or_else(|e| fail(format!("UI Automation cache request failed: {}", e.message())));
        for p in PROPS {
            let _ = cr.AddProperty(*p);
        }
        let _ = cr.SetTreeScope(scope);
        if let Ok(cond) = uia.ControlViewCondition() {
            let _ = cr.SetTreeFilter(&cond);
        }
        cr
    }
}

/// Snapshot of a window's tree (control view), preorder. max_depth 0 = window element only.
pub fn tree(hwnd: HWND, window_index: usize, max_depth: usize) -> Vec<Node> {
    let uia = automation();
    let root = match unsafe { uia.ElementFromHandle(hwnd) } {
        Ok(r) => r,
        Err(e) => fail(format!("UI Automation can't read this window: {}", e.message())),
    };
    let cr = cache_request(TreeScope_Subtree);
    let root = match unsafe { root.BuildUpdatedCache(&cr) } {
        Ok(r) => r,
        Err(e) => fail(format!("UI Automation can't read this window: {}", e.message())),
    };
    let mut out: Vec<Node> = Vec::new();
    walk(root, window_index.to_string(), 0, None, max_depth, &mut out);
    out
}

fn walk(el: IUIAutomationElement, r#ref: String, depth: usize, parent: Option<usize>, max_depth: usize, out: &mut Vec<Node>) {
    let idx = out.len();
    out.push(node_from(el.clone(), r#ref.clone(), depth, parent));
    if depth >= max_depth || out.len() > 20000 {
        return;
    }
    let kids = match unsafe { el.GetCachedChildren() } {
        Ok(k) => k,
        Err(_) => return,
    };
    let n = unsafe { kids.Length() }.unwrap_or(0);
    for i in 0..n {
        if let Ok(c) = unsafe { kids.GetElement(i) } {
            walk(c, format!("{}.{}", r#ref, i), depth + 1, Some(idx), max_depth, out);
        }
    }
}

/// Snapshot of a single element's cached properties (e.g. the focused element).
pub fn node_of(el: &IUIAutomationElement) -> Option<Node> {
    let cr = cache_request(TreeScope_Element);
    let e = unsafe { el.BuildUpdatedCache(&cr) }.ok()?;
    Some(node_from(e, String::new(), 0, None))
}

/// Resolve a ref against the app's current windows.
pub fn resolve_ref(app: &App, r#ref: &str) -> Node {
    let first = r#ref.split('.').next().unwrap_or("");
    let wi: usize = match first.parse() {
        Ok(i) => i,
        Err(_) => fail(format!("Element not found for ref: {}", r#ref)),
    };
    let w = match app.windows.get(wi) {
        Some(w) => w,
        None => fail(format!("Element not found for ref: {} ({} has {} window{})", r#ref, app.name, app.windows.len(), if app.windows.len() == 1 { "" } else { "s" })),
    };
    let depth = r#ref.split('.').count();
    let nodes = tree(w.hwnd, wi, depth);
    match nodes.into_iter().find(|n| n.r#ref == r#ref) {
        Some(n) => n,
        None => fail(format!("Element not found for ref: {}", r#ref)),
    }
}

/// Deepest element of the app's window containing the physical point.
pub fn element_at(app: &App, win_index: usize, p: POINT) -> Option<Node> {
    let w = app.windows.get(win_index)?;
    let nodes = tree(w.hwnd, win_index, 64);
    let mut best: Option<Node> = None;
    for n in nodes {
        let r = n.rect;
        if r.right <= r.left || r.bottom <= r.top || n.offscreen {
            continue;
        }
        if p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom {
            if best.as_ref().map(|b| n.depth >= b.depth).unwrap_or(true) {
                best = Some(n);
            }
        }
    }
    best
}

pub fn focused_element() -> Option<IUIAutomationElement> {
    unsafe { automation().GetFocusedElement() }.ok()
}

pub fn node_json(n: &Node, win: Option<&RECT>) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("ref".into(), json!(n.r#ref));
    m.insert("role".into(), json!(n.role));
    m.insert("label".into(), json!(n.name));
    m.insert("value".into(), json!(display_value(n)));
    m.insert("enabled".into(), json!(n.enabled));
    m.insert("focused".into(), json!(n.focused));
    if !n.automation_id.is_empty() {
        m.insert("automationId".into(), json!(n.automation_id));
    }
    if n.rect.right > n.rect.left && n.rect.bottom > n.rect.top {
        m.insert("frame".into(), frame_json(&n.rect));
        m.insert("center".into(), center_json(&n.rect));
        if let Some(w) = win {
            m.insert("windowCenter".into(), window_center_json(&n.rect, w));
        }
    } else {
        m.insert("frame".into(), json!({}));
    }
    if n.toggle {
        m.insert("checked".into(), json!(n.toggle_state == 1));
    }
    if n.selection_item {
        m.insert("selected".into(), json!(n.selected));
    }
    Value::Object(m)
}

pub fn display_value(n: &Node) -> String {
    let v = if n.value.chars().count() > 2000 { n.value.chars().take(2000).collect::<String>() + "…" } else { n.value.clone() };
    v
}

pub fn short_json(n: &Node) -> Value {
    json!({"ref": n.r#ref, "role": n.role, "label": n.name})
}

// ---------------------------------------------------------------------------------------------
// patterns

pub fn invoke(n: &Node) -> Result<(), String> {
    unsafe {
        let p: IUIAutomationInvokePattern = n.el.GetCurrentPatternAs(UIA_InvokePatternId).map_err(|e| e.message())?;
        p.Invoke().map_err(|e| e.message())
    }
}

pub fn toggle(n: &Node) -> Result<i32, String> {
    unsafe {
        let p: IUIAutomationTogglePattern = n.el.GetCurrentPatternAs(UIA_TogglePatternId).map_err(|e| e.message())?;
        p.Toggle().map_err(|e| e.message())?;
        Ok(p.CurrentToggleState().map(|s| s.0).unwrap_or(0))
    }
}

pub fn toggle_state(n: &Node) -> Option<i32> {
    unsafe {
        let p: IUIAutomationTogglePattern = n.el.GetCurrentPatternAs(UIA_TogglePatternId).ok()?;
        p.CurrentToggleState().ok().map(|s| s.0)
    }
}

pub fn select_item(n: &Node) -> Result<(), String> {
    unsafe {
        let p: IUIAutomationSelectionItemPattern = n.el.GetCurrentPatternAs(UIA_SelectionItemPatternId).map_err(|e| e.message())?;
        p.Select().map_err(|e| e.message())
    }
}

pub fn expand(n: &Node) -> Result<(), String> {
    unsafe {
        let p: IUIAutomationExpandCollapsePattern = n.el.GetCurrentPatternAs(UIA_ExpandCollapsePatternId).map_err(|e| e.message())?;
        p.Expand().map_err(|e| e.message())
    }
}

pub fn collapse(n: &Node) -> Result<(), String> {
    unsafe {
        let p: IUIAutomationExpandCollapsePattern = n.el.GetCurrentPatternAs(UIA_ExpandCollapsePatternId).map_err(|e| e.message())?;
        p.Collapse().map_err(|e| e.message())
    }
}

pub fn legacy_default_action(n: &Node) -> Result<(), String> {
    unsafe {
        let p: IUIAutomationLegacyIAccessiblePattern = n.el.GetCurrentPatternAs(UIA_LegacyIAccessiblePatternId).map_err(|e| e.message())?;
        p.DoDefaultAction().map_err(|e| e.message())
    }
}

pub fn current_value(el: &IUIAutomationElement) -> Option<String> {
    unsafe {
        let p: IUIAutomationValuePattern = el.GetCurrentPatternAs(UIA_ValuePatternId).ok()?;
        p.CurrentValue().ok().map(|b| b.to_string())
    }
}

pub fn set_value(el: &IUIAutomationElement, v: &str) -> Result<(), String> {
    unsafe {
        let p: IUIAutomationValuePattern = el.GetCurrentPatternAs(UIA_ValuePatternId).map_err(|e| e.message())?;
        p.SetValue(&BSTR::from(v)).map_err(|e| e.message())
    }
}

pub fn range_value(el: &IUIAutomationElement) -> Option<f64> {
    unsafe {
        let p: IUIAutomationRangeValuePattern = el.GetCurrentPatternAs(UIA_RangeValuePatternId).ok()?;
        p.CurrentValue().ok()
    }
}

pub fn set_range_value(el: &IUIAutomationElement, v: f64) -> Result<(), String> {
    unsafe {
        let p: IUIAutomationRangeValuePattern = el.GetCurrentPatternAs(UIA_RangeValuePatternId).map_err(|e| e.message())?;
        p.SetValue(v).map_err(|e| e.message())
    }
}

pub fn text_pattern(el: &IUIAutomationElement) -> Option<IUIAutomationTextPattern> {
    unsafe { el.GetCurrentPatternAs(UIA_TextPatternId).ok() }
}

pub fn document_text(el: &IUIAutomationElement) -> Option<String> {
    let tp = text_pattern(el)?;
    unsafe { tp.DocumentRange().ok()?.GetText(-1).ok().map(|b| b.to_string()) }
}

pub fn selected_text(el: &IUIAutomationElement) -> Option<String> {
    let tp = text_pattern(el)?;
    unsafe {
        let arr = tp.GetSelection().ok()?;
        let n = arr.Length().ok()?;
        let mut s = String::new();
        for i in 0..n {
            if let Ok(r) = arr.GetElement(i) {
                if let Ok(t) = r.GetText(-1) {
                    s.push_str(&t.to_string());
                }
            }
        }
        Some(s)
    }
}

/// Select characters [start, start+len) of the element's document through TextPattern.
pub fn text_select_range(el: &IUIAutomationElement, start: i32, len: i32) -> Result<(), String> {
    let tp = text_pattern(el).ok_or("the element has no UI Automation text pattern")?;
    unsafe {
        let r = tp.DocumentRange().map_err(|e| e.message())?;
        // collapse to the start, then move each endpoint by characters
        r.MoveEndpointByRange(TextPatternRangeEndpoint_End, &r, TextPatternRangeEndpoint_Start).map_err(|e| e.message())?;
        if start > 0 {
            r.MoveEndpointByUnit(TextPatternRangeEndpoint_End, TextUnit_Character, start).map_err(|e| e.message())?;
            r.MoveEndpointByUnit(TextPatternRangeEndpoint_Start, TextUnit_Character, start).map_err(|e| e.message())?;
        }
        if len > 0 {
            r.MoveEndpointByUnit(TextPatternRangeEndpoint_End, TextUnit_Character, len).map_err(|e| e.message())?;
        }
        r.Select().map_err(|e| e.message())
    }
}

pub fn scroll_percent(el: &IUIAutomationElement) -> Option<(f64, f64, bool, bool)> {
    unsafe {
        let p: IUIAutomationScrollPattern = el.GetCurrentPatternAs(UIA_ScrollPatternId).ok()?;
        Some((
            p.CurrentHorizontalScrollPercent().ok()?,
            p.CurrentVerticalScrollPercent().ok()?,
            p.CurrentHorizontallyScrollable().map(|b| b.as_bool()).unwrap_or(false),
            p.CurrentVerticallyScrollable().map(|b| b.as_bool()).unwrap_or(false),
        ))
    }
}

pub fn set_scroll_percent(el: &IUIAutomationElement, h: f64, v: f64) -> Result<(), String> {
    unsafe {
        let p: IUIAutomationScrollPattern = el.GetCurrentPatternAs(UIA_ScrollPatternId).map_err(|e| e.message())?;
        p.SetScrollPercent(h, v).map_err(|e| e.message())
    }
}

pub fn window_pattern(hwnd: HWND) -> Option<IUIAutomationWindowPattern> {
    unsafe {
        let el = automation().ElementFromHandle(hwnd).ok()?;
        el.GetCurrentPatternAs(UIA_WindowPatternId).ok()
    }
}

/// Nearest node at or above `idx` that has a native window handle.
pub fn nearest_hwnd(nodes: &[Node], idx: usize) -> Option<HWND> {
    let mut i = Some(idx);
    while let Some(k) = i {
        if !nodes[k].hwnd.0.is_null() {
            return Some(nodes[k].hwnd);
        }
        i = nodes[k].parent;
    }
    None
}

pub fn element_hwnd(el: &IUIAutomationElement) -> Option<HWND> {
    unsafe { el.CurrentNativeWindowHandle().ok().filter(|h| !h.0.is_null()) }
}

#[allow(dead_code)]
pub fn same_element(a: &IUIAutomationElement, b: &IUIAutomationElement) -> bool {
    unsafe { automation().CompareElements(a, b).map(|x| x.as_bool()).unwrap_or(false) }
}

#[allow(dead_code)]
pub fn as_unknown(el: &IUIAutomationElement) -> windows::core::IUnknown {
    el.cast().unwrap()
}
