//! Semantic color mapping — maps tonal palette steps to UI roles.
//!
//! Each semantic token picks a specific step from a tonal palette.
//! The step changes between light and dark mode (usually mirrored: dark = 100 - light).

use crate::engine::tonal::TonalPalette;

/// Which mode we're generating for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Light,
    Dark,
}

/// Fallback surface base step (tonal step 0–100) when the caller does not
/// specify `surface_lightness`.
///
/// The surface step is user-selectable; these are only the values used when
/// no choice is given. Light and dark are independent — deliberately *not*
/// mirrored. This is the single source of truth: `engine::generate` reads it
/// for `actual_surface`, and `SemanticColors::resolve` reads it for the
/// actual color resolution, so the two can never disagree.
pub fn default_surface_lightness(mode: Mode) -> u8 {
    match mode {
        Mode::Dark => 5,
        Mode::Light => 95,
    }
}

/// All tonal palettes needed by the theme.
pub struct Palettes {
    pub primary: TonalPalette,
    pub secondary: TonalPalette,
    pub tertiary: TonalPalette,
    pub quaternary: TonalPalette,
    pub neutral: TonalPalette,
    pub link: TonalPalette,
    pub success: TonalPalette,
    pub danger: TonalPalette,
    pub warning: TonalPalette,

    /// The tonal step of the user's chosen primary color.
    /// Used to place buttons at the user's intended lightness.
    pub primary_base_step: usize,
    pub secondary_base_step: usize,
    pub tertiary_base_step: usize,
    pub quaternary_base_step: usize,
    pub success_base_step: usize,
    pub danger_base_step: usize,
    pub warning_base_step: usize,
}

/// Offsets the four solid outlines sit at, in `step_size` units walked away
/// from a family's surface base toward its text.
const OUTLINE_OFFSETS: [i32; 4] = [7, 11, 15, 20];

/// Outlines for one surface family (6 levels).
///
/// Two alpha levels taken at the family's own text end, which is what lets
/// them survive any background, then four solid ones walked from the family's
/// base step toward that text.
#[derive(Debug, Clone)]
pub struct Outlines {
    pub subtle: String,
    pub soft: String,
    pub middle: String,
    pub strong: String,
    pub heavy: String,
    pub solid: String,
}

impl Outlines {
    /// Resolve against `palette`, walking away from `base` toward the text.
    ///
    /// Direction comes from the family's own text ladder rather than from
    /// re-measuring the surface. A surface sitting mid-ramp can still be given
    /// light text, and an outline that walked the other way would contradict
    /// the text resolved to sit on top of it.
    fn resolve(palette: &TonalPalette, base: usize, needs_light_text: bool) -> Self {
        let dir: i32 = if needs_light_text { 1 } else { -1 };
        let text_end: usize = if needs_light_text { 90 } else { 10 };
        // The offsets were calibrated for the neutral page at step 5 or 95,
        // where there are 95 steps of room between the base and the ramp end
        // in the text direction. A chromatic base sits mid-ramp and has less,
        // so the walk scales down with the room it actually has. At the
        // default page step the scale is 1 and nothing moves.
        let room = if dir > 0 { 100 - base } else { base } as f32;
        let scale = room / 95.0;
        let step = |offset: i32| -> usize {
            let walk = (offset as f32 * 3.0 * scale).round() as i32;
            (base as i32 + walk * dir).clamp(0, 100) as usize
        };
        Self {
            subtle: palette.rgba(text_end, 0.08),
            soft: palette.rgba(text_end, 0.15),
            middle: palette.hex(step(OUTLINE_OFFSETS[0])),
            strong: palette.hex(step(OUTLINE_OFFSETS[1])),
            heavy: palette.hex(step(OUTLINE_OFFSETS[2])),
            solid: palette.hex(step(OUTLINE_OFFSETS[3])),
        }
    }
}

/// Complete color scale for one surface type.
#[derive(Debug, Clone)]
pub struct SurfaceFamily {
    // Surface steps (elevation ladder)
    pub s0: String,
    pub s1: String,
    pub s2: String,
    pub s3: String,
    pub s4: String,
    // Text on this surface (7 levels: 6 solid from tonal palette + 1 alpha-based)
    pub text: String,
    pub text_default: String,
    pub text_soft: String,
    pub text_dim: String,
    pub text_muted: String,
    pub text_faint: String,
    pub text_disabled: String,
    // Outlines cut from this family's own ladder.
    pub outlines: Outlines,
}

/// Step indices used to build a surface family (for display/debugging).
#[derive(Debug, Clone)]
pub struct SurfaceSteps {
    pub surface: [usize; 5], // s0..s4
    pub text: [usize; 6],    // text, default, soft, dim, muted, faint
}

impl SurfaceFamily {
    /// Re-cut this family's outlines.
    ///
    /// Called after a text override, because the outline direction follows the
    /// text ladder that ends up on the family rather than the one `resolve`
    /// first computed.
    fn with_outlines(mut self, palette: &TonalPalette, base: usize, needs_light_text: bool) -> Self {
        self.outlines = Outlines::resolve(palette, base, needs_light_text);
        self
    }

    /// Copy text hex colors from another family (keeps own surface steps).
    /// Used when two families share the same palette (e.g. tint borrows from surface).
    fn with_text_from(mut self, source: &SurfaceFamily) -> Self {
        self.text = source.text.clone();
        self.text_default = source.text_default.clone();
        self.text_soft = source.text_soft.clone();
        self.text_dim = source.text_dim.clone();
        self.text_muted = source.text_muted.clone();
        self.text_faint = source.text_faint.clone();
        self.text_disabled = source.text_disabled.clone();
        self
    }

    /// Re-resolve text colors using step indices from `ref_family` but colors
    /// from `palette`. This keeps the chromatic hue while using the reference
    /// family's lightness distribution.
    fn with_text_steps_from(
        mut self,
        palette: &TonalPalette,
        ref_base: usize,
        step_size: i32,
        ref_dir: i32,
        ref_needs_light: bool,
        text_spread: f32,
    ) -> Self {
        let _ref_step = |offset: i32| -> usize {
            (ref_base as i32 + offset * step_size * ref_dir).clamp(0, 100) as usize
        };
        // Text levels spread out from the extreme end (100 for light text, 0
        // for dark). text_spread scales the gap between each adjacent level.
        let extreme: i32 = if ref_needs_light { 100 } else { 0 };
        let dir: i32 = if ref_needs_light { -1 } else { 1 };
        let text_step = |offset: i32| -> usize {
            let gap = (offset as f32 * step_size as f32 * text_spread).round() as i32;
            (extreme + dir * gap).clamp(0, 100) as usize
        };
        self.text = palette.hex(extreme as usize);
        self.text_default = palette.hex(text_step(1));
        self.text_soft = palette.hex(text_step(2));
        self.text_dim = palette.hex(text_step(3));
        self.text_muted = palette.hex(text_step(4));
        self.text_faint = palette.hex(text_step(5));
        self.text_disabled = palette.rgba(text_step(1), 0.50);
        self
    }

    fn steps(
        base: usize,
        step_size: i32,
        step_dir: i32,
        needs_light_text: bool,
        text_spread: f32,
    ) -> SurfaceSteps {
        let step = |offset: i32| -> usize {
            (base as i32 + offset * step_size * step_dir).clamp(0, 100) as usize
        };
        // Text levels spread out from the extreme end (100 for light text,
        // 0 for dark). text_spread scales the gap between each adjacent level.
        let extreme: i32 = if needs_light_text { 100 } else { 0 };
        let dir: i32 = if needs_light_text { -1 } else { 1 };
        let text_step = |offset: i32| -> usize {
            let gap = (offset as f32 * step_size as f32 * text_spread).round() as i32;
            (extreme + dir * gap).clamp(0, 100) as usize
        };
        let t_text = extreme as usize;
        let (t_default, t_soft, t_dim, t_muted, t_faint) = (
            text_step(1),
            text_step(2),
            text_step(3),
            text_step(4),
            text_step(5),
        );
        SurfaceSteps {
            surface: [step(0), step(1), step(2), step(3), step(4)],
            text: [t_text, t_default, t_soft, t_dim, t_muted, t_faint],
        }
    }

    fn resolve(
        palette: &TonalPalette,
        base: usize,
        step_size: i32,
        step_dir: i32,
        needs_light_text: bool,
        text_spread: f32,
    ) -> Self {
        let step = |offset: i32| -> usize {
            (base as i32 + offset * step_size * step_dir).clamp(0, 100) as usize
        };
        // Text levels spread out from the extreme end (100 for light text,
        // 0 for dark). text_spread scales the gap between each adjacent level.
        let extreme: i32 = if needs_light_text { 100 } else { 0 };
        let dir: i32 = if needs_light_text { -1 } else { 1 };
        let text_step = |offset: i32| -> usize {
            let gap = (offset as f32 * step_size as f32 * text_spread).round() as i32;
            (extreme + dir * gap).clamp(0, 100) as usize
        };
        let t_text = extreme as usize;
        let t_default = text_step(1);
        let t_soft = text_step(2);
        let t_dim = text_step(3);
        let t_muted = text_step(4);
        let t_faint = text_step(5);
        Self {
            s0: palette.hex(step(0)),
            s1: palette.hex(step(1)),
            s2: palette.hex(step(2)),
            s3: palette.hex(step(3)),
            s4: palette.hex(step(4)),
            text: palette.hex(t_text),
            text_default: palette.hex(t_default),
            text_soft: palette.hex(t_soft),
            text_dim: palette.hex(t_dim),
            text_muted: palette.hex(t_muted),
            text_faint: palette.hex(t_faint),
            text_disabled: palette.rgba(t_default, 0.50),
            outlines: Outlines::resolve(palette, base, needs_light_text),
        }
    }
}

/// Resolved semantic color tokens — all as hex strings.
#[derive(Debug, Clone)]
pub struct SemanticColors {
    // Neutral surfaces
    pub surface: SurfaceFamily, // Main neutral surface (s0-s4 + text levels)
    pub surface_s5: String,     // Neutral has 6 steps, not 5

    // Neutral outlines (6 levels)
    pub outline_subtle: String,
    pub outline_soft: String,
    pub outline_middle: String,
    pub outline_strong: String,
    pub outline_heavy: String,
    pub outline_solid: String,
    pub outline_subtle_alpha: String,
    pub outline_soft_alpha: String,

    // Variant surfaces
    pub tint: SurfaceFamily,      // Neutral mid-tone
    pub dark_tint: SurfaceFamily, // Neutral darker mid-tone
    pub black: SurfaceFamily,     // Near-black

    // Chromatic surfaces
    pub primary: SurfaceFamily,
    pub primary_container: SurfaceFamily, // Primary tint (pastel)
    pub secondary: SurfaceFamily,
    pub secondary_container: SurfaceFamily,
    pub tertiary: SurfaceFamily,
    pub tertiary_container: SurfaceFamily,
    pub quaternary: SurfaceFamily,
    pub quaternary_container: SurfaceFamily,

    // Signal surfaces
    pub success: SurfaceFamily,
    pub success_container: SurfaceFamily, // Success tint
    pub danger: SurfaceFamily,
    pub danger_container: SurfaceFamily,
    pub warning: SurfaceFamily,
    pub warning_container: SurfaceFamily,

    // Accent colors on neutral surfaces
    pub on_surface_primary: String,
    pub on_surface_secondary: String,
    pub on_surface_tertiary: String,
    pub on_surface_quaternary: String,
    pub on_surface_link: String,
    pub on_surface_success: String,
    pub on_surface_danger: String,
    pub on_surface_warning: String,

    // Shadows
    pub shadow_color: String,
    pub shadow_color_medium: String,
    pub shadow_color_soft: String,

    // Step indices for each family (for display in showcase primitives)
    pub family_steps: Vec<(&'static str, SurfaceSteps)>,
}

impl SemanticColors {
    pub fn resolve(
        palettes: &Palettes,
        mode: Mode,
        surface_lightness: Option<u8>,
        text_spread: f32,
    ) -> Self {
        let n = &palettes.neutral;
        let p = &palettes.primary;
        let s = &palettes.secondary;
        let te = &palettes.tertiary;
        let q = &palettes.quaternary;
        let su = &palettes.success;
        let da = &palettes.danger;
        let wa = &palettes.warning;
        let l = &palettes.link;

        // Surface base step — user-selected, or the mode's fallback default.
        let srf = surface_lightness
            .map(usize::from)
            .unwrap_or_else(|| default_surface_lightness(mode) as usize);

        // Base steps for colored surfaces.
        //
        // Lower cap: enough contrast against the page background (srf).
        // At least srf+20 so buttons are distinguishable from the background.
        //
        // The four chromatic families have no upper cap. They used to be
        // clamped to 50 so light text always worked, which made `needs_light`
        // below a foregone conclusion and landed a bright input darker than it
        // was picked, identically in both modes. The text direction is
        // resolved per family instead, so the surface can be the colour the
        // reader actually chose.
        //
        // Signals keep the cap. Success, danger and warning are inverted by
        // design and carry light text whatever the input, so they stay in the
        // dark half of the scale.
        let signal_max_step = 50;
        let min_dark_step = if mode == Mode::Dark {
            (srf + 20).min(40)
        } else {
            0
        };
        let chromatic_step = |step: usize| step.clamp(min_dark_step, 100);
        let signal_step = |step: usize| step.clamp(min_dark_step, signal_max_step);
        let pri = chromatic_step(palettes.primary_base_step);
        let sec = chromatic_step(palettes.secondary_base_step);
        let ter = chromatic_step(palettes.tertiary_base_step);
        let qua = chromatic_step(palettes.quaternary_base_step);
        // Signal colors shifted darker so text is always light (inverted).
        let sig_darken: usize = 10;
        let suc = signal_step(palettes.success_base_step.saturating_sub(sig_darken));
        let dan = signal_step(palettes.danger_base_step.saturating_sub(sig_darken));
        let war = signal_step(palettes.warning_base_step.saturating_sub(sig_darken));

        // Text contrast direction: step ≤ 55 → needs light text, step > 55 → dark text.
        // Based on the clamped tonal step, not the gamma-sensitive OKLCH
        // lightness, so text direction stays stable when gamma changes.
        let needs_light = |step: usize| step <= 55;
        let pri_light = needs_light(pri);
        let sec_light = needs_light(sec);
        let ter_light = needs_light(ter);
        let qua_light = needs_light(qua);
        // Signal colors always use light text (inverted) regardless of step.
        let suc_light = true;
        let dan_light = true;
        let war_light = true;
        // Elevation direction: darker modes go up, lighter modes go down.
        let is_dark_surface = srf < 50;
        let step_dir: i32 = if is_dark_surface { 1 } else { -1 };
        // Uniform step offsets — the tonal palette's ease_lightness curve
        // already provides wider perceptual gaps at the dark end.
        let step_size: i32 = 3;
        let srf_step = |offset: i32| -> usize {
            (srf as i32 + offset * step_size * step_dir).clamp(0, 100) as usize
        };
        // Outlines: away from surface toward the text.
        let out_step =
            |offset: i32| -> usize { (srf as i32 + offset * 3 * step_dir).clamp(0, 100) as usize };

        // Colored surface step direction: always darker (-1). Elevation on a
        // chromatic family reads as the colour deepening, in both modes, which
        // is what keeps a primary button's hover state recognisably primary.
        let color_step_dir: i32 = -1;

        // Container base steps (tint versions of colored surfaces)
        let container_base_dark: usize = 25;
        let container_base_light: usize = 80;

        // Neutral surface uses the same resolve() pipeline as all chromatic families.
        let surface =
            SurfaceFamily::resolve(n, srf, step_size, step_dir, is_dark_surface, text_spread);

        let surface_s5 = n.hex(srf_step(5));

        // Tint surface families
        let tint_base: usize = if is_dark_surface { 25 } else { 80 };
        let dark_tint_base: usize = if is_dark_surface { 15 } else { 50 };
        let black_base: usize = if is_dark_surface { 0 } else { 3 };

        let tint = if is_dark_surface {
            SurfaceFamily::resolve(n, tint_base, step_size, 1, true, text_spread)
        } else {
            SurfaceFamily::resolve(n, tint_base, step_size, -1, false, text_spread)
        };

        let dark_tint = if is_dark_surface {
            SurfaceFamily::resolve(n, dark_tint_base, step_size, 1, true, text_spread)
        } else {
            SurfaceFamily::resolve(n, dark_tint_base, step_size, -1, true, text_spread)
        };

        // Black surface (stays dark in both modes)
        let black = SurfaceFamily::resolve(n, black_base, step_size, 1, true, text_spread);

        // Primary family
        let primary =
            SurfaceFamily::resolve(p, pri, step_size, color_step_dir, pri_light, text_spread);

        // Primary container
        let pri_cont_base = if is_dark_surface {
            container_base_dark
        } else {
            container_base_light
        };
        let pri_cont_light = p.oklch_lightness(pri_cont_base) < 0.65;
        let primary_container = SurfaceFamily::resolve(
            p,
            pri_cont_base,
            step_size,
            if is_dark_surface { 1 } else { -1 },
            pri_cont_light,
            text_spread,
        );

        // Secondary family
        let secondary =
            SurfaceFamily::resolve(s, sec, step_size, color_step_dir, sec_light, text_spread);
        let sec_cont_base = if is_dark_surface {
            container_base_dark
        } else {
            container_base_light
        };
        let sec_cont_light = s.oklch_lightness(sec_cont_base) < 0.65;
        let secondary_container = SurfaceFamily::resolve(
            s,
            sec_cont_base,
            step_size,
            if is_dark_surface { 1 } else { -1 },
            sec_cont_light,
            text_spread,
        );

        // Tertiary family
        let tertiary =
            SurfaceFamily::resolve(te, ter, step_size, color_step_dir, ter_light, text_spread);
        let ter_cont_base = if is_dark_surface {
            container_base_dark
        } else {
            container_base_light
        };
        let ter_cont_light = te.oklch_lightness(ter_cont_base) < 0.65;
        let tertiary_container = SurfaceFamily::resolve(
            te,
            ter_cont_base,
            step_size,
            if is_dark_surface { 1 } else { -1 },
            ter_cont_light,
            text_spread,
        );

        // Quaternary family
        let quaternary =
            SurfaceFamily::resolve(q, qua, step_size, color_step_dir, qua_light, text_spread);
        let qua_cont_base = if is_dark_surface {
            container_base_dark
        } else {
            container_base_light
        };
        let qua_cont_light = q.oklch_lightness(qua_cont_base) < 0.65;
        let quaternary_container = SurfaceFamily::resolve(
            q,
            qua_cont_base,
            step_size,
            if is_dark_surface { 1 } else { -1 },
            qua_cont_light,
            text_spread,
        );

        // Success family
        let success =
            SurfaceFamily::resolve(su, suc, step_size, color_step_dir, suc_light, text_spread);
        let suc_cont_base = if is_dark_surface {
            container_base_dark
        } else {
            container_base_light
        };
        let suc_cont_light = su.oklch_lightness(suc_cont_base) < 0.65;
        let success_container = SurfaceFamily::resolve(
            su,
            suc_cont_base,
            step_size,
            if is_dark_surface { 1 } else { -1 },
            suc_cont_light,
            text_spread,
        );

        // Danger family
        let danger =
            SurfaceFamily::resolve(da, dan, step_size, color_step_dir, dan_light, text_spread);
        let dan_cont_base = if is_dark_surface {
            container_base_dark
        } else {
            container_base_light
        };
        let dan_cont_light = da.oklch_lightness(dan_cont_base) < 0.65;
        let danger_container = SurfaceFamily::resolve(
            da,
            dan_cont_base,
            step_size,
            if is_dark_surface { 1 } else { -1 },
            dan_cont_light,
            text_spread,
        );

        // Warning family
        let warning =
            SurfaceFamily::resolve(wa, war, step_size, color_step_dir, war_light, text_spread);
        let war_cont_base = if is_dark_surface {
            container_base_dark
        } else {
            container_base_light
        };
        let war_cont_light = wa.oklch_lightness(war_cont_base) < 0.65;
        let warning_container = SurfaceFamily::resolve(
            wa,
            war_cont_base,
            step_size,
            if is_dark_surface { 1 } else { -1 },
            war_cont_light,
            text_spread,
        );

        // ── On-surface text overrides ──
        // Create reference text colors for cross-theme use.
        // "dark_on_surface" = light text (as if dark theme), always needs_light_text=true
        // "light_on_surface" = dark text (as if light theme), always needs_light_text=false
        let dark_ref = SurfaceFamily::resolve(n, 5, step_size, 1, true, text_spread);

        // Each family's outlines are re-cut after its text override, because
        // the outline direction follows the text ladder the family ends up
        // with rather than the one `resolve` first computed.

        // 1. on-surface-tint → copy on-surface (same as neutral)
        let tint = tint
            .with_text_from(&surface)
            .with_outlines(n, tint_base, is_dark_surface);

        // 2. on-surface-dark-tint → dark theme copies on-surface (light text),
        //    light theme uses dark-ref (dark text on dark-tint surface)
        let dark_tint = if is_dark_surface {
            dark_tint.with_text_from(&surface)
        } else {
            dark_tint.with_text_from(&dark_ref)
        }
        .with_outlines(n, dark_tint_base, true);

        // 3. on-surface-black → always dark theme on-surface (light text)
        let black = black
            .with_text_from(&dark_ref)
            .with_outlines(n, black_base, true);

        // 4. on-surface-primary (and chromatic) → colors from the chromatic
        //    palette for hue/saturation, spread from whichever end that
        //    family's own surface step calls for. The direction is the same in
        //    both themes because the surface step it is taken from is.
        //    Signals are inverted by design and stay on light text.
        let dark_base: usize = 5; // dark theme neutral surface base
        let primary = primary
            .with_text_steps_from(p, dark_base, step_size, 1, pri_light, text_spread)
            .with_outlines(p, pri, pri_light);
        let secondary = secondary
            .with_text_steps_from(s, dark_base, step_size, 1, sec_light, text_spread)
            .with_outlines(s, sec, sec_light);
        let tertiary = tertiary
            .with_text_steps_from(te, dark_base, step_size, 1, ter_light, text_spread)
            .with_outlines(te, ter, ter_light);
        let quaternary = quaternary
            .with_text_steps_from(q, dark_base, step_size, 1, qua_light, text_spread)
            .with_outlines(q, qua, qua_light);
        let success = success
            .with_text_steps_from(su, dark_base, step_size, 1, suc_light, text_spread)
            .with_outlines(su, suc, suc_light);
        let danger = danger
            .with_text_steps_from(da, dark_base, step_size, 1, dan_light, text_spread)
            .with_outlines(da, dan, dan_light);
        let warning = warning
            .with_text_steps_from(wa, dark_base, step_size, 1, war_light, text_spread)
            .with_outlines(wa, war, war_light);

        // 5. on-surface-*-container → current theme's neutral on-surface step
        //    indices (srf), colors from chromatic palette.
        let primary_container = primary_container
            .with_text_steps_from(p, srf, step_size, step_dir, is_dark_surface, text_spread)
            .with_outlines(p, pri_cont_base, is_dark_surface);
        let secondary_container = secondary_container
            .with_text_steps_from(s, srf, step_size, step_dir, is_dark_surface, text_spread)
            .with_outlines(s, sec_cont_base, is_dark_surface);
        let tertiary_container = tertiary_container
            .with_text_steps_from(te, srf, step_size, step_dir, is_dark_surface, text_spread)
            .with_outlines(te, ter_cont_base, is_dark_surface);
        let quaternary_container = quaternary_container
            .with_text_steps_from(q, srf, step_size, step_dir, is_dark_surface, text_spread)
            .with_outlines(q, qua_cont_base, is_dark_surface);
        let success_container = success_container
            .with_text_steps_from(su, srf, step_size, step_dir, is_dark_surface, text_spread)
            .with_outlines(su, suc_cont_base, is_dark_surface);
        let danger_container = danger_container
            .with_text_steps_from(da, srf, step_size, step_dir, is_dark_surface, text_spread)
            .with_outlines(da, dan_cont_base, is_dark_surface);
        let warning_container = warning_container
            .with_text_steps_from(wa, srf, step_size, step_dir, is_dark_surface, text_spread)
            .with_outlines(wa, war_cont_base, is_dark_surface);

        // Accent on neutral — slightly adjusted for visibility
        let accent_offset: usize = 2;
        let on_surface_primary = if is_dark_surface {
            p.hex((pri + accent_offset).min(100))
        } else {
            p.hex(pri.saturating_sub(accent_offset))
        };
        let on_surface_secondary = if is_dark_surface {
            s.hex((sec + 5).min(100))
        } else {
            s.hex(sec.saturating_sub(5))
        };
        let on_surface_tertiary = if is_dark_surface {
            te.hex((ter + accent_offset).min(100))
        } else {
            te.hex(ter.saturating_sub(accent_offset))
        };
        let on_surface_quaternary = if is_dark_surface {
            q.hex((qua + accent_offset).min(100))
        } else {
            q.hex(qua.saturating_sub(accent_offset))
        };
        let on_surface_link = if is_dark_surface {
            l.hex(52)
        } else {
            l.hex(48)
        };
        let on_surface_success = if is_dark_surface {
            su.hex((suc + accent_offset).min(100))
        } else {
            su.hex(suc.saturating_sub(accent_offset))
        };
        let on_surface_danger = if is_dark_surface {
            da.hex((dan + accent_offset).min(100))
        } else {
            da.hex(dan.saturating_sub(accent_offset))
        };
        let on_surface_warning = if is_dark_surface {
            wa.hex((war + accent_offset).min(100))
        } else {
            wa.hex(war.saturating_sub(accent_offset))
        };

        // Shadows
        let (shadow_color, shadow_color_medium, shadow_color_soft) = if is_dark_surface {
            ("#000000".into(), "#000000".into(), "#000000".into())
        } else {
            (n.rgba(5, 0.15), n.rgba(2, 0.25), n.rgba(5, 0.08))
        };

        // The neutral page outlines are the neutral family's own set. These
        // aliases stay because components reference them directly.
        let no = surface.outlines.clone();

        Self {
            surface,
            surface_s5,

            outline_subtle: n.hex(out_step(1)),
            outline_soft: n.hex(out_step(3)),
            outline_middle: no.middle,
            outline_strong: no.strong,
            outline_heavy: no.heavy,
            outline_solid: no.solid,
            outline_subtle_alpha: no.subtle,
            outline_soft_alpha: no.soft,

            tint,
            dark_tint,
            black,

            primary,
            primary_container,
            secondary,
            secondary_container,
            tertiary,
            tertiary_container,
            quaternary,
            quaternary_container,

            success,
            success_container,
            danger,
            danger_container,
            warning,
            warning_container,

            on_surface_primary,
            on_surface_secondary,
            on_surface_tertiary,
            on_surface_quaternary,
            on_surface_link,
            on_surface_success,
            on_surface_danger,
            on_surface_warning,

            shadow_color,
            shadow_color_medium,
            shadow_color_soft,

            family_steps: vec![
                (
                    "surface",
                    SurfaceFamily::steps(srf, step_size, step_dir, is_dark_surface, text_spread),
                ),
                (
                    "surface-primary",
                    SurfaceFamily::steps(pri, step_size, color_step_dir, pri_light, text_spread),
                ),
                (
                    "surface-secondary",
                    SurfaceFamily::steps(sec, step_size, color_step_dir, sec_light, text_spread),
                ),
                (
                    "surface-tertiary",
                    SurfaceFamily::steps(ter, step_size, color_step_dir, ter_light, text_spread),
                ),
                (
                    "surface-quaternary",
                    SurfaceFamily::steps(qua, step_size, color_step_dir, qua_light, text_spread),
                ),
                (
                    "surface-success",
                    SurfaceFamily::steps(suc, step_size, color_step_dir, suc_light, text_spread),
                ),
                (
                    "surface-danger",
                    SurfaceFamily::steps(dan, step_size, color_step_dir, dan_light, text_spread),
                ),
                (
                    "surface-warning",
                    SurfaceFamily::steps(war, step_size, color_step_dir, war_light, text_spread),
                ),
                // Neutral variants and containers, so every family shown in the
                // showcase carries its step numbers rather than a blank row.
                (
                    "surface-tint",
                    SurfaceFamily::steps(
                        tint_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        is_dark_surface,
                        text_spread,
                    ),
                ),
                (
                    "surface-dark-tint",
                    SurfaceFamily::steps(
                        dark_tint_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        true,
                        text_spread,
                    ),
                ),
                (
                    "surface-black",
                    SurfaceFamily::steps(black_base, step_size, 1, true, text_spread),
                ),
                (
                    "surface-primary-container",
                    SurfaceFamily::steps(
                        pri_cont_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        is_dark_surface,
                        text_spread,
                    ),
                ),
                (
                    "surface-secondary-container",
                    SurfaceFamily::steps(
                        sec_cont_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        is_dark_surface,
                        text_spread,
                    ),
                ),
                (
                    "surface-tertiary-container",
                    SurfaceFamily::steps(
                        ter_cont_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        is_dark_surface,
                        text_spread,
                    ),
                ),
                (
                    "surface-quaternary-container",
                    SurfaceFamily::steps(
                        qua_cont_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        is_dark_surface,
                        text_spread,
                    ),
                ),
                (
                    "surface-success-container",
                    SurfaceFamily::steps(
                        suc_cont_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        is_dark_surface,
                        text_spread,
                    ),
                ),
                (
                    "surface-danger-container",
                    SurfaceFamily::steps(
                        dan_cont_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        is_dark_surface,
                        text_spread,
                    ),
                ),
                (
                    "surface-warning-container",
                    SurfaceFamily::steps(
                        war_cont_base,
                        step_size,
                        if is_dark_surface { 1 } else { -1 },
                        is_dark_surface,
                        text_spread,
                    ),
                ),
            ],
        }
    }
}
