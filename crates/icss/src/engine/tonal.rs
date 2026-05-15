//! Tonal palette generation — 101-step luminosity scales in OKLCH.
//!
//! Given a base color, produces steps 0 (black) through 100 (white)
//! preserving the hue and saturation while varying lightness.

use palette::convert::{FromColorUnclamped, IntoColorUnclamped};
use palette::{Oklab, Oklch, Srgb};

/// A tonal palette: 101 sRGB colors indexed by luminosity step.
/// Step 0 = black, step 100 = white, base hue at ~step 50.
#[derive(Clone)]
pub struct TonalPalette {
    pub steps: [Srgb<f32>; 101],
}

impl TonalPalette {
    /// Find the tonal step closest to a given base hex color.
    ///
    /// Returns 0–100. Useful for determining the "natural" step of
    /// the user's chosen color within its own tonal scale.
    pub fn base_step(base_hex: &str, gamma: f32) -> usize {
        let base = parse_hex(base_hex);
        let base_oklch: Oklch = Oklch::from_color_unclamped(base);
        let base_l = base_oklch.l;

        let mut best_step = 50;
        let mut best_dist = f32::MAX;
        for i in 0..=100 {
            let step_l = ease_lightness(i as f32 / 100.0, gamma);
            let dist = (step_l - base_l).abs();
            if dist < best_dist {
                best_dist = dist;
                best_step = i;
            }
        }
        best_step
    }

    /// Get the OKLCH lightness of a palette step.
    ///
    /// Used for perceptual text contrast decisions instead of
    /// raw step numbers or WCAG 2 relative luminance.
    pub fn oklch_lightness(&self, step: usize) -> f32 {
        let c = self.steps[step.min(100)];
        let oklch: Oklch = Oklch::from_color_unclamped(c);
        oklch.l
    }

    /// Get a step as an `#RRGGBB` hex string.
    pub fn hex(&self, step: usize) -> String {
        let c = self.steps[step.min(100)];
        let r = (c.red.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (c.green.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (c.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    /// OKLCH hue in degrees for a base hex color.
    pub fn hue_of(base_hex: &str) -> f32 {
        let base = parse_hex(base_hex);
        let oklch: Oklch = Oklch::from_color_unclamped(base);
        oklch.hue.into_inner()
    }

    /// Get a step with alpha as `rgba(r, g, b, a)` string.
    pub fn rgba(&self, step: usize, alpha: f32) -> String {
        let c = self.steps[step.min(100)];
        let r = (c.red.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (c.green.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (c.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("rgba({r}, {g}, {b}, {alpha:.2})")
    }
}

/// Generate a tonal palette from a base hex color (e.g., "#EF443B").
///
/// The base color's hue and chroma are preserved. Lightness is
/// distributed across 0–100 with wider perceptual gaps at the dark end.
pub fn generate_tonal_palette(base_hex: &str, gamma: f32) -> TonalPalette {
    let base_rgb = parse_hex(base_hex);
    let base_oklab: Oklab = Oklab::from_color_unclamped(base_rgb);
    let base_l = base_oklab.l.clamp(0.001, 0.999);

    // Anchor endpoints: white (L=1) → input → black (L=0). Each step's
    // colour is a straight LAB-space lerp between input and the relevant
    // endpoint, indexed by target lightness. Same idea as
    // chroma.js's `chroma.scale([white, input, black]).get('lab.l')` —
    // chroma naturally fades to 0 at both extremes because we're
    // interpolating a→0 and b→0 along the way; no separate envelope or
    // saturation cap needed, no gamut-clamp gymnastics.
    let white = Oklab::new(1.0, 0.0, 0.0);
    let black = Oklab::new(0.0, 0.0, 0.0);

    let mut steps = [Srgb::new(0.0f32, 0.0, 0.0); 101];

    for (i, step) in steps.iter_mut().enumerate() {
        let t = i as f32 / 100.0;
        // Non-linear lightness curve: wider gaps at dark end. (gamma=1
        // is the linear default; the showcase exposes this slider.)
        let l_target = ease_lightness(t, gamma);

        let lab = if l_target >= base_l {
            let alpha = ((l_target - base_l) / (1.0 - base_l)).clamp(0.0, 1.0);
            lerp_oklab(base_oklab, white, alpha)
        } else {
            let alpha = ((base_l - l_target) / base_l).clamp(0.0, 1.0);
            lerp_oklab(base_oklab, black, alpha)
        };

        let rgb: Srgb<f32> = lab.into_color_unclamped();
        *step = Srgb::new(
            rgb.red.clamp(0.0, 1.0),
            rgb.green.clamp(0.0, 1.0),
            rgb.blue.clamp(0.0, 1.0),
        );
    }

    // Force exact endpoints.
    steps[0] = Srgb::new(0.0, 0.0, 0.0);
    steps[100] = Srgb::new(1.0, 1.0, 1.0);

    TonalPalette { steps }
}

/// Linear interpolation in OKLab space.
fn lerp_oklab(a: Oklab, b: Oklab, t: f32) -> Oklab {
    Oklab::new(
        a.l + (b.l - a.l) * t,
        a.a + (b.a - a.a) * t,
        a.b + (b.b - a.b) * t,
    )
}

/// Generate a neutral palette with teal-bias correction.
///
/// Pure mathematical greys look warm/brown in mid-dark shades.
/// This adds a slight hue shift toward teal (~200° in OKLCH)
/// strongest around steps 30–50, tapering at extremes.
pub fn generate_neutral_palette(base_hex: &str, gamma: f32) -> TonalPalette {
    let base_rgb = parse_hex(base_hex);
    let base_oklch: Oklch = Oklch::from_color_unclamped(base_rgb);
    let base_hue_deg = base_oklch.hue.into_inner();
    let base_chroma = base_oklch.chroma;
    let base_l = base_oklch.l.clamp(0.001, 0.999);

    let white = Oklab::new(1.0, 0.0, 0.0);
    let black = Oklab::new(0.0, 0.0, 0.0);

    let mut steps = [Srgb::new(0.0f32, 0.0, 0.0); 101];

    for (i, step) in steps.iter_mut().enumerate() {
        let t = i as f32 / 100.0;
        let l_target = ease_lightness(t, gamma);

        // Teal correction: shift hue toward 200° (cool grey) most around
        // steps 30–50, tapering at extremes — applied to a "reference
        // input" Oklab that the lerp bends toward, keeping the same lerp
        // mechanics as the chromatic palette.
        let correction_strength = teal_correction_weight(t);
        let teal_hue: f32 = 200.0;
        let hue_shift = (teal_hue - base_hue_deg) * correction_strength * 0.15;
        let corrected_hue = base_hue_deg + hue_shift;
        let h_rad = corrected_hue.to_radians();
        let chroma_floor = 0.005 * correction_strength;
        let input = Oklab::new(
            base_l,
            base_chroma.max(chroma_floor) * h_rad.cos(),
            base_chroma.max(chroma_floor) * h_rad.sin(),
        );

        let lab = if l_target >= base_l {
            let alpha = ((l_target - base_l) / (1.0 - base_l)).clamp(0.0, 1.0);
            lerp_oklab(input, white, alpha)
        } else {
            let alpha = ((base_l - l_target) / base_l).clamp(0.0, 1.0);
            lerp_oklab(input, black, alpha)
        };

        let rgb: Srgb<f32> = lab.into_color_unclamped();
        *step = Srgb::new(
            rgb.red.clamp(0.0, 1.0),
            rgb.green.clamp(0.0, 1.0),
            rgb.blue.clamp(0.0, 1.0),
        );
    }

    steps[0] = Srgb::new(0.0, 0.0, 0.0);
    steps[100] = Srgb::new(1.0, 1.0, 1.0);

    TonalPalette { steps }
}

/// Lightness mapping — single power curve controlled by gamma.
///
/// gamma=1.0: linear. <1 spreads dark end. >1 spreads light end.
fn ease_lightness(t: f32, gamma: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    t.powf(gamma)
}

/// Public gamut mapping entry point for use outside this module.
pub fn gamut_map_pub(l: f32, target_chroma: f32, hue: f32) -> Srgb<f32> {
    gamut_map(l, target_chroma, hue)
}

/// Generate a "closest-path" scale over a set of anchor hues at constant
/// OKLCH lightness.
///
/// Sorts the anchor hues around the color wheel, then drops the single
/// largest arc between adjacent anchors (the biggest empty segment), and
/// distributes `n` swatches with equal hue-angle spacing along the remaining
/// path — so consecutive colors are always close in hue and no long empty
/// stretch is crossed. Returns `(color, hue_deg, is_anchor)` per swatch.
/// An anchor-slot is flagged when a swatch lands near one of the input hues.
///
/// Lightness comes from `ease_lightness(step/100, gamma)` so the scale
/// matches the tonal-palette step. Chroma is the maximum in-gamut value at
/// each hue (saturation peaks at every slot).
pub fn closest_path_scale(
    anchor_hexes: &[&str],
    step: usize,
    n: usize,
    gamma: f32,
) -> Vec<(Srgb<f32>, f32, bool)> {
    if n == 0 || anchor_hexes.is_empty() {
        return Vec::new();
    }
    let t = (step.min(100) as f32) / 100.0;
    let l = ease_lightness(t, gamma);

    // Average parent chroma across the anchors — bounds each wheel slot's
    // saturation so it matches the parents' overall vibrancy instead of
    // sitting on the sRGB gamut ceiling at every hue. `gamut_map`
    // additionally clamps when this target exceeds what's achievable at
    // the slot's hue/lightness.
    let parent_chroma = avg_oklch_chroma(anchor_hexes);

    // Gather anchor hues, normalised to [0, 360).
    let mut hues: Vec<f32> = anchor_hexes
        .iter()
        .map(|h| {
            let mut deg = TonalPalette::hue_of(h);
            deg = ((deg % 360.0) + 360.0) % 360.0;
            deg
        })
        .collect();
    hues.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Single anchor → fall back to a full 360° wheel starting at that hue.
    if hues.len() == 1 {
        let start = hues[0];
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let hue = (start + (i as f32) * 360.0 / (n as f32)) % 360.0;
            let chroma = parent_chroma.min(max_srgb_chroma(l, hue));
            out.push((gamut_map(l, chroma, hue), hue, i == 0));
        }
        return out;
    }

    // Compute arc gaps between consecutive sorted hues (with wrap).
    let k = hues.len();
    let mut gaps = Vec::with_capacity(k);
    for i in 0..k {
        let next = hues[(i + 1) % k];
        let here = hues[i];
        let gap = if (i + 1) % k == 0 {
            (next + 360.0) - here
        } else {
            next - here
        };
        gaps.push(gap);
    }
    // Drop the largest gap — walk the path starting from the anchor right
    // after it.
    let max_idx = gaps
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let start_idx = (max_idx + 1) % k;

    // Build the ordered anchor list along the retained path, unwrapping hue
    // so it's strictly increasing.
    let mut path: Vec<f32> = Vec::with_capacity(k);
    path.push(hues[start_idx]);
    for step_i in 1..k {
        let cur = hues[(start_idx + step_i) % k];
        let prev = *path.last().unwrap();
        let mut h = cur;
        while h < prev {
            h += 360.0;
        }
        path.push(h);
    }

    let total_span = *path.last().unwrap() - path[0];
    // Degenerate (all anchors same hue) — fall back to even wheel.
    if total_span <= f32::EPSILON {
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let hue = (path[0] + (i as f32) * 360.0 / (n as f32)) % 360.0;
            let chroma = parent_chroma.min(max_srgb_chroma(l, hue));
            out.push((gamut_map(l, chroma, hue), hue, false));
        }
        return out;
    }

    let anchor_tol = if n > 1 {
        (total_span / ((n - 1) as f32)) * 0.5
    } else {
        total_span
    };

    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let u = if n == 1 {
            0.0
        } else {
            (i as f32) / ((n - 1) as f32)
        };
        let hue_unwrapped = path[0] + u * total_span;
        let hue = ((hue_unwrapped % 360.0) + 360.0) % 360.0;
        let is_anchor = path
            .iter()
            .any(|a| ((*a - hue_unwrapped).abs()) <= anchor_tol);
        let chroma = parent_chroma.min(max_srgb_chroma(l, hue));
        out.push((gamut_map(l, chroma, hue), hue, is_anchor));
    }
    out
}

/// Mean OKLCH chroma across a slice of hex colours. `0.0` for an empty
/// slice. Used to bound wheel/scale saturation by the parents' average
/// vibrancy rather than the sRGB gamut ceiling.
fn avg_oklch_chroma(hexes: &[&str]) -> f32 {
    if hexes.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0_f32;
    for h in hexes {
        let rgb = parse_hex(h);
        let oklch: Oklch = Oklch::from_color_unclamped(rgb);
        sum += oklch.chroma;
    }
    sum / hexes.len() as f32
}

/// Generate a color wheel at constant OKLCH lightness with evenly spaced hues.
///
/// `step` is the target tonal step (0–100) — lightness is taken from the
/// same `ease_lightness(step/100, gamma)` curve used by the palettes so
/// that colors visually match step-50 (or whichever step is chosen) of
/// existing palettes. Chroma at each hue is bounded by the average OKLCH
/// chroma of `anchor_hexes` (so the wheel respects the parents' overall
/// saturation rather than maxing out at the sRGB gamut ceiling). Pass
/// an empty `anchor_hexes` slice for the historical max-chroma behaviour.
/// Starting hue defaults to 0° (red); `n` colors are produced at 360°/n
/// spacing.
pub fn color_wheel(anchor_hexes: &[&str], step: usize, n: usize, gamma: f32) -> Vec<Srgb<f32>> {
    if n == 0 {
        return Vec::new();
    }
    let t = (step.min(100) as f32) / 100.0;
    let l = ease_lightness(t, gamma);
    let parent_chroma = if anchor_hexes.is_empty() {
        f32::INFINITY
    } else {
        avg_oklch_chroma(anchor_hexes)
    };
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let hue = (i as f32) * 360.0 / (n as f32);
        let chroma = parent_chroma.min(max_srgb_chroma(l, hue));
        let rgb = gamut_map(l, chroma, hue);
        out.push(Srgb::new(
            rgb.red.clamp(0.0, 1.0),
            rgb.green.clamp(0.0, 1.0),
            rgb.blue.clamp(0.0, 1.0),
        ));
    }
    out
}

/// Find the maximum OKLCH chroma that fits sRGB at a given lightness and hue.
pub fn max_srgb_chroma(l: f32, hue: f32) -> f32 {
    let hue_val: palette::OklabHue<f32> = hue.into();
    let mut lo = 0.0_f32;
    let mut hi = 0.4; // OKLCH chroma rarely exceeds 0.4 in sRGB
    for _ in 0..20 {
        let mid = (lo + hi) * 0.5;
        let rgb: Srgb<f32> = Oklch::new(l, mid, hue_val).into_color_unclamped();
        if in_gamut(&rgb) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Map an OKLCH color into sRGB gamut by reducing chroma until it fits.
///
/// Naive RGB clamping shifts hue and desaturates unevenly. This binary-searches
/// for the maximum chroma at the target lightness and hue that fits in sRGB,
/// preserving hue and maximizing saturation.
fn gamut_map(
    l: f32,
    target_chroma: f32,
    hue: impl Into<palette::OklabHue<f32>> + Copy,
) -> Srgb<f32> {
    let hue_val = hue.into();
    // Quick check: does the target chroma already fit?
    let rgb: Srgb<f32> = Oklch::new(l, target_chroma, hue_val).into_color_unclamped();
    if in_gamut(&rgb) {
        // `in_gamut` permits ±0.001 tolerance for fp slop, so the early-out
        // can return values like -0.0007 / 1.0007 — `iced::Color::from_rgb`
        // strictly validates `[0, 1]` and panics. Clamp before returning.
        return Srgb::new(
            rgb.red.clamp(0.0, 1.0),
            rgb.green.clamp(0.0, 1.0),
            rgb.blue.clamp(0.0, 1.0),
        );
    }

    // Binary search: find max chroma in [0, target_chroma] that fits sRGB.
    let mut lo = 0.0_f32;
    let mut hi = target_chroma;
    for _ in 0..16 {
        let mid = (lo + hi) * 0.5;
        let rgb: Srgb<f32> = Oklch::new(l, mid, hue_val).into_color_unclamped();
        if in_gamut(&rgb) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let rgb: Srgb<f32> = Oklch::new(l, lo, hue_val).into_color_unclamped();
    Srgb::new(
        rgb.red.clamp(0.0, 1.0),
        rgb.green.clamp(0.0, 1.0),
        rgb.blue.clamp(0.0, 1.0),
    )
}

fn in_gamut(rgb: &Srgb<f32>) -> bool {
    let e = 0.001; // small tolerance for floating-point
    rgb.red >= -e
        && rgb.red <= 1.0 + e
        && rgb.green >= -e
        && rgb.green <= 1.0 + e
        && rgb.blue >= -e
        && rgb.blue <= 1.0 + e
}

/// Lightness-based chroma taper used by the tonal palette generators.
///
/// Plateau ≈ 1.0 for L in roughly [0.12, 0.92]; outside that band chroma
/// smoothly fades to 0 at pure black / white. Without this, very dark
/// steps request full `base_chroma`, `gamut_map` clamps to the sRGB
/// ceiling at that lightness, and the result looks "pumped" — a flat,
/// maximally-saturated deep colour instead of fading toward neutral.
fn chroma_envelope_l(l: f32) -> f32 {
    if l <= 0.0 || l >= 1.0 {
        return 0.0;
    }
    let low_edge: f32 = 0.12;
    let high_edge: f32 = 0.08; // distance from L=1
    let low = smoothstep01(l / low_edge);
    let high = smoothstep01((1.0 - l) / high_edge);
    low.min(high)
}

fn smoothstep01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Teal correction weight: peaks at t=0.35, zero at extremes.
fn teal_correction_weight(t: f32) -> f32 {
    let center = 0.35;
    let width = 0.25;
    let d = ((t - center) / width).abs();
    (1.0 - d * d).max(0.0)
}

fn parse_hex(hex: &str) -> Srgb<f32> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() < 6 {
        return Srgb::new(0.5, 0.5, 0.5);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Srgb::new(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_endpoints() {
        let p = generate_tonal_palette("#EF443B", 1.0);
        // Step 0 should be black.
        assert!(p.steps[0].red < 0.01);
        // Step 100 should be white.
        assert!(p.steps[100].red > 0.99);
    }

    #[test]
    fn palette_monotonic_lightness() {
        let p = generate_tonal_palette("#0f3460", 1.0);
        // Each step should be brighter than the previous.
        for i in 1..=100 {
            let prev_lum = luminance(&p.steps[i - 1]);
            let curr_lum = luminance(&p.steps[i]);
            assert!(
                curr_lum >= prev_lum - 0.01,
                "step {i} is darker than step {}: {curr_lum} < {prev_lum}",
                i - 1
            );
        }
    }

    #[test]
    fn neutral_has_teal_tint() {
        let p = generate_neutral_palette("#8B959B", 1.0);
        // Mid-dark step should have a slight blue-green tint.
        let mid = p.steps[35];
        // Blue channel should be at least as strong as red (teal bias).
        assert!(
            mid.blue >= mid.red - 0.05,
            "step 35 should have teal tint: r={}, b={}",
            mid.red,
            mid.blue
        );
    }

    #[test]
    fn hex_output() {
        let p = generate_tonal_palette("#EF443B", 1.0);
        let h = p.hex(50);
        assert!(h.starts_with('#'));
        assert_eq!(h.len(), 7);
    }

    fn luminance(c: &Srgb<f32>) -> f32 {
        0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue
    }
}
