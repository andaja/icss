# icss-showcase — Theme Showcase App

Interactive theme editor and component gallery. Lets you tweak theme inputs in
a sidebar and see every styled iced widget update live.

## Running

```bash
cargo run --release -p icss-showcase
```

Theme settings persist to `showcase-vars.conf` next to the binary. Delete the
file to reset to defaults. The app also writes `theme-dark.icss` /
`theme-light.icss` next to the binary unless the "Save .icss" toggle is off.

## Architecture

### Pipeline

```
ThemeVars (sidebar edits)
    |
    v
icss::engine::generate()
    |-- Derive signal colors (success, danger, warning) from 4 chromatic inputs
    |-- Generate 101-step tonal palettes per color (OKLab interpolation)
    |-- Map semantic tokens (surfaces, outlines, text contrast) for light/dark mode
    |-- Resolve dimensional tokens (spacing, sizing, radius, typography)
    |-- Compose ICSS string
    |
    v
icss::Theme::load(&icss)
    |-- Parse CSS rules, selectors, properties
    |-- Build class index for fast lookup
    |-- Create resolution cache
    |
    v
Widget styling via closures
    theme.button(&["button", "primary"])  -->  button::Style
    theme.container(&["section"])         -->  container::Style
```

Every edit in the sidebar triggers a full regeneration:
`ThemeVars -> ICSS -> Theme`.

### ThemeVars (editable inputs)

| Input | Default | Effect |
|-------|---------|--------|
| primary | `#1101CB` | Primary brand color |
| secondary | `#3DAAFA` | Secondary accent |
| tertiary | `#C42451` | Tertiary accent |
| quaternary | `#064E56` | Fourth chromatic color |
| neutral | `#8B959B` | Neutral/grey base (hue-tinted) |
| link | `#0D5A9E` | Link color |
| increment | 8 | Base spacing unit (px). All spacing multiplies from this |
| font_increment | 9 | Base font size unit (px). All type scales multiply from this |
| radius_factor | 1.6 | radius_increment = increment x factor |
| dark_mode | true | Light/dark semantic mapping |
| surface_lightness | 5 | Background lightness (0–100, maps to neutral palette step) |
| gamma | 1.0 | Tonal lightness curve exponent (<1 spreads darks, >1 spreads lights) |
| text_spread | 1.0 | On-surface text step-spacing multiplier |
| font_family | platform default | SF Pro (macOS) / Segoe UI (Windows) / Roboto (other) |

`surface_lightness`, `gamma`, and `text_spread` are stored per mode — the app
keeps separate `dark_*` and `light_*` values and swaps the active set when
`dark_mode` is toggled.

Signal colors (success, danger, warning) are **derived**, not editable — the
engine finds the nearest theme hue to green/red/yellow and clamps to safe
ranges. Each can be manually overridden via the corresponding `*_override`
field (empty → derive).

### Crate layout

Everything lives in one library crate, `icss`, with three modules:

**`icss::engine`** — Pure generation. Takes `ThemeInputs`, returns `ThemeOutput`
containing:
- `icss`: Complete ICSS theme string
- `dims`: Resolved `DimTokens` (10 spacing, 7 sizing, 8 radius, 19 typography scales)
- `derived_success/danger/warning`: Computed signal colors
- `neutral_palette`: 101 RGB values for visualization
- `scale_colors`: 20-swatch closest-path chromatic scale

Key modules: `tonal.rs` (tonal palettes), `signal.rs` (signal derivation),
`semantic.rs` (token mapping), `dims.rs` (dimensional scales),
`compose.rs` (ICSS output), `tailwind.rs` (Tailwind preset).

**`icss::theme`** — CSS parser + style resolver. Parses ICSS into an indexed
stylesheet. Provides typed closure factories per widget (`theme.button()`,
`theme.container()`, `theme.text_input()`, etc.). Resolution: find matching
rules by class subset, filter by pseudo-class, merge by specificity
`(pseudo_count, class_count)`, cache result.

**`icss::widgets`** — Reusable custom widgets used in the showcase:
- `DataTable` — sortable, paginated, searchable table with sticky header and row selection
- `TileGrid` — responsive tile grid with Flow/Horizontal/Vertical layouts
- `ButtonGroup` — mutually exclusive toggle buttons
- `ControlGroup` — grouped checkboxes/radios
- `IconInput` — text input with prefix/suffix icons
- `StickySection` — header that sticks during scroll (uses iced overlay system)
- `TabBar` — tabbed navigation
- `Menu` — dropdown menu
- `Animation` — tick-driven FadeIn/FadeOut/SlideIn/SlideOut with easing

### Module Layout

```
apps/showcase/src/
    main.rs          # State, Msg, boot/update/view, section view functions
    generate.rs      # ThemeVars -> icss::engine::generate() -> ThemeOutput
    persist.rs       # Save/load ThemeVars to showcase-vars.conf
    color_picker.rs  # HSV square + hue bar canvas widgets
```

### UI Structure

**Sidebar** (fixed left, own static theme):
- HSV color picker (saturation/value square + hue bar)
- 6 color swatches with hex inputs (click swatch to activate picker)
- 3 derived signal swatches (read-only, with optional override inputs)
- Dimensional sliders (increment, font_increment, radius_factor)
- Surface lightness / gamma / text-spread sliders
- Dark mode toggle
- Font family selector
- Restart button (applies font change, saves and exits)

**Content area** (scrollable, uses generated theme) with two pages:

**Components page** — interactive widget demos:
- Buttons: primary/success/danger/warning/ghost, sizes (md/sm/xs), pill, disabled states
- Emphasized buttons: gradient outline with hover/press states
- Inputs: text input, error state, combo box
- Button group, control group (checkboxes + radios)
- Controls: checkbox, toggler, radio
- Sliders, progress bars (4 variants)
- Pick list, text editor, tooltip
- Tile grid with layout switcher (Flow/Horizontal/Vertical)
- Data table with sort, search, pagination, row selection
- Typography scale (body/label/title/headline at all weights)
- Text colors on surface

**Primitives page** — raw design token visualization:
- Surface families: surface (s0–s5), tint, dark-tint, black — each showing text levels (text, default, soft, dim, muted, faint, disabled)
- Outline levels: subtle, soft, middle, strong, heavy, solid
- Chromatic families: primary, secondary, tertiary, quaternary (normal + container)
- Signal families: success, danger, warning (normal + container)
- Shadows: soft, medium, default (rendered examples)
- Typography: all font size tokens with weights
- Spacing: visual representation of all spacing tokens
- Neutral tonal palette: 101-step gradient bar
- Animations: fade and slide demos (left/top/right/bottom)

### Related Docs

- [ICSS.md](ICSS.md) — ICSS syntax spec, supported properties, specificity rules
- [theme-creation.md](theme-creation.md) — generative design-system architecture (tonal palettes, semantic mapping, composition)
- [COMPONENT-CATALOG.md](COMPONENT-CATALOG.md) — full class reference for every widget
