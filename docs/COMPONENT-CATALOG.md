# Component Catalog

Machine-readable reference for styling iced widgets via the ICSS theme system.
**Source of truth:** `crates/icss/src/engine/compose.rs` generates all rules.
**Live preview:** `cargo run --release -p icss-showcase`.

## How to Use

Every widget is styled by passing class lists to the theme resolver:
```rust
let t = &state.theme;
button("Click").style(t.button(&["button", "primary"]))
container(content).style(t.container(&["section"]))
```

Classes compose conjunctively: `&["button", "primary", "small"]` merges rules from `.button`, `.primary`, `.button.primary`, and `.small`.

## Buttons — `t.button(&[...])`

Always include `"button"` as the first class.

| Classes | Purpose |
|---------|---------|
| `["button", "primary"]` | Brand-colored filled button with shadow |
| `["button", "success"]` | Positive/confirm action |
| `["button", "danger"]` | Destructive/delete action |
| `["button", "warning"]` | Caution action |
| `["button", "default"]` | Neutral outlined button |
| `["button", "ghost"]` | Transparent, text-only button |

**Size modifiers** — append to any variant:

| Modifier | Effect |
|----------|--------|
| `"small"` | Smaller border-radius (r-75) |
| `"tiny"` | Even smaller radius (r-50) |
| `"pill"` | Fully rounded ends (r-infinite) |
| `"round"` | Circular (r-infinite) |

**Size tokens** — for font/padding consistency, use `t.sizing()`:
```rust
let md = t.sizing(&["sz-md"]);  // font_size, pad_v, pad_h, gap, min_width
let sm = t.sizing(&["sz-sm"]);
let xs = t.sizing(&["sz-xs"]);
button(text("Go").size(md.font_size)).padding(md.padding())
```

**States:** `:hover`, `:active`, `:disabled` are resolved automatically from iced `Status`.

## Containers — `t.container(&[...])`

| Classes | Purpose |
|---------|---------|
| `["page"]` | Full page background (surface-s0) |
| `["section"]` | Elevated card (surface-s1, rounded) |

**Framed sections** — use `t.frame()` for section + body padding:
```rust
t.frame(
    t.column(&["subsection"])
        .push(t.text("Title", &["title-small"]))
        .push(content),
    &["section", "section-body"],
)
```

## Text — `t.text(&[...])`

Typography classes set font-size, color, and weight in one call.

**Headings:**

| Class | Weight | Use for |
|-------|--------|---------|
| `"headline-xlarge"` | 300 | Hero/display text |
| `"headline-large"` | 300 | Page hero |
| `"headline-medium"` | 400 | Large heading |
| `"headline-small"` | 400 | Section heading |
| `"headline-micro"` | 400 | Small heading |

**Titles:**

| Class | Weight | Use for |
|-------|--------|---------|
| `"title-large"` | 600 | Page titles |
| `"title-medium"` | 600 | Section titles |
| `"title-small"` | 600 | Subsection titles |

**Labels (UI text):**

| Class | Weight | Color | Use for |
|-------|--------|-------|---------|
| `"label-large"` | 600 | text-default | Large control labels |
| `"label-medium"` | 600 | text-default | Standard control labels |
| `"label-small"` | 600 | text-default | Small labels, field descriptions |
| `"label-micro"` | 600 | text-soft | Tiny labels, badges, annotations |

**Body (reading text):**

| Class | Weight | Color | Use for |
|-------|--------|-------|---------|
| `"body-large"` | 400 | text-default | Long-form content |
| `"body-medium"` | 400 | text-default | Standard paragraphs |
| `"body-small"` | 400 | text-default | Compact content |
| `"body-micro"` | 400 | text-soft | Fine print |

**Caption:**

| Class | Weight | Color |
|-------|--------|-------|
| `"caption"` | 400 | text-soft |

**Color modifiers** — append to any text class:

| Modifier | Color variable |
|----------|---------------|
| `"text-default"` | text-default |
| `"text-soft"` | text-soft |
| `"text-muted"` | text-muted |
| `"text-faint"` | text-faint |
| `"text-primary"` | surface-primary-s0 |
| `"text-secondary"` | surface-secondary-s0 |
| `"text-tertiary"` | surface-tertiary-s0 |
| `"text-quaternary"` | surface-quaternary-s0 |
| `"text-link"` | surface-link-s0 |
| `"text-success"` | surface-success-s0 |
| `"text-danger"` | surface-danger-s0 |
| `"text-warning"` | surface-warning-s0 |

Example: `t.text("Error", &["label-small", "text-danger"])`

## Text Input — `t.text_input(&[...])`

| Classes | Purpose |
|---------|---------|
| `["input"]` | Standard text input |
| `["input", "sz-md"]` | Medium input (with sizing) |
| `["input", "sz-sm"]` | Small input |
| `["input", "sz-xs"]` | Extra-small input |
| `["input", "error"]` | Error state (red border) |

States: `:hover`, `:focus`, `:disabled` automatic.

## Text Editor — `t.text_editor(&[...])`

| Classes | Purpose |
|---------|---------|
| `["editor", "sz-md"]` | Multiline text editor |

States: `:hover`, `:focus` automatic.

## Checkbox — `t.checkbox(&[...])`

| Classes | Purpose |
|---------|---------|
| `["checkbox", "sz-md"]` | Medium checkbox |
| `["checkbox", "sz-sm"]` | Small checkbox |
| `["checkbox", "sz-xs"]` | Extra-small checkbox |

Pair with sizing for label alignment:
```rust
let md = t.sizing(&["sz-md"]);
checkbox(is_checked).label("Accept")
    .size(md.font_size).text_size(md.font_size).spacing(md.gap)
    .on_toggle(Msg::Toggle).style(t.checkbox(&["checkbox", "sz-md"]))
```

States: `:hover`, `:checked`, `:disabled` automatic.

## Toggler �� `t.toggler(&[...])`

| Classes | Purpose |
|---------|---------|
| `["toggle"]` | Standard toggle switch |

States: `:hover`, `:checked`, `:disabled` automatic.

## Radio — `t.radio(&[...])`

| Classes | Purpose |
|---------|---------|
| `["radio"]` | Standard radio button |

States: `:hover`, `:checked` automatic.

## Slider — `t.slider(&[...])`

| Classes | Purpose |
|---------|---------|
| `["slider"]` | Standard slider |

States: `:hover` automatic.

## Progress Bar — `t.progress_bar(&[...])`

| Classes | Purpose |
|---------|---------|
| `["progress"]` | Default progress bar (primary accent) |
| `["progress", "success"]` | Green progress |
| `["progress", "danger"]` | Red progress |
| `["progress", "warning"]` | Yellow progress |

## Pick List (Dropdown) — `t.pick_list(&[...])` + `t.menu(&[...])`

Always pair with a menu style:
```rust
pick_list(options, selected, on_select)
    .style(t.pick_list(&["select", "sz-md"]))
    .menu_style(t.menu(&["select-menu", "sz-md"]))
```

| Pick list classes | Menu classes | Size |
|-------------------|-------------|------|
| `["select", "sz-md"]` | `["select-menu", "sz-md"]` | Medium |
| `["select", "sz-sm"]` | `["select-menu", "sz-sm"]` | Small |
| `["select", "sz-xs"]` | `["select-menu", "sz-xs"]` | Extra-small |

States: `:hover`, `:disabled` automatic.

## Scrollable — `t.scrollable(&[...])`

| Classes | Purpose |
|---------|---------|
| `["scroll"]` | Standard scrollbar styling |

## Rule (Divider) — `t.rule(&[...])`

| Classes | Purpose |
|---------|---------|
| `["divider"]` | Horizontal or vertical divider line |

```rust
rule::horizontal(1).style(t.rule(&["divider"]))
```

## Layout Helpers

**Rows** — `t.row(&[...])`:

| Classes | Gap |
|---------|-----|
| `["row"]` | space-100 (standard) |
| `["row-tight"]` | space-75 |
| `["row-loose"]` | space-200 |
| `["cluster"]` | space-50 |
| `["field-row"]` | space-100 |

**Columns** — `t.column(&[...])`:

| Classes | Gap |
|---------|-----|
| `["stack"]` | space-100 (standard) |
| `["stack-tight"]` | space-75 |
| `["stack-loose"]` | space-200 |
| `["subsection"]` | space-100 |
| `["field-col"]` | space-50 |

**Page/section body** — used with `t.frame()`:

| Classes | Padding | Gap |
|---------|---------|-----|
| `["section-body"]` | space-200 | space-150 |
| `["page-body"]` | space-200 | space-300 |

## Color Variables — `t.color_var("...")`

For cases where you need a raw `iced::Color` (e.g. dynamic alpha, canvas drawing):

**Text hierarchy:**
`"text"`, `"text-default"`, `"text-soft"`, `"text-muted"`, `"text-faint"`, `"text-disabled"`

**Surfaces:** `"surface-s0"` through `"surface-s5"`

**Outlines:** `"outline-subtle"`, `"outline-soft"`, `"outline-middle"`, `"outline-strong"`, `"outline-heavy"`, `"outline-solid"`

**Chromatic surfaces:** `"surface-primary-s0"` through `"surface-primary-s4"` (same pattern for secondary, tertiary, quaternary)

**Signal surfaces:** `"surface-success-s0"` through `"surface-success-s4"` (same for danger, warning)

**On-surface text (for colored backgrounds):** `"on-surface-primary"`, `"on-surface-primary-default"`, `"on-surface-primary-soft"` (same pattern for success, danger, warning, secondary, tertiary, quaternary)

**Container surfaces:** `"surface-primary-container-s0"` through s4 (same for other families)

**Shadows:** `"shadow"`, `"shadow-medium"`, `"shadow-soft"`

Always provide a fallback: `t.color_var("surface-success-s0").unwrap_or(Color::from_rgb(0.2, 0.8, 0.4))`

## Custom Widgets (icss::widgets)

These accept a `&Theme` reference and style themselves internally:

- **`TileGrid`** — `grid.view(t)` — responsive tile layout
- **`DataTable`** — `table.view(t)` — sortable, paginated table
- **`ButtonGroup`** — segmented toggle buttons
- **`ControlGroup`** — grouped checkboxes/radios
- **`IconInput`** — text input with prefix/suffix icons
- **`StickySection`** — sticky scroll header
