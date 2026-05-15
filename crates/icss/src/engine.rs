//! Generative design system — produces `.icss` themes from base variables.
//!
//! Takes ~12 inputs (4 chromatic colors + neutral + link, increment,
//! font-increment, radius-factor, light/dark mode) and deterministically
//! produces a complete `.icss` theme.
//!
//! Signal colors (success/danger/warning) are derived from the 4 chromatic
//! colors, constrained to traditional hue ranges with high saturation.

pub mod compose;
pub mod dims;
pub mod semantic;
pub mod signal;
pub mod tailwind;
pub mod tonal;

use dims::{DimInputs, DimTokens};
use semantic::{Mode, Palettes, SemanticColors};
use signal::derive_signals;
use tonal::{TonalPalette, closest_path_scale, generate_neutral_palette, generate_tonal_palette};

/// Number of swatches in the closest-path theme scale exposed as
/// `--scale-colors-0` … `--scale-colors-{N-1}` in the generated ICSS.
pub const SCALE_COLORS_COUNT: usize = 20;

/// All inputs needed to generate a complete theme.
///
/// Note: success/danger/warning are derived automatically from the
/// 4 chromatic colors. No need to specify them.
#[derive(Debug, Clone)]
pub struct ThemeInputs {
    // Chromatic colors (hex strings)
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
    pub quaternary: String,
    // Neutral + link
    pub neutral: String,
    pub link: String,

    // Dimensions
    pub increment: f32,
    pub font_increment: f32,
    pub radius_factor: f32,

    // Mode
    pub dark_mode: bool,

    /// Base surface lightness (tonal step 0–100).
    /// Controls how light/dark the main background is.
    /// `None` falls back to `semantic::default_surface_lightness` for the
    /// mode (95 light / 5 dark). Range: ~10–99.
    pub surface_lightness: Option<u8>,

    /// Gamma correction applied to the tonal lightness curve.
    /// 1.0 = linear. <1 spreads dark end. >1 spreads light end.
    /// Typical range: 0.3–3.0.
    pub gamma: Option<f32>,

    /// On-surface text step spacing multiplier.
    /// 1.0 = mirror surface spacing exactly. 2.0 = double the gap.
    /// 0.5 = half the gap. Range: 0.1–3.0.
    pub text_spread: Option<f32>,

    /// Manual override for the success signal color. `None` (or empty) →
    /// auto-derive from the 4 chromatic colors via `derive_signals`.
    pub success_override: Option<String>,
    /// Manual override for the danger signal color.
    pub danger_override: Option<String>,
    /// Manual override for the warning signal color.
    pub warning_override: Option<String>,
}

impl Default for ThemeInputs {
    fn default() -> Self {
        Self {
            primary: "#1101CB".into(),
            secondary: "#3DAAFA".into(),
            tertiary: "#C42451".into(),
            quaternary: "#064E56".into(),
            neutral: "#8B959B".into(),
            link: "#0D5A9E".into(),
            increment: 8.0,
            font_increment: 9.0,
            radius_factor: 1.6,
            dark_mode: true,
            surface_lightness: None,
            gamma: None,
            text_spread: None,
            success_override: None,
            danger_override: None,
            warning_override: None,
        }
    }
}

/// Result of theme generation — includes derived signal colors
/// and resolved dimensions so the UI can apply padding/font-size.
#[derive(Debug, Clone)]
pub struct ThemeOutput {
    pub icss: String,
    pub colors: SemanticColors,
    pub dims: DimTokens,
    pub derived_success: String,
    pub derived_danger: String,
    pub derived_warning: String,
    /// The actual surface lightness step used (for display).
    pub surface_lightness: u8,
    /// Neutral tonal palette as 101 `[r, g, b]` f32 triples (for visualization).
    pub neutral_palette: Vec<[f32; 3]>,
    /// Step indices for each surface family (for primitives display).
    pub family_steps: Vec<(&'static str, semantic::SurfaceSteps)>,
    /// Closest-path scale at step 50 — `SCALE_COLORS_COUNT` swatches as
    /// `#RRGGBB` strings. Also emitted as `--scale-colors-N` vars in the
    /// ICSS. Consumers (e.g. device-tile tints) should read this vec
    /// rather than re-computing the scale.
    pub scale_colors: Vec<String>,
}

/// Generate a complete `.icss` theme string from inputs.
pub fn generate(inputs: &ThemeInputs) -> ThemeOutput {
    // 0. Signal colors: derive from the 4 chromatic colors, then apply
    // any manual overrides. An override is a 7-char `#RRGGBB` string;
    // `None` or anything shorter falls back to the derived value.
    let derived = derive_signals(
        &inputs.primary,
        &inputs.secondary,
        &inputs.tertiary,
        &inputs.quaternary,
    );
    let pick_override = |o: &Option<String>, fallback: &str| -> String {
        match o.as_deref() {
            Some(s) if s.starts_with('#') && s.len() == 7 => s.to_string(),
            _ => fallback.to_string(),
        }
    };
    let signals = signal::SignalColors {
        success: pick_override(&inputs.success_override, &derived.success),
        danger: pick_override(&inputs.danger_override, &derived.danger),
        warning: pick_override(&inputs.warning_override, &derived.warning),
    };

    // 1. Generate tonal palettes.
    let gamma = inputs.gamma.unwrap_or(1.0);
    let text_spread = inputs.text_spread.unwrap_or(1.0);
    let palettes = Palettes {
        primary: generate_tonal_palette(&inputs.primary, gamma),
        secondary: generate_tonal_palette(&inputs.secondary, gamma),
        tertiary: generate_tonal_palette(&inputs.tertiary, gamma),
        quaternary: generate_tonal_palette(&inputs.quaternary, gamma),
        neutral: generate_neutral_palette(&inputs.neutral, gamma),
        link: generate_tonal_palette(&inputs.link, gamma),
        success: generate_tonal_palette(&signals.success, gamma),
        danger: generate_tonal_palette(&signals.danger, gamma),
        warning: generate_tonal_palette(&signals.warning, gamma),
        primary_base_step: TonalPalette::base_step(&inputs.primary, gamma),
        secondary_base_step: TonalPalette::base_step(&inputs.secondary, gamma),
        tertiary_base_step: TonalPalette::base_step(&inputs.tertiary, gamma),
        quaternary_base_step: TonalPalette::base_step(&inputs.quaternary, gamma),
        success_base_step: TonalPalette::base_step(&signals.success, gamma),
        danger_base_step: TonalPalette::base_step(&signals.danger, gamma),
        warning_base_step: TonalPalette::base_step(&signals.warning, gamma),
    };

    // 2. Resolve semantic colors.
    let mode = if inputs.dark_mode {
        Mode::Dark
    } else {
        Mode::Light
    };
    let colors = SemanticColors::resolve(&palettes, mode, inputs.surface_lightness, text_spread);

    // 3. Resolve dimensions.
    let dims = DimTokens::resolve(&DimInputs {
        increment: inputs.increment,
        font_increment: inputs.font_increment,
        radius_factor: inputs.radius_factor,
    });

    // 4. Compute the closest-path scale (20 swatches over P/S/T/Q) and
    // compose .icss output.
    let scale = closest_path_scale(
        &[
            &inputs.primary,
            &inputs.secondary,
            &inputs.tertiary,
            &inputs.quaternary,
        ],
        50,
        SCALE_COLORS_COUNT,
        gamma,
    );
    let scale_hex: Vec<String> = scale
        .iter()
        .map(|(c, _, _)| {
            let r = (c.red.clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (c.green.clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (c.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
            format!("#{r:02X}{g:02X}{b:02X}")
        })
        .collect();
    let icss = compose::compose_icss(&colors, &dims, &scale_hex);

    let actual_surface = inputs
        .surface_lightness
        .unwrap_or_else(|| semantic::default_surface_lightness(mode));

    let neutral_palette: Vec<[f32; 3]> = palettes
        .neutral
        .steps
        .iter()
        .map(|c| {
            [
                c.red.clamp(0.0, 1.0),
                c.green.clamp(0.0, 1.0),
                c.blue.clamp(0.0, 1.0),
            ]
        })
        .collect();

    let family_steps = colors.family_steps.clone();

    ThemeOutput {
        icss,
        colors,
        dims,
        derived_success: signals.success,
        derived_danger: signals.danger,
        derived_warning: signals.warning,
        surface_lightness: actual_surface,
        neutral_palette,
        family_steps,
        scale_colors: scale_hex,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_icss() {
        let output = generate(&ThemeInputs::default());
        assert!(output.icss.contains(":root {"));
        assert!(output.icss.contains(".button {"));
        assert!(output.icss.contains(".primary {"));
        assert!(output.icss.contains(".input {"));
        assert!(output.icss.contains(".checkbox {"));
    }

    #[test]
    fn light_and_dark_differ() {
        let mut inputs = ThemeInputs::default();
        inputs.dark_mode = true;
        let dark = generate(&inputs);
        inputs.dark_mode = false;
        let light = generate(&inputs);
        assert_ne!(dark.icss, light.icss);
    }

    #[test]
    fn different_increments_change_radii() {
        let mut inputs = ThemeInputs::default();
        inputs.increment = 8.0;
        let a = generate(&inputs);
        inputs.increment = 13.0;
        let b = generate(&inputs);
        assert_ne!(a.icss, b.icss);
    }

    #[test]
    fn derived_signals_are_valid_hex() {
        let output = generate(&ThemeInputs::default());
        assert!(output.derived_success.starts_with('#'));
        assert!(output.derived_danger.starts_with('#'));
        assert!(output.derived_warning.starts_with('#'));
        assert_eq!(output.derived_success.len(), 7);
    }
}
