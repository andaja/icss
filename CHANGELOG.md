# Changelog

## 0.1.0 (unreleased)

First public release.

- `icss::theme`: `.icss` parser and style resolver for iced 0.14. Class
  conjunction, `:root` variables, 5 pseudo-classes.
- `icss::engine`: generative design system. 12 inputs produce a complete
  theme (OKLCH tonal palettes, semantic surface families, dimensional
  tokens).
- `icss::widgets`: `DataTable`, `TileGrid`, `ButtonGroup`, `ControlGroup`,
  `IconInput`, `StickySection`, `TabBar`.
- Showcase app with a live theme editor: `cargo run -p icss-showcase`.

Engine changes since the initial import:

- Outlines are cut per surface family from its own palette and emitted as
  `--outline-{family}-{level}`. The 4 solid levels scale to the room between
  the family base and the ramp end, so a mid-ramp family stays inside the
  ramp. The neutral page set is unchanged at the default surface steps.
- Chromatic surfaces keep the picked base step. The upper cap at step 50 is
  gone and the text direction is resolved per family. Signal surfaces keep
  the cap.
- The showcase Primitives page shows each family's outline row, and step
  numbers for containers and the neutral variants.
