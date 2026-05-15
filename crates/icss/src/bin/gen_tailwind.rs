//! Generate a Tailwind CSS preset from the ICSS design system.
//!
//! Reads the same `default-vars.conf` used by the desktop app and outputs
//! a `tailwind.preset.js` for the admin web app.
//!
//! Usage:
//!   cargo run -p icss --bin gen_tailwind
//!   cargo run -p icss --bin gen_tailwind -- --output path/to/preset.js
//!   cargo run -p icss --bin gen_tailwind -- --vars path/to/vars.conf

use std::collections::HashMap;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse --vars and --output flags
    let mut vars_path: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--vars" => {
                i += 1;
                vars_path = args.get(i).cloned();
            }
            "--output" => {
                i += 1;
                output_path = args.get(i).cloned();
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Find workspace root (crates/icss → 2 levels up)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).join("../..");

    // Resolve vars config path
    let vars = if let Some(ref p) = vars_path {
        Path::new(p).to_path_buf()
    } else {
        let local = workspace_root.join("target/debug/showcase-vars.conf");
        let preset = workspace_root.join("apps/showcase/default-vars.conf");
        if local.exists() {
            local
        } else if preset.exists() {
            preset
        } else {
            eprintln!("No vars config found. Use --vars <path> or create default-vars.conf");
            std::process::exit(1);
        }
    };

    eprintln!("Reading theme vars from: {}", vars.display());
    let mut inputs = parse_vars(&vars);

    // Generate dark theme (matching the admin app's dark UI)
    inputs.dark_mode = true;
    inputs.surface_lightness = Some(5);

    let output = icss::engine::generate(&inputs);
    let preset = icss::engine::tailwind::compose_tailwind_preset(&output.colors, &output.dims);

    // Resolve output path
    let dest = if let Some(ref p) = output_path {
        Path::new(p).to_path_buf()
    } else {
        workspace_root.join("web/rl-admin/tailwind.preset.js")
    };

    std::fs::write(&dest, &preset).unwrap_or_else(|e| {
        eprintln!("Failed to write {}: {e}", dest.display());
        std::process::exit(1);
    });

    eprintln!("Wrote Tailwind preset to: {}", dest.display());
}

fn parse_vars(path: &Path) -> icss::engine::ThemeInputs {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let map: HashMap<&str, &str> = content
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect();

    let defaults = icss::engine::ThemeInputs::default();

    icss::engine::ThemeInputs {
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
        dark_mode: true,
        surface_lightness: map.get("surface_lightness").and_then(|v| v.parse().ok()),
        gamma: map.get("gamma").and_then(|v| v.parse().ok()),
        text_spread: map.get("text_spread").and_then(|v| v.parse().ok()),
        success_override: map
            .get("success_override")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        danger_override: map
            .get("danger_override")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        warning_override: map
            .get("warning_override")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
    }
}
