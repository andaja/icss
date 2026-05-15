//! One-call widget builders — CSS-like ergonomics for iced widgets.
//!
//! These methods combine sizing, styling, and widget construction into a single
//! call, mirroring how CSS classes work: you pass a class list and get a fully
//! configured widget back.
//!
//! # Before (manual wiring)
//!
//! ```rust,ignore
//! let sz = t.sizing(&["sz-md"]);
//! let sb = font_family.weighted(Weight::Semibold);
//! button(text("Accept").size(sz.font_size).font(sb))
//!     .padding(sz.padding())
//!     .style(t.button(&["button", "primary"]))
//!     .on_press(Msg::Accept)
//! ```
//!
//! # After (one-call builder)
//!
//! ```rust,ignore
//! t.btn("Accept", &["button", "primary", "sz-md"])
//!     .on_press(Msg::Accept)
//! ```
//!
//! Font size, padding, gap, weight, colors, hover/active/disabled states —
//! all resolved from the class list. The returned widget can still be chained
//! with `.width()`, `.on_press()`, etc.

use std::borrow::Borrow;
use std::ops::RangeInclusive;

use iced::widget::text::IntoFragment;
use iced::widget::{
    button, checkbox, pick_list, progress_bar, radio, rule, slider, text, text_editor, text_input,
    toggler,
};
use iced::{Alignment, Element, Font, Length, Padding, Pixels, Theme as IcedTheme, font};

use crate::theme::resolve::Theme;
use crate::theme::resolve::sizing::ComponentSize;
use crate::theme::resolve::text::weight_from_css;

impl Theme {
    // ── Internal helpers ─────────────────────────────────────────────

    /// Resolve horizontal alignment from `text-align` CSS property.
    fn resolve_halign(
        &self,
        classes: &[&str],
        default: iced::alignment::Horizontal,
    ) -> iced::alignment::Horizontal {
        let computed = self.resolve(classes, None);
        match computed.keyword("text-align") {
            Some("center") => iced::alignment::Horizontal::Center,
            Some("left") => iced::alignment::Horizontal::Left,
            Some("right") => iced::alignment::Horizontal::Right,
            _ => default,
        }
    }

    /// Resolve vertical alignment from `vertical-align` CSS property.
    fn resolve_valign(
        &self,
        classes: &[&str],
        default: iced::alignment::Vertical,
    ) -> iced::alignment::Vertical {
        let computed = self.resolve(classes, None);
        match computed.keyword("vertical-align") {
            Some("center") => iced::alignment::Vertical::Center,
            Some("top") => iced::alignment::Vertical::Top,
            Some("bottom") => iced::alignment::Vertical::Bottom,
            _ => default,
        }
    }

    /// Compute button padding with optional vertical shift from ICSS `padding-offset`.
    /// Positive offset = more top, less bottom (shifts content down).
    fn btn_padding(&self, sz: &ComponentSize, classes: &[&str]) -> Padding {
        let computed = self.resolve(classes, None);
        let offset = computed.length("padding-offset").unwrap_or(0.0);
        Padding {
            top: sz.pad_v + offset,
            bottom: (sz.pad_v - offset).max(0.0),
            left: sz.pad_h,
            right: sz.pad_h,
        }
    }

    /// Resolve font from classes, using `default_weight` when CSS doesn't set one.
    fn resolve_font(&self, classes: &[&str], default_weight: font::Weight) -> Font {
        let computed = self.resolve(classes, None);
        let weight = computed
            .number("font-weight")
            .map(weight_from_css)
            .unwrap_or(default_weight);
        Font {
            weight,
            ..self.default_font()
        }
    }

    /// Make styled button text with correct line-height for vertical centering.
    ///
    /// Without explicit `LineHeight::Absolute`, iced's default line-height adds
    /// extra space below the text baseline, making top/bottom padding look
    /// asymmetric even when the padding values are equal.
    fn btn_text<'a, Message: 'a>(
        &self,
        label: impl IntoFragment<'a>,
        sz: &ComponentSize,
        f: Font,
        type_class: &str,
        classes: &[&str],
    ) -> iced::widget::Container<'a, Message, IcedTheme> {
        // Read baseline-offset from ICSS (e.g. .btn-label.sz-md { baseline-offset: 2px; }).
        let mut resolve_classes: Vec<&str> = classes.to_vec();
        resolve_classes.push(type_class);
        let computed = self.resolve(&resolve_classes, None);
        let offset = computed.length("baseline-offset").unwrap_or(0.0);

        iced::widget::container(
            text(label)
                .size(sz.font_size)
                .line_height(iced::widget::text::LineHeight::Absolute(Pixels(
                    sz.font_size,
                )))
                .font(f),
        )
        .padding(Padding {
            top: offset,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
    }

    // ── Buttons ──────────────────────────────────────────────────────

    /// Build a fully styled text button from a class list.
    ///
    /// ```rust,ignore
    /// t.btn("Accept", &["button", "primary", "sz-md"])
    ///     .on_press(Msg::Accept)
    /// ```
    ///
    /// Classes should include the button variant (`"button"`, `"primary"`,
    /// `"success"`, etc.) and a sizing class (`"sz-md"`, `"sz-sm"`, `"sz-xs"`).
    /// Font weight defaults to semibold; override via CSS `font-weight`.
    pub fn btn<'a, Message: Clone + 'a>(
        &'a self,
        label: impl IntoFragment<'a>,
        classes: &'a [&'a str],
    ) -> button::Button<'a, Message, IcedTheme> {
        let sz = self.sizing(classes);
        let f = self.resolve_font(classes, font::Weight::Semibold);

        let ha = self.resolve_halign(classes, iced::alignment::Horizontal::Center);
        let va = self.resolve_valign(classes, iced::alignment::Vertical::Center);

        let content = iced::widget::container(self.btn_text(label, &sz, f, "btn-label", classes))
            .align_x(ha)
            .align_y(va);

        button(content)
            .padding(self.btn_padding(&sz, classes))
            .style(self.button(classes))
    }

    /// Build a button with an icon element + text label.
    ///
    /// ```rust,ignore
    /// // With SVG icon (what rl-desktop uses):
    /// let icon = svg(mdi::icon_handle(mdi::CHECK)).width(sz).height(sz);
    /// t.btn_icon(icon, "Accept", &["button", "success", "sz-md"])
    ///     .on_press(Msg::Accept)
    /// ```
    ///
    /// The icon can be any `Element` — SVG, text icon, container, etc.
    /// The builder handles layout (row, gap, padding, button style).
    /// The caller is responsible for sizing the icon to match (use
    /// `t.sizing(classes).font_size` for the icon dimensions).
    pub fn btn_icon<'a, Message: Clone + 'a>(
        &'a self,
        icon: impl Into<Element<'a, Message, IcedTheme>>,
        label: impl IntoFragment<'a>,
        classes: &'a [&'a str],
    ) -> button::Button<'a, Message, IcedTheme> {
        let sz = self.sizing(classes);
        let f = self.resolve_font(classes, font::Weight::Semibold);

        let content = iced::widget::container(
            iced::widget::row![
                icon.into(),
                self.btn_text(label, &sz, f, "btn-icon-label", classes)
            ]
            .spacing(sz.gap)
            .align_y(Alignment::Center),
        )
        .center_x(Length::Shrink);

        // Icon+label buttons use symmetric padding — the text baseline
        // compensation is handled inside btn_text via baseline-offset.
        // padding-offset only applies to text-only buttons (btn).
        button(content)
            .padding(sz.padding())
            .style(self.button(classes))
    }

    /// Build a themed SVG icon — colour resolved from a class set.
    ///
    /// Resolution order (one step softer than the matching text colour):
    ///
    /// 1. `icon-color` (explicit; future-proof).
    /// 2. `accent-color` — for button rules in the engine this is
    ///    `var(--on-surface-X-soft)`, exactly one step softer than the
    ///    button text's `color: var(--on-surface-X-default)`.
    /// 3. `color` — same brightness as the text; visually matches.
    /// 4. `--text-muted` from `:root` — neutral fallback when the class
    ///    set declares nothing colour-related (e.g. a bare `mouse_area`).
    /// 5. Solid grey — only if the theme has no `--text-muted` either.
    ///
    /// Pre-clamping is unnecessary: the values come from the parsed ICSS,
    /// which clamps internally.
    pub fn themed_icon<'a>(
        &'a self,
        handle: iced::widget::svg::Handle,
        size: f32,
        classes: &[&str],
    ) -> iced::widget::Svg<'a> {
        let computed = self.resolve(classes, None);
        let resolved = Self::resolve_color(&computed, "icon-color")
            .or_else(|| Self::resolve_color(&computed, "accent-color"))
            .or_else(|| Self::resolve_color(&computed, "color"))
            .map(|c| c.to_iced())
            .or_else(|| self.color_var("text-muted"))
            .unwrap_or(iced::Color::from_rgb(0.7, 0.7, 0.7));
        iced::widget::svg(handle)
            .width(size)
            .height(size)
            .style(move |_, _| iced::widget::svg::Style {
                color: Some(resolved),
            })
    }

    /// Build a square icon-only button.
    ///
    /// ```rust,ignore
    /// t.btn_sq(icon_element, &["button", "ghost", "sz-md"])
    ///     .on_press(Msg::Settings)
    /// ```
    ///
    /// The button is square — side length matches min_h from ICSS so
    /// icon-only buttons have the same height as text and icon+label buttons.
    pub fn btn_sq<'a, Message: Clone + 'a>(
        &'a self,
        icon: impl Into<Element<'a, Message, IcedTheme>>,
        classes: &'a [&'a str],
    ) -> button::Button<'a, Message, IcedTheme> {
        let sz = self.sizing(classes);
        let side = sz.min_h.unwrap_or(sz.font_size + 2.0 * sz.pad_v);

        let content = iced::widget::container(icon)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        button(content)
            .padding(Padding::ZERO)
            .width(side)
            .height(side)
            .style(self.button(classes))
    }

    // ── Text Input ───────────────────────────────────────────────────

    /// Build a fully styled text input from a class list.
    ///
    /// ```rust,ignore
    /// t.input("Search...", &state.query, &["input", "sz-md"])
    ///     .on_input(Msg::QueryChanged)
    /// ```
    pub fn input<'a, Message: Clone + 'a>(
        &'a self,
        placeholder: &str,
        value: &str,
        classes: &'a [&'a str],
    ) -> text_input::TextInput<'a, Message, IcedTheme> {
        let sz = self.sizing(classes);

        text_input(placeholder, value)
            .size(sz.font_size)
            .padding(sz.padding())
            .style(self.text_input(classes))
    }

    // ── Checkbox ─────────────────────────────────────────────────────

    /// Build a fully styled checkbox from a class list.
    ///
    /// ```rust,ignore
    /// t.check(is_checked, "Remember me", &["checkbox", "sz-md"])
    ///     .on_toggle(Msg::RememberToggled)
    /// ```
    pub fn check<'a, Message: 'a>(
        &'a self,
        is_checked: bool,
        label: impl IntoFragment<'a>,
        classes: &'a [&'a str],
    ) -> checkbox::Checkbox<'a, Message, IcedTheme> {
        let sz = self.sizing(classes);

        checkbox(is_checked)
            .label(label)
            .size(sz.font_size)
            .text_size(sz.font_size)
            .spacing(sz.gap)
            .style(self.checkbox(classes))
    }

    // ── Toggler ──────────────────────────────────────────────────────

    /// Build a fully styled toggler from a class list.
    ///
    /// ```rust,ignore
    /// t.toggle("Dark mode", is_dark, &["toggle", "sz-md"])
    ///     .on_toggle(Msg::DarkModeChanged)
    /// ```
    pub fn toggle<'a, Message: 'a>(
        &'a self,
        label: impl IntoFragment<'a>,
        is_toggled: bool,
        classes: &'a [&'a str],
    ) -> toggler::Toggler<'a, Message, IcedTheme> {
        let sz = self.sizing(classes);

        toggler(is_toggled)
            .label(label)
            .size(sz.font_size)
            .text_size(sz.font_size)
            .spacing(sz.gap)
            .style(self.toggler(classes))
    }

    // ── Radio ────────────────────────────────────────────────────────

    /// Build a fully styled radio button from a class list.
    ///
    /// ```rust,ignore
    /// t.radio_btn("Option A", MyEnum::A, state.selected, Msg::Selected, &["radio", "sz-md"])
    /// ```
    ///
    /// Note: iced's `radio()` constructor requires the `on_click` callback,
    /// so it must be passed here (unlike buttons where `.on_press()` is optional).
    pub fn radio_btn<'a, Message, V>(
        &'a self,
        label: impl Into<String>,
        value: V,
        selected: Option<V>,
        on_click: impl FnOnce(V) -> Message + 'a,
        classes: &'a [&'a str],
    ) -> radio::Radio<'a, Message, IcedTheme>
    where
        Message: Clone + 'a,
        V: Copy + Eq + 'a,
    {
        let sz = self.sizing(classes);
        // iced radio size must be even for pixel-perfect rendering.
        let size = {
            let n = sz.font_size.round() as i32;
            (if n & 1 == 1 { n + 1 } else { n }) as f32
        };

        radio(label, value, selected, on_click)
            .size(size)
            .text_size(sz.font_size)
            .spacing(sz.gap)
            .style(self.radio(classes))
    }

    // ── Slider ───────────────────────────────────────────────────────

    /// Build a fully styled slider from a class list.
    ///
    /// ```rust,ignore
    /// t.slide(0.0..=100.0, state.volume, Msg::VolumeChanged, &["slider"])
    /// ```
    pub fn slide<'a, Message>(
        &'a self,
        range: RangeInclusive<f32>,
        value: f32,
        on_change: impl Fn(f32) -> Message + 'a,
        classes: &'a [&'a str],
    ) -> slider::Slider<'a, f32, Message, IcedTheme>
    where
        Message: Clone + 'a,
    {
        slider(range, value, on_change).style(self.slider(classes))
    }

    // ── Progress Bar ─────────────────────────────────────────────────

    /// Build a fully styled progress bar from a class list.
    ///
    /// ```rust,ignore
    /// t.progress(0.0..=100.0, pct, &["progress", "success"])
    /// ```
    pub fn progress<'a>(
        &'a self,
        range: RangeInclusive<f32>,
        value: f32,
        classes: &'a [&'a str],
    ) -> progress_bar::ProgressBar<'a, IcedTheme> {
        progress_bar(range, value).style(self.progress_bar(classes))
    }

    // ── Pick List (Select) ───────────────────────────────────────────

    /// Build a fully styled pick list (dropdown) from a class list.
    ///
    /// ```rust,ignore
    /// t.select(&options, state.selected, Msg::Selected, &["select", "sz-md"])
    /// ```
    ///
    /// Automatically derives menu classes from the input classes (replaces
    /// `"select"` with `"select-menu"`).
    pub fn select<'a, T, L, V, Message>(
        &'a self,
        options: L,
        selected: Option<V>,
        on_select: impl Fn(T) -> Message + 'a,
        classes: &'a [&'a str],
    ) -> pick_list::PickList<'a, T, L, V, Message, IcedTheme>
    where
        T: ToString + PartialEq + Clone + 'a,
        L: Borrow<[T]> + 'a,
        V: Borrow<T> + 'a,
        Message: Clone + 'a,
    {
        let sz = self.sizing(classes);

        // Build menu classes: replace "select" with "select-menu".
        let menu_classes: Vec<&str> = classes
            .iter()
            .map(|c| if *c == "select" { "select-menu" } else { *c })
            .collect();

        // Pre-resolve menu style to avoid lifetime issues with the local vec.
        let menu_computed = self.resolve(&menu_classes, None);
        let menu_style = super::menu::build_menu_style(&menu_computed);

        pick_list(options, selected, on_select)
            .text_size(sz.font_size)
            .padding([sz.pad_v, sz.pad_h])
            .style(self.pick_list(classes))
            .menu_style(move |_: &IcedTheme| menu_style)
    }

    // ── Text Editor ──────────────────────────────────────────────────

    /// Build a fully styled text editor as an `Element`.
    ///
    /// ```rust,ignore
    /// t.editor(&state.editor_content, Msg::EditorAction, &["editor", "sz-md"])
    /// ```
    ///
    /// Returns `Element` because `TextEditor` has a `Highlighter` generic that
    /// requires `iced_core` types not re-exported by `iced`. The `on_action`
    /// callback is required here since we can't chain on an `Element`.
    pub fn editor<'a, Message: Clone + 'a>(
        &'a self,
        content: &'a text_editor::Content,
        on_action: impl Fn(text_editor::Action) -> Message + 'a,
        classes: &'a [&'a str],
    ) -> Element<'a, Message, IcedTheme> {
        let sz = self.sizing(classes);

        text_editor(content)
            .on_action(on_action)
            .size(sz.font_size)
            .padding(sz.padding())
            .style(self.text_editor(classes))
            .into()
    }

    // ── Rule (Divider) ───────────────────────────────────────────────

    /// Build a styled horizontal divider from a class list.
    ///
    /// ```rust,ignore
    /// t.divider(1.0, &["divider"])
    /// ```
    pub fn divider<'a>(&'a self, height: f32, classes: &'a [&'a str]) -> rule::Rule<'a, IcedTheme> {
        rule::horizontal(height).style(self.rule(classes))
    }
}
