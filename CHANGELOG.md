# Changelog

## 0.1.1

Packaging and metadata fixes. No library code changes.

- README images and doc links are now absolute URLs. Relative paths were
  resolved against the package directory on crates.io, so the showcase gif
  and every `docs/` link 404ed on the crate page.
- `rust-version` corrected to 1.88. The declared 1.85 was never buildable:
  iced 0.14 and wgpu 27 both require 1.88.
- Added CI (build/test on Linux, macOS, Windows; fmt, clippy, MSRV check).
- Cleared all clippy warnings and applied rustfmt.

## 0.1.0

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
