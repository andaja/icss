//! Dimensional system — spacing, sizing, radius, typography scales.
//!
//! All values derive from a base `increment` through multiplication.

/// Dimensional inputs.
#[derive(Debug, Clone, Copy)]
pub struct DimInputs {
    pub increment: f32,
    pub font_increment: f32,
    pub radius_factor: f32, // radius_increment = increment * radius_factor
}

impl Default for DimInputs {
    fn default() -> Self {
        Self {
            increment: 8.0,
            font_increment: 8.0,
            radius_factor: 0.5,
        }
    }
}

/// Resolved dimensional tokens.
#[derive(Debug, Clone)]
pub struct DimTokens {
    // Spacing
    pub space_25: f32,
    pub space_50: f32,
    pub space_75: f32,
    pub space_100: f32,
    pub space_150: f32,
    pub space_200: f32,
    pub space_250: f32,
    pub space_300: f32,
    pub space_400: f32,
    pub space_500: f32,

    // Sizing
    pub size_100: f32,
    pub size_150: f32,
    pub size_200: f32,
    pub size_250: f32,
    pub size_300: f32,
    pub size_400: f32,
    pub size_500: f32,

    // Radius
    pub radius_0: f32,
    pub radius_25: f32,
    pub radius_50: f32,
    pub radius_75: f32,
    pub radius_100: f32,
    pub radius_150: f32,
    pub radius_200: f32,
    pub radius_infinite: f32,

    // Typography (font sizes)
    pub font_body_micro: f32,
    pub font_body_small: f32,
    pub font_body_medium_small: f32,
    pub font_body_medium: f32,
    pub font_body_large: f32,
    pub font_label_micro: f32,
    pub font_label_small: f32,
    pub font_label_medium: f32,
    pub font_label_large: f32,
    pub font_title_small: f32,
    pub font_title_medium: f32,
    pub font_title_large: f32,
    pub font_headline_micro: f32,
    pub font_headline_small: f32,
    pub font_headline_medium: f32,
    pub font_headline_large: f32,
    pub font_headline_xlarge: f32,
    pub font_headline_xxlarge: f32,
    pub font_headline_xxxlarge: f32,
}

impl DimTokens {
    pub fn resolve(inputs: &DimInputs) -> Self {
        let inc = inputs.increment;
        let finc = inputs.font_increment;
        let rinc = inc * inputs.radius_factor;

        Self {
            // Spacing: multiplier × increment (rounded to nearest int)
            space_25: (inc * 0.25).round(),
            space_50: (inc * 0.5).round(),
            space_75: (inc * 0.75).round(),
            space_100: inc,
            space_150: (inc * 1.5).round(),
            space_200: (inc * 2.0).round(),
            space_250: (inc * 2.5).round(),
            space_300: (inc * 3.0).round(),
            space_400: (inc * 4.0).round(),
            space_500: (inc * 5.0).round(),

            // Sizing
            size_100: inc,
            size_150: (inc * 1.5).round(),
            size_200: (inc * 2.0).round(),
            size_250: (inc * 2.5).round(),
            size_300: (inc * 3.0).round(),
            size_400: (inc * 4.0).round(),
            size_500: (inc * 5.0).round(),

            // Radius
            radius_0: (rinc * 0.25)
                .round()
                .max(if rinc > 0.0 { 1.0 } else { 0.0 }),
            radius_25: (rinc * 0.25)
                .round()
                .max(if rinc > 0.0 { 1.0 } else { 0.0 }),
            radius_50: (rinc * 0.5).round(),
            radius_75: (rinc * 0.75).round(),
            radius_100: rinc.round(),
            radius_150: (rinc * 1.5).round(),
            radius_200: (rinc * 2.0).round(),
            radius_infinite: 1000.0,

            // Typography: multiplier × font_increment
            // Body: 400 weight, reading text
            font_body_micro: (finc * 1.25).round(),
            font_body_small: (finc * 1.5).round(),
            font_body_medium_small: (finc * 1.625).round(),
            font_body_medium: (finc * 1.75).round(),
            font_body_large: (finc * 2.0).round(),
            // Label: 600 semibold, UI controls
            font_label_micro: (finc * 1.25).round(),
            font_label_small: (finc * 1.5).round(),
            font_label_medium: (finc * 1.75).round(),
            font_label_large: (finc * 2.0).round(),
            // Title: 600 semibold, section headings
            font_title_small: (finc * 2.25).round(),
            font_title_medium: (finc * 2.75).round(),
            font_title_large: (finc * 3.0).round(),
            // Headline: 300 light (small ≤ 30px), 400 regular (> 30px)
            font_headline_micro: (finc * 2.5).round(),
            font_headline_small: (finc * 3.0).round(),
            font_headline_medium: (finc * 3.5).round(),
            font_headline_large: (finc * 4.5).round(),
            font_headline_xlarge: (finc * 5.5).round(),
            font_headline_xxlarge: (finc * 7.0).round(),
            font_headline_xxxlarge: (finc * 9.0).round(),
        }
    }
}

/// Size presets for component composition.
#[derive(Debug, Clone, Copy)]
pub struct SizePreset {
    pub pad_v: f32,
    pub pad_h: f32,
    pub gap: f32,
    pub font_size: f32,
}

impl SizePreset {
    pub fn tiny(d: &DimTokens) -> Self {
        Self {
            pad_v: d.space_25,
            pad_h: d.space_75,
            gap: d.space_50,
            font_size: d.font_label_medium,
        }
    }

    pub fn micro(d: &DimTokens) -> Self {
        Self {
            pad_v: d.space_50,
            pad_h: d.space_100,
            gap: d.space_50,
            font_size: d.font_label_medium,
        }
    }

    pub fn small(d: &DimTokens) -> Self {
        Self {
            pad_v: d.space_75,
            pad_h: d.space_150,
            gap: d.space_100,
            font_size: d.font_label_large,
        }
    }

    pub fn medium(d: &DimTokens) -> Self {
        Self {
            pad_v: d.space_100,
            pad_h: d.space_200,
            gap: d.space_100,
            font_size: d.font_label_large,
        }
    }
}
