//! CSS-like theme engine for iced.
//!
//! Parses `.icss` files (a CSS subset with multi-class conjunctive selectors)
//! into widget styles for the iced GUI framework.
//!
//! # Theme file format
//!
//! ```css
//! :root {
//!   --primary: #0f3460;
//!   --text: #eaeaea;
//! }
//!
//! .button {
//!   color: #ffffff;
//!   border-radius: 8px;
//! }
//!
//! .primary {
//!   background-color: var(--primary);
//! }
//!
//! .primary:hover {
//!   background-color: #1a4a80;
//! }
//!
//! .button.primary {
//!   box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
//! }
//! ```
//!
//! # Rust API
//!
//! ```rust,ignore
//! let theme = icss::theme::Theme::load(include_str!("theme.icss"))?;
//! button("Connect").style(theme.button(&["button", "primary"]));
//! ```

pub mod color;
pub mod css;
pub mod resolve;

// Primary API.
pub use css::{ParseError, Stylesheet, parse_stylesheet};
pub use resolve::Theme;
