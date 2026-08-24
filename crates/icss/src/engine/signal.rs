//! Signal color derivation — success, danger, warning from theme colors.
//!
//! Signal colors are "softly wired" to the 4 chromatic theme colors:
//! - They match the nearest theme color's hue, clamped to safe ranges
//! - Saturation is kept high (min 50%) and slightly above the source
//! - Traditional color coding is preserved: green=success, red=danger, yellow=warning

use palette::convert::FromColorUnclamped;
use palette::{Oklch, Srgb};

/// Hue ranges for signal colors (in OKLCH degrees).
/// OKLCH hue wheel: red ~25-30, orange ~55-70, yellow ~85-100,
/// green ~145-155, teal ~180-200, blue ~260, purple ~310.
const SUCCESS_HUE_RANGE: (f32, f32) = (140.0, 170.0); // green
const DANGER_HUE_RANGE: (f32, f32) = (15.0, 40.0); // red
const WARNING_HUE_RANGE: (f32, f32) = (60.0, 100.0); // yellow/amber (avoid orange)

/// Minimum chroma for signal colors (ensures visibility).
const MIN_SIGNAL_CHROMA: f32 = 0.14;

/// Chroma boost over the source theme color.
#[allow(dead_code)]
const CHROMA_BOOST: f32 = 0.02;

/// Derived signal colors as hex strings.
#[derive(Debug, Clone)]
pub struct SignalColors {
    pub success: String,
    pub danger: String,
    pub warning: String,
}

/// Derive signal colors from the 4 chromatic theme colors.
///
/// For each signal, finds the theme color with hue closest to the
/// signal's acceptable range, then uses that hue (clamped) with
/// boosted saturation.
pub fn derive_signals(
    primary: &str,
    secondary: &str,
    tertiary: &str,
    quaternary: &str,
) -> SignalColors {
    let theme_colors = [
        to_oklch(primary),
        to_oklch(secondary),
        to_oklch(tertiary),
        to_oklch(quaternary),
    ];

    SignalColors {
        success: derive_one(&theme_colors, SUCCESS_HUE_RANGE, 155.0),
        danger: derive_one(&theme_colors, DANGER_HUE_RANGE, 25.0),
        warning: derive_one(&theme_colors, WARNING_HUE_RANGE, 80.0),
    }
}

fn derive_one(theme_colors: &[Oklch; 4], hue_range: (f32, f32), default_hue: f32) -> String {
    // Find the theme color whose hue is closest to the signal range.
    let mut best_idx = 0;
    let mut best_dist = f32::MAX;

    for (i, color) in theme_colors.iter().enumerate() {
        let h: f32 = color.hue.into_inner();
        let dist = hue_distance_to_range(h, hue_range);
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }

    let source = &theme_colors[best_idx];
    let source_hue: f32 = source.hue.into_inner();

    // Use source hue if it's within range, otherwise clamp to nearest edge.
    let target_hue = if best_dist < 0.1 {
        // Already in range.
        source_hue
    } else if best_dist < 40.0 {
        // Close enough — clamp to range boundary.
        clamp_hue_to_range(source_hue, hue_range)
    } else {
        // Too far — use default hue.
        default_hue
    };

    // Lightness: ~0.65 on the OKLCH scale (good for both light and dark modes).
    let target_l = 0.65;

    // Signal colors should always be vivid. Use the maximum in-gamut chroma
    // at this lightness/hue so signals pop regardless of theme saturation.
    let max_chroma = crate::engine::tonal::max_srgb_chroma(target_l, target_hue);
    // Use 90% of max gamut to stay comfortably inside, but never below MIN_SIGNAL_CHROMA.
    let target_chroma = (max_chroma * 0.9).max(MIN_SIGNAL_CHROMA);

    let rgb = crate::engine::tonal::gamut_map_pub(target_l, target_chroma, target_hue);
    let r = (rgb.red.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (rgb.green.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (rgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// Distance from a hue to the nearest point in a hue range.
/// Returns 0 if hue is within the range.
fn hue_distance_to_range(hue: f32, range: (f32, f32)) -> f32 {
    let (lo, hi) = range;

    if lo <= hi {
        // Normal range (no wrap).
        if hue >= lo && hue <= hi {
            0.0
        } else {
            let d_lo = angular_distance(hue, lo);
            let d_hi = angular_distance(hue, hi);
            d_lo.min(d_hi)
        }
    } else {
        // Wrapping range (e.g., 350..15).
        if hue >= lo || hue <= hi {
            0.0
        } else {
            let d_lo = angular_distance(hue, lo);
            let d_hi = angular_distance(hue, hi);
            d_lo.min(d_hi)
        }
    }
}

/// Clamp a hue to the nearest edge of a range.
fn clamp_hue_to_range(hue: f32, range: (f32, f32)) -> f32 {
    let (lo, hi) = range;
    let d_lo = angular_distance(hue, lo);
    let d_hi = angular_distance(hue, hi);
    if d_lo <= d_hi { lo } else { hi }
}

/// Shortest angular distance between two hues (0–360).
fn angular_distance(a: f32, b: f32) -> f32 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

fn to_oklch(hex: &str) -> Oklch {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let (r, g, b) = if hex.len() >= 6 {
        (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0,
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0,
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0,
        )
    } else {
        (0.5, 0.5, 0.5)
    };
    Oklch::from_color_unclamped(Srgb::new(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_signals() {
        let signals = derive_signals("#EF443B", "#FA863E", "#C42451", "#29C3D4");
        // Success should be greenish (OKLCH hue 140–170).
        let s = to_oklch(&signals.success);
        let h: f32 = s.hue.into_inner();
        assert!(
            (135.0..=175.0).contains(&h),
            "success hue {h} not in green range"
        );

        // Danger should be reddish (OKLCH hue 15–40).
        let d = to_oklch(&signals.danger);
        let h: f32 = d.hue.into_inner();
        assert!(h <= 45.0 || h >= 345.0, "danger hue {h} not in red range");

        // Warning should be yellow/amber (OKLCH hue 55–100).
        let w = to_oklch(&signals.warning);
        let h: f32 = w.hue.into_inner();
        // Allow some drift from gamut clamping roundtrip.
        assert!(
            (45.0..=110.0).contains(&h),
            "warning hue {h} not in yellow range"
        );
    }

    #[test]
    fn teal_purple_theme_signals() {
        let signals = derive_signals("#7B2D8E", "#29C3D4", "#8844AA", "#22AA88");
        let s = to_oklch(&signals.success);
        let h: f32 = s.hue.into_inner();
        assert!(
            (135.0..=175.0).contains(&h),
            "success hue {h} should stay green"
        );
    }

    #[test]
    fn signal_chroma_is_high() {
        let signals = derive_signals("#888888", "#999999", "#AAAAAA", "#777777");
        // Even with grey theme colors, signals should have visible chroma.
        let s = to_oklch(&signals.success);
        assert!(s.chroma >= 0.10, "success chroma {} too low", s.chroma);
    }
}
