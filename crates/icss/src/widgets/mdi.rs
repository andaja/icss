//! Material Design Icons (MDI) from Pictogrammers for use with iced's SVG renderer.
//!
//! Each icon is a filled-path constant in a 24x24 viewBox (MDI standard).
//!
//! Usage in custom widgets (advanced renderer):
//! ```ignore
//! use icss::widgets::mdi;
//! let svg = mdi::icon_svg(mdi::HOUSE, Some(Color::WHITE));
//! renderer.draw_svg(svg, bounds, clip_bounds);
//! ```
//!
//! Usage in iced widget tree:
//! ```ignore
//! use icss::widgets::mdi;
//! iced::widget::svg(mdi::icon_handle(mdi::HOUSE)).width(16).height(16)
//! ```

use iced::Color;
use iced::advanced::svg::{Handle, Svg};

/// Raw SVG path data (the `d` attribute of a `<path>` element in a 24x24 viewBox).
pub type IconData = &'static str;

// ---------------------------------------------------------------------------
// Sidebar icons
// ---------------------------------------------------------------------------

/// Monitor — Devices
pub const MONITOR: IconData = "M21,16H3V4H21M21,2H3C1.89,2 1,2.89 1,4V16A2,2 0 0,0 3,18H10V20H8V22H16V20H14V18H21A2,2 0 0,0 23,16V4C23,2.89 22.1,2 21,2Z";

/// History — History
pub const CLOCK: IconData = "M13.5,8H12V13L16.28,15.54L17,14.33L13.5,12.25V8M13,3A9,9 0 0,0 4,12H1L4.96,16.03L9,12H6A7,7 0 0,1 13,5A7,7 0 0,1 20,12A7,7 0 0,1 13,19C11.07,19 9.32,18.21 8.06,16.94L6.64,18.36C8.27,20 10.5,21 13,21A9,9 0 0,0 22,12A9,9 0 0,0 13,3";

/// Chat — Messages
pub const MESSAGE_SQUARE: IconData = "M12,3C17.5,3 22,6.58 22,11C22,15.42 17.5,19 12,19C10.76,19 9.57,18.82 8.47,18.5C5.55,21 2,21 2,21C4.33,18.67 4.7,17.1 4.75,16.5C3.05,15.07 2,13.13 2,11C2,6.58 6.5,3 12,3Z";

/// Cog — Settings
pub const SETTINGS: IconData = "M12,15.5A3.5,3.5 0 0,1 8.5,12A3.5,3.5 0 0,1 12,8.5A3.5,3.5 0 0,1 15.5,12A3.5,3.5 0 0,1 12,15.5M19.43,12.97C19.47,12.65 19.5,12.33 19.5,12C19.5,11.67 19.47,11.34 19.43,11L21.54,9.37C21.73,9.22 21.78,8.95 21.66,8.73L19.66,5.27C19.54,5.05 19.27,4.96 19.05,5.05L16.56,6.05C16.04,5.66 15.5,5.32 14.87,5.07L14.5,2.42C14.46,2.18 14.25,2 14,2H10C9.75,2 9.54,2.18 9.5,2.42L9.13,5.07C8.5,5.32 7.96,5.66 7.44,6.05L4.95,5.05C4.73,4.96 4.46,5.05 4.34,5.27L2.34,8.73C2.21,8.95 2.27,9.22 2.46,9.37L4.57,11C4.53,11.34 4.5,11.67 4.5,12C4.5,12.33 4.53,12.65 4.57,12.97L2.46,14.63C2.27,14.78 2.21,15.05 2.34,15.27L4.34,18.73C4.46,18.95 4.73,19.03 4.95,18.95L7.44,17.94C7.96,18.34 8.5,18.68 9.13,18.93L9.5,21.58C9.54,21.82 9.75,22 10,22H14C14.25,22 14.46,21.82 14.5,21.58L14.87,18.93C15.5,18.67 16.04,18.34 16.56,17.94L19.05,18.95C19.27,19.03 19.54,18.95 19.66,18.73L21.66,15.27C21.78,15.05 21.73,14.78 21.54,14.63L19.43,12.97Z";

/// Bug — Debug
pub const BUG: IconData = "M14,12H10V10H14M14,16H10V14H14M20,8H17.19C16.74,7.22 16.12,6.55 15.37,6.04L17,4.41L15.59,3L13.42,5.17C12.96,5.06 12.5,5 12,5C11.5,5 11.04,5.06 10.59,5.17L8.41,3L7,4.41L8.62,6.04C7.88,6.55 7.26,7.22 6.81,8H4V10H6.09C6.04,10.33 6,10.66 6,11V12H4V14H6V15C6,15.34 6.04,15.67 6.09,16H4V18H6.81C7.85,19.79 9.78,21 12,21C14.22,21 16.15,19.79 17.19,18H20V16H17.91C17.96,15.67 18,15.34 18,15V14H20V12H18V11C18,10.66 17.96,10.33 17.91,10H20V8Z";

// ---------------------------------------------------------------------------
// Tab bar icons
// ---------------------------------------------------------------------------

/// Home — Home tab
pub const HOUSE: IconData = "M10,20V14H14V20H19V12H22L12,3L2,12H5V20H10Z";

/// Close — X button
pub const X: IconData = "M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z";

/// Weather night — Dark mode (moon)
pub const MOON: IconData = "M17.75,4.09L15.22,6.03L16.13,9.09L13.5,7.28L10.87,9.09L11.78,6.03L9.25,4.09L12.44,4L13.5,1L14.56,4L17.75,4.09M21.25,11L19.61,12.25L20.2,14.23L18.5,13.06L16.8,14.23L17.39,12.25L15.75,11L17.81,10.95L18.5,9L19.19,10.95L21.25,11M18.97,15.95C19.8,15.87 20.69,17.05 20.16,17.8C19.84,18.25 19.5,18.67 19.08,19.07C15.17,23 8.84,23 4.94,19.07C1.03,15.17 1.03,8.83 4.94,4.93C5.34,4.53 5.76,4.17 6.21,3.85C6.96,3.32 8.14,4.21 8.06,5.04C7.79,7.9 8.75,10.87 10.95,13.06C13.14,15.26 16.1,16.22 18.97,15.95M17.33,17.97C14.5,17.81 11.7,16.64 9.53,14.5C7.36,12.31 6.2,9.5 6.04,6.68C3.23,9.82 3.34,14.64 6.35,17.66C9.37,20.67 14.19,20.78 17.33,17.97Z";

/// White balance sunny — Light mode (sun)
pub const SUN: IconData = "M3.55 19.09L4.96 20.5L6.76 18.71L5.34 17.29M12 6C8.69 6 6 8.69 6 12S8.69 18 12 18 18 15.31 18 12C18 8.68 15.31 6 12 6M20 13H23V11H20M17.24 18.71L19.04 20.5L20.45 19.09L18.66 17.29M20.45 5L19.04 3.6L17.24 5.39L18.66 6.81M13 1H11V4H13M6.76 5.39L4.96 3.6L3.55 5L5.34 6.81L6.76 5.39M1 13H4V11H1M13 20H11V23H13";

/// Circle (filled) — Connection indicator
pub const CIRCLE: IconData =
    "M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z";

/// Circle outline (ring) — Disconnected indicator
pub const CIRCLE_OUTLINE: IconData = "M12,20A8,8 0 0,1 4,12A8,8 0 0,1 12,4A8,8 0 0,1 20,12A8,8 0 0,1 12,20M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z";

/// Wifi — Connected
pub const WIFI: IconData = "M12,21L15.6,16.2C14.6,15.45 13.35,15 12,15C10.65,15 9.4,15.45 8.4,16.2L12,21M12,3C7.95,3 4.21,4.34 1.2,6.6L3,9C5.5,7.12 8.62,6 12,6C15.38,6 18.5,7.12 21,9L22.8,6.6C19.79,4.34 16.05,3 12,3M12,9C9.3,9 6.81,9.89 4.8,11.4L6.6,13.8C8.1,12.67 9.97,12 12,12C14.03,12 15.9,12.67 17.4,13.8L19.2,11.4C17.19,9.89 14.7,9 12,9Z";

/// Dots vertical — App menu
pub const ELLIPSIS_VERTICAL: IconData = "M12,16A2,2 0 0,1 14,18A2,2 0 0,1 12,20A2,2 0 0,1 10,18A2,2 0 0,1 12,16M12,10A2,2 0 0,1 14,12A2,2 0 0,1 12,14A2,2 0 0,1 10,12A2,2 0 0,1 12,10M12,4A2,2 0 0,1 14,6A2,2 0 0,1 12,8A2,2 0 0,1 10,6A2,2 0 0,1 12,4Z";

// ---------------------------------------------------------------------------
// Viewer toolbar icons
// ---------------------------------------------------------------------------

/// Fullscreen — Maximize
pub const MAXIMIZE: IconData =
    "M5,5H10V7H7V10H5V5M14,5H19V10H17V7H14V5M17,14H19V19H14V17H17V14M10,17V19H5V14H7V17H10Z";

/// Fullscreen exit — Minimize / exit fullscreen
pub const MINIMIZE: IconData =
    "M14,14H19V16H16V19H14V14M5,14H10V19H8V16H5V14M8,5H10V10H5V8H8V5M19,8V10H14V5H16V8H19Z";

/// Window minimize — horizontal line (Windows titlebar control)
pub const WINDOW_MINIMIZE: IconData = "M20,14H4V12H20V14Z";

/// Window maximize — square outline (Windows titlebar control)
pub const WINDOW_MAXIMIZE: IconData = "M4,4H20V20H4V4M6,6V18H18V6H6Z";

/// Folder — File transfer
pub const FOLDER: IconData =
    "M10,4H4C2.89,4 2,4.89 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V8C22,6.89 21.1,6 20,6H12L10,4Z";

/// Logout — Disconnect
pub const LOG_OUT: IconData = "M17 7L15.59 8.41L18.17 11H8V13H18.17L15.59 15.58L17 17L22 12M4 5H12V3H4C2.9 3 2 3.9 2 5V19C2 20.1 2.9 21 4 21H12V19H4V5Z";

/// Refresh — Reconnect
pub const REFRESH_CW: IconData = "M17.65,6.35C16.2,4.9 14.21,4 12,4A8,8 0 0,0 4,12A8,8 0 0,0 12,20C15.73,20 18.84,17.45 19.73,14H17.65C16.83,16.33 14.61,18 12,18A6,6 0 0,1 6,12A6,6 0 0,1 12,6C13.66,6 15.14,6.69 16.22,7.78L13,11H20V4L17.65,6.35Z";

/// Send — Chat send button
pub const SEND: IconData = "M2,21L23,12L2,3V10L17,12L2,14V21Z";

/// Microphone — Mic active / unmuted
pub const MICROPHONE: IconData = "M12,2A3,3 0 0,1 15,5V11A3,3 0 0,1 12,14A3,3 0 0,1 9,11V5A3,3 0 0,1 12,2M19,11C19,14.53 16.39,17.44 13,17.93V21H11V17.93C7.61,17.44 5,14.53 5,11H7A5,5 0 0,0 12,16A5,5 0 0,0 17,11H19Z";

/// Microphone off — Mic muted
pub const MICROPHONE_OFF: IconData = "M19,11H17.3C17.3,11.74 17.14,12.43 16.87,13.05L18.1,14.28C18.66,13.3 19,12.19 19,11M15,11.16L9,5.18V5A3,3 0 0,1 12,2A3,3 0 0,1 15,5V11L15,11.16M4.27,3L21,19.73L19.73,21L15.54,16.81C14.77,17.27 13.91,17.58 13,17.72V21H11V17.72C7.72,17.23 5,14.41 5,11H6.7C6.7,14 9.24,16.1 12,16.1C12.81,16.1 13.6,15.91 14.31,15.58L12.65,13.92C12.44,13.97 12.22,14 12,14A3,3 0 0,1 9,11V10.28L3,4.27L4.27,3Z";

/// Cursor default — Follow cursor / mouse pointer
pub const MOUSE_POINTER: IconData = "M13.64,21.97C13.14,22.21 12.54,22 12.31,21.5L10.13,16.76L7.62,18.78C7.45,18.92 7.24,19 7,19A1,1 0 0,1 6,18V3A1,1 0 0,1 7,2C7.24,2 7.47,2.09 7.64,2.23L7.65,2.22L19.14,11.86C19.57,12.22 19.62,12.85 19.27,13.27C19.12,13.45 18.91,13.57 18.7,13.61L15.54,14.23L17.74,18.96C18,19.46 17.76,20.05 17.26,20.28L13.64,21.97Z";

/// View split vertical — Multi-window / columns
pub const COLUMNS_2: IconData =
    "M13,5H21V19H13V5M3,5H11V7H3V5M3,11V9H11V11H3M3,19V17H11V19H3M3,15V13H11V15H3Z";

/// Crop free — Scale / fit mode
pub const SCAN: IconData = "M19,3H15V5H19V9H21V5C21,3.89 20.1,3 19,3M19,19H15V21H19A2,2 0 0,0 21,19V15H19M5,15H3V19A2,2 0 0,0 5,21H9V19H5M3,5V9H5V5H9V3H5A2,2 0 0,0 3,5Z";

/// Information — Device / session info
pub const INFO: IconData = "M13,9H11V7H13M13,17H11V11H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z";

/// Keyboard — Keyboard settings
pub const KEYBOARD: IconData = "M19,10H17V8H19M19,13H17V11H19M16,10H14V8H16M16,13H14V11H16M16,17H8V15H16M7,10H5V8H7M7,13H5V11H7M8,11H10V13H8M8,8H10V10H8M11,11H13V13H11M11,8H13V10H11M20,5H4C2.89,5 2,5.89 2,7V17A2,2 0 0,0 4,19H20A2,2 0 0,0 22,17V7C22,5.89 21.1,5 20,5Z";

/// Shield half full — Permissions
pub const SHIELD_HALF: IconData = "M21,11C21,16.55 17.16,21.74 12,23C6.84,21.74 3,16.55 3,11V5L12,1L21,5V11M12,21C15.75,20 19,15.54 19,11.22V6.3L12,3.18V21Z";

/// Flash — Actions menu / zap
pub const ZAP: IconData = "M7,2V13H10V22L17,10H13L17,2H7Z";

/// Chevron down — Dropdown indicator
pub const CHEVRON_DOWN: IconData = "M7.41,8.58L12,13.17L16.59,8.58L18,10L12,16L6,10L7.41,8.58Z";

/// Chevron left — Back / collapse
pub const CHEVRON_LEFT: IconData = "M15.41,16.58L10.83,12L15.41,7.41L14,6L8,12L14,18L15.41,16.58Z";

/// Chevron right — Forward / expand
pub const CHEVRON_RIGHT: IconData = "M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z";

// ---------------------------------------------------------------------------
// History icons
// ---------------------------------------------------------------------------

/// Key — Unattended / auto-accepted session
pub const KEY: IconData = "M7 14C5.9 14 5 13.1 5 12S5.9 10 7 10 9 10.9 9 12 8.1 14 7 14M12.6 10C11.8 7.7 9.6 6 7 6C3.7 6 1 8.7 1 12S3.7 18 7 18C9.6 18 11.8 16.3 12.6 14H16V18H20V14H23V10H12.6Z";

/// Account — User-accepted session
pub const USER: IconData = "M12,4A4,4 0 0,1 16,8A4,4 0 0,1 12,12A4,4 0 0,1 8,8A4,4 0 0,1 12,4M12,14C16.42,14 20,15.79 20,18V20H4V18C4,15.79 7.58,14 12,14Z";

/// File document — Log / chat transcript
pub const FILE_TEXT: IconData = "M13,9H18.5L13,3.5V9M6,2H14L20,8V20A2,2 0 0,1 18,22H6C4.89,22 4,21.1 4,20V4C4,2.89 4.89,2 6,2M15,18V16H6V18H15M18,14V12H6V14H18Z";

/// Play — Connect / reconnect
pub const PLAY: IconData = "M8,5.14V19.14L19,12.14L8,5.14Z";

/// Arrow top right — Outgoing connection
pub const ARROW_UP_RIGHT: IconData = "M5,17.59L15.59,7H9V5H19V15H17V8.41L6.41,19L5,17.59Z";

/// Arrow bottom left — Incoming connection
pub const ARROW_DOWN_LEFT: IconData = "M19,6.41L17.59,5L7,15.59V9H5V19H15V17H8.41L19,6.41Z";

// ---------------------------------------------------------------------------
// Accept prompt icons
// ---------------------------------------------------------------------------

/// Shield — Incoming connection / permissions
pub const SHIELD: IconData =
    "M12,1L3,5V11C3,16.55 6.84,21.74 12,23C17.16,21.74 21,16.55 21,11V5L12,1Z";

/// Shield check — Accept / approved
pub const SHIELD_CHECK: IconData = "M10,17L6,13L7.41,11.59L10,14.17L16.59,7.58L18,9M12,1L3,5V11C3,16.55 6.84,21.74 12,23C17.16,21.74 21,16.55 21,11V5L12,1Z";

/// Shield remove — Deny
pub const SHIELD_X: IconData = "M19.43,19L21.5,21.11L20.12,22.5L18.03,20.41L15.91,22.53L14.5,21.11L16.61,19L14.5,16.86L15.88,15.47L18,17.59L20.12,15.47L21.55,16.9L19.43,19M12,1L21,5V11C21,11.9 20.9,12.78 20.71,13.65C19.9,13.23 19,13 18,13A6,6 0 0,0 12,19C12,20.36 12.45,21.62 13.22,22.62L12,23C6.84,21.74 3,16.55 3,11V5L12,1Z";

/// Checkmark
pub const CHECK: IconData = "M21,7L9,19L3.5,13.5L4.91,12.09L9,16.17L19.59,5.59L21,7Z";

/// Timer — Countdown
pub const TIMER: IconData = "M19.03 7.39L20.45 5.97C20 5.46 19.55 5 19.04 4.56L17.62 6C16.07 4.74 14.12 4 12 4C7.03 4 3 8.03 3 13S7.03 22 12 22C17 22 21 17.97 21 13C21 10.88 20.26 8.93 19.03 7.39M13 14H11V7H13V14M15 1H9V3H15V1Z";

/// Bell — Auto-DnD indicator badge (active = OS notifications still fire).
pub const BELL: IconData = "M14 20A2 2 0 0 1 12 22A2 2 0 0 1 10 20H14M12 2A1 1 0 0 1 13 3V4.08C15.84 4.56 18 7.03 18 10V16L20 18V19H4V18L6 16V10C6 7.03 8.16 4.56 11 4.08V3A1 1 0 0 1 12 2Z";
/// Bell with strikethrough — Auto-DnD engaged (OS notifications silenced).
pub const BELL_OFF: IconData = "M3 6.27L4.28 5L21 21.72L19.73 23L17.71 21H4V20L6 18V12C6 10.95 6.21 9.94 6.59 9L3 6.27M16 16.86L8.41 9.27C8.85 8.18 9.81 7.4 11 7.1V6A1 1 0 0 1 12 5A1 1 0 0 1 13 6V7.1C15.16 7.63 16.79 9.62 17 12V16L16 16.86M14 21A2 2 0 0 1 12 23A2 2 0 0 1 10 21H14Z";

// ---------------------------------------------------------------------------
// Chat icons
// ---------------------------------------------------------------------------

/// Paperclip — Attachment
pub const PAPERCLIP: IconData = "M16.5,6V17.5A4,4 0 0,1 12.5,21.5A4,4 0 0,1 8.5,17.5V5A2.5,2.5 0 0,1 11,2.5A2.5,2.5 0 0,1 13.5,5V15.5A1,1 0 0,1 12.5,16.5A1,1 0 0,1 11.5,15.5V6H10V15.5A2.5,2.5 0 0,1 12.5,18A2.5,2.5 0 0,1 15,15.5V5A4,4 0 0,1 11,1A4,4 0 0,1 7,5V17.5A5.5,5.5 0 0,0 12.5,23A5.5,5.5 0 0,0 18,17.5V6H16.5Z";

/// Content Copy
pub const COPY: IconData = "M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z";

/// Close / X mark (alias for X)
pub const CLOSE: IconData = X;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Wrap icon data into a full SVG document string.
/// MDI icons are filled paths — no stroke, just `fill="currentColor"`.
fn svg_document(icon: IconData) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="{icon}"/></svg>"#
    )
}

/// Create an SVG [`Handle`] from icon data (for use with `iced::widget::svg`).
pub fn icon_handle(icon: IconData) -> Handle {
    Handle::from_memory(svg_document(icon).into_bytes())
}

/// Create an SVG [`Handle`] — stroke_width parameter accepted for API compat but ignored
/// (MDI icons are filled, not stroked).
pub fn icon_handle_sw(icon: IconData, _stroke_width: f32) -> Handle {
    icon_handle(icon)
}

/// Create an [`Svg`] ready for `renderer.draw_svg()` with an optional color tint.
pub fn icon_svg(icon: IconData, color: Option<Color>) -> Svg {
    let mut svg = Svg::new(icon_handle(icon));
    if let Some(c) = color {
        svg = svg.color(c);
    }
    svg
}

/// Create an [`Svg`] with optional color tint — stroke_width accepted for API compat
/// but ignored (MDI icons are filled, not stroked).
pub fn icon_svg_sw(icon: IconData, _stroke_width: f32, color: Option<Color>) -> Svg {
    icon_svg(icon, color)
}
