# Theme Creation — Generative Design System

This document describes how the ICSS theme engine turns a small set of inputs
into a complete `.icss` stylesheet. It tracks the actual implementation in
[`crates/icss/src/engine/`](../crates/icss/src/engine/) — not a pre-implementation
spec.

The engine is a five-stage pipeline. Each stage is deterministic: the same
inputs always produce the same output.

```
signal  →  tonal  →  dims  →  semantic  →  compose
```

| Stage | Module | Responsibility |
|---|---|---|
| signal | [`signal.rs`](../crates/icss/src/engine/signal.rs) | Derive success/danger/warning colors from the chromatic inputs |
| tonal | [`tonal.rs`](../crates/icss/src/engine/tonal.rs) | Generate 101-step luminosity palettes |
| dims | [`dims.rs`](../crates/icss/src/engine/dims.rs) | Resolve spacing/sizing/radius/typography from increments |
| semantic | [`semantic.rs`](../crates/icss/src/engine/semantic.rs) | Map palette steps to UI roles (surfaces, text, outlines) |
| compose | [`compose.rs`](../crates/icss/src/engine/compose.rs) | Emit the `.icss` stylesheet |

`engine::generate(&ThemeInputs) -> ThemeOutput` (in
[`engine.rs`](../crates/icss/src/engine.rs)) runs the whole pipeline.

---

## 1. Inputs

[`ThemeInputs`](../crates/icss/src/engine.rs) collects everything the engine needs.

**Colors (6 hex strings):**

| Input | Default | Role |
|---|---|---|
| `primary` | `#1101CB` | Brand, primary actions |
| `secondary` | `#3DAAFA` | Secondary actions |
| `tertiary` | `#C42451` | Accent |
| `quaternary` | `#064E56` | Accent 2 |
| `neutral` | `#8B959B` | Surfaces, text, outlines |
| `link` | `#0D5A9E` | Link text |

The three signal colors (success / danger / warning) are **not** inputs — they
are derived from the 4 chromatic colors (see §2). Each can be manually
overridden via `success_override` / `danger_override` / `warning_override`
(a `#RRGGBB` string; anything else falls back to the derived value).

**Dimensions (3 numbers):**

| Input | Default | Role |
|---|---|---|
| `increment` | `8.0` | Base unit for all spacing and sizing |
| `font_increment` | `9.0` | Base unit for all font sizes (independent of `increment`) |
| `radius_factor` | `1.6` | Multiplier: `radius_increment = increment × radius_factor` |

**Mode and tuning:**

| Input | Default | Role |
|---|---|---|
| `dark_mode` | `true` | Light or dark resolution |
| `surface_lightness` | `None` | Override the base surface tonal step (see §5) |
| `gamma` | `None` → `1.0` | Lightness curve exponent (see §3) |
| `text_spread` | `None` → `1.0` | On-surface text step-spacing multiplier (see §5) |

That's ~12 values. From them the engine produces 9 tonal palettes (909 colors),
the full set of semantic color tokens, the dimensional tokens, and a complete
component stylesheet.

---

## 2. Signal Color Derivation — `signal.rs`

Success, danger, and warning are "softly wired" to the 4 chromatic inputs so
that signals stay on-brand while preserving traditional color coding.

For each signal, `derive_signals` finds the chromatic input whose OKLCH hue is
closest to the signal's target hue range:

| Signal | Hue range (OKLCH°) | Default hue |
|---|---|---|
| success | 140–170 (green) | 155 |
| danger | 15–40 (red) | 25 |
| warning | 60–100 (yellow/amber) | 80 |

- If the nearest input hue is **inside** the range, it is used directly.
- If it is **within 40°**, it is clamped to the nearest range edge.
- If it is **further than 40°**, the default hue is used.

The chosen hue is rendered at OKLCH lightness `0.65` with chroma at 90% of the
maximum in-gamut value (never below `MIN_SIGNAL_CHROMA = 0.14`) — so signals
stay vivid regardless of how saturated the theme colors are.

---

## 3. Tonal Palette Generation — `tonal.rs`

### Goal

Given one base color, produce a 101-step luminosity scale, step `0` = black
through step `100` = white, preserving the base color's hue and chroma while
varying lightness. `TonalPalette` holds exactly `[Srgb<f32>; 101]` — there is
no separate named "base" entry; the base color's natural step is *computed*
on demand by `TonalPalette::base_step`.

### Color space

Interpolation happens in **OKLab**. Each step is a straight LAB-space lerp
between the input color and an anchor — white (`L=1`) for lighter steps, black
(`L=0`) for darker steps — indexed by a target lightness. Because both `a` and
`b` fade toward 0 as the lerp approaches an anchor, the scale naturally
desaturates at the extremes with no separate envelope or saturation cap. This
mirrors `chroma.js`'s `scale([white, input, black]).get('lab.l')`.

OKLCH is used only where polar coordinates are needed: gamut mapping
(`gamut_map`, `max_srgb_chroma`) and hue math.

### Lightness curve

The mapping from step index to target lightness is a single power curve:

```
ease_lightness(t, gamma) = t ^ gamma          (t = step / 100)
```

- `gamma = 1.0` — linear (default).
- `gamma < 1.0` — spreads the dark end (wider perceptual gaps in shadows).
- `gamma > 1.0` — spreads the light end.

There is **no fixed elevation ladder and no hardcoded per-step gap table** —
the curve is the single source of step distribution, and the showcase exposes
`gamma` as a slider.

### Neutral correction

Pure mathematical greys read warm/brown at mid-dark lightness.
`generate_neutral_palette` applies a hue shift toward teal (200° in OKLCH).
The correction strength follows `teal_correction_weight(t)`, a parabola
peaking at `t ≈ 0.35` (around step 35) and falling to zero at the extremes.
The shift applied is `(200 - base_hue) × weight × 0.15`, plus a small chroma
floor so near-grey inputs still pick up the tint.

### Closest-path scale

`closest_path_scale` produces an evenly-spaced chromatic scale across the 4
chromatic anchors at a constant tonal step. It sorts the anchor hues around the
wheel, drops the single largest empty arc, and distributes `N` swatches with
equal hue spacing along the remaining path — so consecutive swatches are always
close in hue. The engine computes this at step 50 with
`SCALE_COLORS_COUNT = 20` swatches and exposes them as `--scale-colors-0` …
`--scale-colors-19` in the output (plus `--scale-colors-count`). Consumers that
need a stable per-entity tint set (e.g. device tiles) should read these vars
rather than recomputing.

---

## 4. Dimensional System — `dims.rs`

All spatial values derive from three roots by multiplication. `DimTokens::resolve`
takes `DimInputs { increment, font_increment, radius_factor }` and rounds every
result to the nearest integer.

**Roots:**

- `increment` — base unit for spacing and sizing.
- `font_increment` — base unit for font sizes, independent of `increment`.
- `radius_increment = increment × radius_factor` — embeds the rounding character.

**Scales** (multiplier × root, rounded):

| Scale | Root | Tokens (multiplier) |
|---|---|---|
| Spacing | `increment` | `space_25` (0.25), `_50` (0.5), `_75` (0.75), `_100` (1.0), `_150` (1.5), `_200` (2.0), `_250` (2.5), `_300` (3.0), `_400` (4.0), `_500` (5.0) |
| Sizing | `increment` | `size_100` (1.0), `_150` (1.5), `_200` (2.0), `_250` (2.5), `_300` (3.0), `_400` (4.0), `_500` (5.0) |
| Radius | `radius_increment` | `radius_0` & `radius_25` (0.25, floored to ≥1px when nonzero), `_50` (0.5), `_75` (0.75), `_100` (1.0), `_150` (1.5), `_200` (2.0), `radius_infinite` (fixed `1000`) |
| Typography | `font_increment` | see below |

**Typography multipliers** (× `font_increment`):

| Group | Tokens (multiplier) |
|---|---|
| body | micro 1.25, small 1.5, medium-small 1.625, medium 1.75, large 2.0 |
| label | micro 1.25, small 1.5, medium 1.75, large 2.0 |
| title | small 2.25, medium 2.75, large 3.0 |
| headline | micro 2.5, small 3.0, medium 3.5, large 4.5, x-large 5.5, xx-large 7.0, xxx-large 9.0 |

`label` and `body` share size values at matching names — the distinction is
weight/semantics, applied at the component layer, not dimension. Weights are
not stored as tokens; they are assigned per component class in `compose.rs`
(see COMPONENT-CATALOG.md).

When `radius_factor = 0`, every radius token collapses to `0` (sharp corners);
larger factors round everything proportionally. Swapping any of the three
roots reshapes the whole UI without touching component definitions.

---

## 5. Semantic Color Mapping — `semantic.rs`

`SemanticColors::resolve` maps tonal-palette steps to UI roles for the chosen
mode. There is no static light/dark index table and no `dark = 100 - light`
inversion enum — surfaces are computed from a base step and a direction.

### Surface base step

The base surface step (`srf`) is `surface_lightness` if provided, otherwise the
mode default:

- **Dark mode:** step `5`
- **Light mode:** step `95`

`is_dark_surface = srf < 50` then drives the elevation direction
(`step_dir = +1` for dark surfaces, `-1` for light) and the text end
(light text from step 100 for dark surfaces, dark text from step 0 for light).

> **Known code inconsistency.** `semantic.rs` resolves light mode at step `95`,
> but `engine.rs` reports `actual_surface` (the value shown in the UI) as `97`
> for light mode, and the `ThemeInputs::surface_lightness` doc comment says
> `97 / 3`. The value that actually drives color resolution is `5` (dark) /
> `95` (light). The `97` and `3` figures should be reconciled in the code.

### Elevation ladder

A `SurfaceFamily` carries 5 surface steps `s0`–`s4` plus 7 text levels. Each
step is computed:

```
step(offset) = clamp(base + offset × step_size × step_dir, 0, 100)
```

with `step_size = 3`. Uniform offsets are intentional — the `ease_lightness`
curve already provides wider perceptual gaps at the dark end, so the index math
stays linear. The neutral surface additionally exposes `surface_s5` (offset 5),
giving it 6 steps.

### Text levels

Each family resolves 7 text colors: `text`, `text_default`, `text_soft`,
`text_dim`, `text_muted`, `text_faint` (6 solid steps from the palette) and
`text_disabled` (the `text_default` step at 50% alpha). Text levels spread out
from the extreme end of the scale (100 for light text, 0 for dark); the gap
between adjacent levels is scaled by `text_spread` (1.0 = mirror the surface
spacing, exposed as a showcase slider).

### Outlines

Six solid neutral outlines — `subtle`, `soft`, `middle`, `strong`, `heavy`,
`solid` — at offsets 1, 3, 7, 11, 15, 20 from the surface base, plus two
alpha-based variants (`subtle_alpha` at 8%, `soft_alpha` at 15%). The alpha
outlines work across a wider range of backgrounds than a fixed step pick.

### Surface families

`resolve` produces these families, all from the same algorithm:

- **Neutral:** `surface`, `tint`, `dark_tint`, `black`. `tint`/`dark_tint`
  base steps depend on mode; `black` stays dark in both modes.
- **Chromatic:** `primary`, `secondary`, `tertiary`, `quaternary`, each with a
  matching pastel `*_container`.
- **Signal:** `success`, `danger`, `warning`, each with a `*_container`.

Chromatic and signal surfaces start from the user color's own natural step
(`base_step`), clamped: never lighter than step 50 (so light text always
works), and in dark mode never lighter than `min(srf + 20, 40)` (so buttons
stay distinguishable from the page). Signal surfaces are additionally darkened
by 10 steps. Containers use base step 25 (dark) or 80 (light).

Text-contrast direction is decided by the clamped step (`step ≤ 55` → light
text), not by gamma-sensitive OKLCH lightness, so text direction stays stable
when `gamma` changes.

### Accents and shadows

`on_surface_*` accent colors place each chromatic/signal hue *on* the neutral
surface, nudged a couple of steps for visibility. Shadow colors are pure black
in dark mode; in light mode they are low-alpha neutral tints
(`shadow`, `shadow_medium`, `shadow_soft` — exactly three).

---

## 6. Composition — `compose.rs`

`compose_icss` emits the final stylesheet: a `:root` block of CSS custom
properties followed by component rule sets.

**`:root` exposes:**

- Every surface family as `--{prefix}-s0..s4`, `--on-{prefix}`,
  `--on-{prefix}-default/soft/dim/muted/faint/disabled`.
- Outline, shadow, and `--scale-colors-N` variables.
- All dimensional tokens as `--space-*`, `--size-*`, `--radius-*`, `--font-*`.

**Component rules** are plain CSS-like classes (`.button`, `.primary`,
`.input`, `.checkbox`, `.toggle`, `.tooltip`, `.toast`, `.sz-md`, …) with
`:hover` / `:active` / `:focus` / `:checked` / `:disabled` pseudo-states.
Classes compose conjunctively — e.g. `["button", "primary", "small"]` merges
`.button`, `.primary`, `.button.primary`, and `.small`.

There are three emitted size classes — `.sz-xs`, `.sz-sm`, `.sz-md` — carrying
padding, font-size, gap, and min-width.

The generated stylesheet is the authority for what classes exist and what they
do. For the consumer-facing catalog of components and their class lists, see
[COMPONENT-CATALOG.md](./COMPONENT-CATALOG.md). For the live editor that drives
`ThemeInputs` interactively, see [SHOWCASE.md](./SHOWCASE.md).

### Tailwind preset

`tailwind.rs` (`compose_tailwind_preset`, invoked by the `gen_tailwind` binary)
emits a `tailwind.preset.js` that maps the semantic tokens onto standard
Tailwind palette slots (`slate`, `blue`, `green`, `red`, `amber`, `purple`), so
a web app can use Tailwind classes drawn from the same generated theme.

---

## 7. Summary

From ~12 inputs the engine deterministically produces:

- 9 tonal palettes × 101 steps = 909 colors (6 from inputs, 3 derived signals).
- The full semantic color token set (surfaces, text levels, outlines, accents,
  shadows), resolved per light/dark mode.
- ~30 dimensional tokens (spacing, sizing, radius, typography).
- A complete `.icss` component stylesheet.
- A closest-path 20-swatch chromatic scale.
- Optionally, a Tailwind preset.

Editing any input — a base color, an increment, the mode — regenerates the
entire theme consistently, with no per-component edits.
