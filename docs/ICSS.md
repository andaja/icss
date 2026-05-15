# ICSS — CSS-like Theme Engine for iced

`.icss` is a CSS subset for styling iced applications. It uses standard CSS syntax with multi-class conjunctive selectors (like HTML's `class="button primary small"`).

## Quick Example

```css
:root {
    --primary: #0f3460;
    --text: #eaeaea;
}

.button {
    color: #ffffff;
    border-radius: 8px;
}

.primary {
    background-color: var(--primary);
}

.primary:hover {
    background-color: #1a4a80;
}

.button.primary {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}
```

```rust
let theme = icss::Theme::load(include_str!("theme.icss"))?;
button("Connect").style(theme.button(&["button", "primary"]));
```

---

## Selectors

### Class Selectors

Every selector starts with `.` and a class name. Multiple classes are conjunctive (AND).

| Syntax | Meaning |
|--------|---------|
| `.button` | Matches elements with class "button" |
| `.button.primary` | Matches elements with BOTH "button" AND "primary" |
| `.button.primary.small` | All three classes must be present |

An element with classes `["button", "primary", "small"]` will match `.button`, `.primary`, `.button.primary`, and `.button.primary.small` — but NOT `.button.danger`.

### Pseudo-Classes

Pseudo-classes represent widget interaction states. They are appended after the class selector.

| Pseudo-class | Maps to iced Status | Applies to |
|---|---|---|
| `:hover` | `Status::Hovered` | button, text_input, checkbox, toggler, pick_list, scrollable |
| `:active` | `Status::Pressed` | button |
| `:disabled` | `Status::Disabled` | button, text_input, checkbox, toggler |
| `:focus` | `Status::Focused` | text_input |
| `:checked` | `is_checked: true` | checkbox, toggler, radio |

Examples:
```css
.button:hover { background-color: #1a4a80; }
.button.primary:disabled { background-color: #333; color: rgba(255,255,255,0.4); }
.input:focus { border-color: var(--primary); }
.checkbox:checked { accent-color: var(--primary); }
```

### Application States as Classes

For app-level states (active tab, error, selected), add/remove classes in Rust:

```rust
let cls = if is_active { &["tab", "active"][..] } else { &["tab"][..] };
button(label).style(theme.button(&cls))
```

```css
.tab { color: var(--text-dim); }
.tab.active { background-color: var(--primary); color: #fff; }
.tab.active:hover { background-color: var(--primary-light); }
```

### Special Selectors

| Selector | Purpose |
|----------|---------|
| `:root { }` | Declares custom properties (CSS variables) |

### Nesting

Rules can be nested. The nested selector's classes merge with the parent's:

```css
.sidebar {
    background-color: var(--surface);

    .action {
        border-radius: 4px;
    }
}
```

This is equivalent to:
```css
.sidebar { background-color: var(--surface); }
.sidebar.action { border-radius: 4px; }
```

---

## Specificity

Specificity determines which rule wins when multiple rules match the same element.

**Formula:** `(pseudo_count, class_count)`

| Selector | Specificity |
|----------|-------------|
| `.button` | (0, 1) |
| `.button:hover` | (1, 1) |
| `.button.primary` | (0, 2) |
| `.button.primary:hover` | (1, 2) |
| `.button.primary.small` | (0, 3) |

- Higher specificity wins.
- Equal specificity: **last rule in source order** wins.
- Base rules (no pseudo) are applied first, then pseudo-specific rules layer on top.

---

## Custom Properties (Variables)

Declared in `:root` and referenced with `var()`:

```css
:root {
    --primary: #0f3460;
    --radius-sm: 4px;
}

.button {
    background-color: var(--primary);
    border-radius: var(--radius-sm);
}
```

### var() with fallback

```css
.button {
    color: var(--text, #ffffff);
}
```

If `--text` is not defined, the fallback `#ffffff` is used.

---

## Value Types

| Type | Examples | Notes |
|------|----------|-------|
| Hex color | `#ff0000`, `#f00`, `#ff000080`, `#f008` | 3, 4, 6, or 8 digits |
| `rgb()` | `rgb(255, 0, 0)` | |
| `rgba()` | `rgba(0, 0, 0, 0.5)` | Alpha: 0.0–1.0 |
| Named color | `red`, `white`, `transparent` | See table below |
| Length | `8px`, `8` | Unitless = px |
| Number | `0.5` | For opacity |
| Keyword | `none`, `transparent` | |
| Variable | `var(--name)`, `var(--name, fallback)` | |

### Named Colors

`black`, `white`, `red`, `green`, `blue`, `yellow`, `cyan`/`aqua`, `magenta`/`fuchsia`, `orange`, `gray`/`grey`, `darkgray`/`darkgrey`, `lightgray`/`lightgrey`, `silver`, `maroon`, `olive`, `lime`, `teal`, `navy`, `purple`, `indigo`, `coral`, `salmon`, `tomato`, `crimson`, `gold`, `khaki`, `skyblue`, `steelblue`, `slategray`/`slategrey`

---

## Supported Properties

### All Widgets (where applicable)

| CSS Property | iced Field | Value Type | Description |
|---|---|---|---|
| `background-color` | `background` | color | Background fill color |
| `color` | `text_color` | color | Text / foreground color |
| `border-radius` | `border.radius` | length | Corner rounding (all corners) |
| `border-width` | `border.width` | length | Border stroke width |
| `border-color` | `border.color` | color | Border stroke color |
| `box-shadow` | `shadow` | shadow | Drop shadow |
| `opacity` | alpha multiplier | number (0–1) | Applied to all colors |

### Shadow Syntax

```css
box-shadow: <offset-x> <offset-y> <blur-radius> <color>;
```

Examples:
```css
box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
box-shadow: 0 4px 12px #00000060;
box-shadow: none;
```

### Button Properties

| Property | Default | States |
|---|---|---|
| `background-color` | none | :hover, :active, :disabled |
| `color` | white | :hover, :active, :disabled |
| `border-radius` | 0 | |
| `border-width` | 0 | |
| `border-color` | transparent | :hover, :active, :disabled |
| `box-shadow` | none | :hover, :active, :disabled |
| `opacity` | 1.0 | :disabled |

### Container Properties

| Property | Default |
|---|---|
| `background-color` | none |
| `color` | none (inherits) |
| `border-radius` | 0 |
| `border-width` | 0 |
| `border-color` | transparent |
| `box-shadow` | none |

Containers are stateless — no pseudo-classes.

### Text Input Properties

| Property | Default | States |
|---|---|---|
| `background-color` | transparent | :hover, :focus, :disabled |
| `color` | white | Text value color |
| `border-radius` | 0 | |
| `border-width` | 0 | |
| `border-color` | transparent | :hover, :focus, :disabled |
| `placeholder-color` | white @ 40% | Placeholder text color |
| `caret-color` | same as color | Cursor / icon color |
| `accent-color` | blue @ 30% | Text selection highlight |

### Checkbox Properties

| Property | Default | States |
|---|---|---|
| `background-color` | transparent | :hover, :disabled, :checked |
| `accent-color` | white | Check icon color |
| `border-radius` | 0 | |
| `border-width` | 0 | |
| `border-color` | transparent | :hover, :disabled, :checked |
| `color` | none | Label text color |

### Toggler Properties

| Property | Default | States |
|---|---|---|
| `background-color` | transparent | :hover, :disabled, :checked |
| `color` | white | Foreground (knob) color |
| `border-width` | 0 | Track border width |
| `border-color` | transparent | Track border color |

The `color` property controls the knob; `background-color` controls the track.

### Radio Properties

| Property | Default | States |
|---|---|---|
| `background-color` | transparent | :hover, :checked |
| `accent-color` | white | Dot color |
| `border-width` | 0 | |
| `border-color` | transparent | :hover, :checked |
| `color` | none | Label text color |

### Slider Properties

| Property | Default | States |
|---|---|---|
| `background-color` | gray | Rail background (unfilled) |
| `accent-color` | blue | Rail foreground (filled) + handle |
| `border-width` | 0 | Handle border width |
| `border-color` | transparent | Handle border color |

### Progress Bar Properties

| Property | Default |
|---|---|
| `background-color` | gray | Track background |
| `accent-color` | blue | Fill bar color |
| `border-radius` | 0 | |

Progress bars are stateless.

### Pick List Properties

| Property | Default | States |
|---|---|---|
| `background-color` | transparent | :hover |
| `color` | white | Text color |
| `placeholder-color` | gray | Placeholder text color |
| `accent-color` | white | Handle (arrow) icon color |
| `border-radius` | 0 | |
| `border-width` | 0 | |
| `border-color` | transparent | :hover |

### Scrollable Properties

| Property | Default | States |
|---|---|---|
| `background-color` | transparent | Rail background |
| `accent-color` | gray | Scroller (thumb) color |
| `border-radius` | 0 | Scroller corner rounding |
| `border-width` | 0 | Scroller border width |
| `border-color` | transparent | Scroller border color |

### Rule (Divider) Properties

| Property | Default |
|---|---|
| `color` | gray | Line color |
| `border-radius` | 0 | Line corner rounding |

Rules are stateless.

---

## Rust API

### Loading a Theme

```rust
use icss::Theme;

let theme = Theme::load(include_str!("themes/dark.icss"))?;
```

### Styling Widgets

Each widget type has a corresponding method on `Theme`:

```rust
// Buttons — takes &[&str] classes, returns Fn(&Theme, Status) -> Style
button("Click").style(theme.button(&["button", "primary"]))

// Containers — stateless
container(content).style(theme.container(&["surface"]))

// Text input
text_input("placeholder", &value).style(theme.text_input(&["input"]))

// Checkbox
checkbox("Label", is_checked).style(theme.checkbox(&["checkbox"]))

// Toggler
toggler(is_toggled).style(theme.toggler(&["toggle"]))

// Radio
radio("Option", value, selected, on_select).style(theme.radio(&["radio"]))

// Slider
slider(range, value, on_change).style(theme.slider(&["slider"]))

// Progress bar
progress_bar(range, value).style(theme.progress_bar(&["progress"]))

// Pick list
pick_list(&options, selected, on_select).style(theme.pick_list(&["select"]))

// Scrollable
scrollable(content).style(theme.scrollable(&["scroll"]))

// Rule (divider)
rule::horizontal(1).style(theme.rule(&["divider"]))
```

### Composable Classes

Classes are building blocks. Combine them for variants:

```rust
// Base + color variant
theme.button(&["button", "primary"])
theme.button(&["button", "danger"])
theme.button(&["button", "success"])

// Base + color + size
theme.button(&["button", "primary", "small"])

// Conditional state class
let cls = if active { vec!["tab", "active"] } else { vec!["tab"] };
theme.button(&cls)
```

---

## Comments

Both block and line comments are supported:

```css
/* Block comment */
.button {
    // Line comment (non-standard but supported)
    color: white;
}
```

---

## Unknown Properties

Properties not recognized by the engine (e.g., `font-size`, `margin`) are:
- Parsed without error
- Stored internally (accessible for future use)
- A `tracing::warn!` is emitted at load time

This keeps `.icss` files forward-compatible.
