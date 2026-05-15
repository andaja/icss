//! Simple key=value persistence for ThemeVars.
//!
//! Saves to `showcase-vars.conf` next to the executable (or current dir).
//! Derived signal colors are not persisted — they're recomputed on load.

use crate::{FontFamily, ThemeVars};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Checked-in preset — used when no local config exists.
const PRESET: &str = include_str!("../default-vars.conf");

fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("showcase-vars.conf")))
        .unwrap_or_else(|| PathBuf::from("showcase-vars.conf"))
}

pub fn save(vars: &ThemeVars) {
    let content = format!(
        "\
primary={primary}
secondary={secondary}
tertiary={tertiary}
quaternary={quaternary}
neutral={neutral}
link={link}
increment={increment}
font_increment={font_increment}
radius_factor={radius_factor}
font_family={font_family}
dark_mode={dark_mode}
surface_lightness={surface_lightness}
gamma={gamma}
text_spread={text_spread}
dark_surface_lightness={dark_surface_lightness}
dark_gamma={dark_gamma}
dark_text_spread={dark_text_spread}
light_surface_lightness={light_surface_lightness}
light_gamma={light_gamma}
light_text_spread={light_text_spread}
success_override={success_override}
danger_override={danger_override}
warning_override={warning_override}
",
        primary = vars.primary,
        secondary = vars.secondary,
        tertiary = vars.tertiary,
        quaternary = vars.quaternary,
        neutral = vars.neutral,
        link = vars.link,
        increment = vars.increment,
        font_increment = vars.font_increment,
        radius_factor = vars.radius_factor,
        font_family = match vars.font_family {
            FontFamily::SFPro => "sfpro",
            FontFamily::SegoeUI => "segoeui",
            FontFamily::Roboto => "roboto",
        },
        dark_mode = vars.dark_mode,
        surface_lightness = vars.surface_lightness,
        gamma = vars.gamma,
        text_spread = vars.text_spread,
        dark_surface_lightness = vars.dark_surface_lightness,
        dark_gamma = vars.dark_gamma,
        dark_text_spread = vars.dark_text_spread,
        light_surface_lightness = vars.light_surface_lightness,
        light_gamma = vars.light_gamma,
        light_text_spread = vars.light_text_spread,
        success_override = vars.success_override,
        danger_override = vars.danger_override,
        warning_override = vars.warning_override,
    );

    let path = config_path();
    if let Err(e) = fs::write(&path, content) {
        tracing::warn!("Failed to save vars to {}: {e}", path.display());
    }
}

/// Load theme vars: local config → checked-in preset → Default.
pub fn load() -> Option<ThemeVars> {
    let path = config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        return Some(parse_vars(&content));
    }
    // No local config — use checked-in preset so fresh clones get the
    // reference theme without needing to copy files around.
    Some(parse_vars(PRESET))
}

fn parse_vars(content: &str) -> ThemeVars {
    let map: HashMap<&str, &str> = content
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect();

    let defaults = ThemeVars::default();

    ThemeVars {
        primary: map
            .get("primary")
            .unwrap_or(&defaults.primary.as_str())
            .to_string(),
        secondary: map
            .get("secondary")
            .unwrap_or(&defaults.secondary.as_str())
            .to_string(),
        tertiary: map
            .get("tertiary")
            .unwrap_or(&defaults.tertiary.as_str())
            .to_string(),
        quaternary: map
            .get("quaternary")
            .unwrap_or(&defaults.quaternary.as_str())
            .to_string(),
        neutral: map
            .get("neutral")
            .unwrap_or(&defaults.neutral.as_str())
            .to_string(),
        link: map
            .get("link")
            .unwrap_or(&defaults.link.as_str())
            .to_string(),
        increment: map
            .get("increment")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.increment),
        font_increment: map
            .get("font_increment")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.font_increment),
        radius_factor: map
            .get("radius_factor")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.radius_factor),
        font_family: match *map.get("font_family").unwrap_or(&"roboto") {
            "sfpro" => FontFamily::SFPro,
            "segoeui" => FontFamily::SegoeUI,
            _ => FontFamily::Roboto,
        },
        dark_mode: map
            .get("dark_mode")
            .map(|v| *v == "true")
            .unwrap_or(defaults.dark_mode),
        surface_lightness: map
            .get("surface_lightness")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.surface_lightness),
        gamma: map
            .get("gamma")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.gamma),
        text_spread: map
            .get("text_spread")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.text_spread),
        dark_surface_lightness: map
            .get("dark_surface_lightness")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.dark_surface_lightness),
        dark_gamma: map
            .get("dark_gamma")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.dark_gamma),
        dark_text_spread: map
            .get("dark_text_spread")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.dark_text_spread),
        light_surface_lightness: map
            .get("light_surface_lightness")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.light_surface_lightness),
        light_gamma: map
            .get("light_gamma")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.light_gamma),
        light_text_spread: map
            .get("light_text_spread")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.light_text_spread),
        success_override: map
            .get("success_override")
            .map(|s| s.to_string())
            .unwrap_or_default(),
        danger_override: map
            .get("danger_override")
            .map(|s| s.to_string())
            .unwrap_or_default(),
        warning_override: map
            .get("warning_override")
            .map(|s| s.to_string())
            .unwrap_or_default(),
        derived_success: String::new(),
        derived_danger: String::new(),
        derived_warning: String::new(),
    }
}
