use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use iced::widget::{
    Row, button, checkbox, column, combo_box, container, mouse_area, pick_list, progress_bar,
    radio, row, rule, scrollable, slider, text, text_editor, text_input, toggler, tooltip,
};
use iced::window;
use iced::{Element, Font, Length, Padding, Point, Size, Task, Theme as IcedTheme};
use icss::theme::Theme;
use icss::widgets::animation::Animation;
use icss::widgets::data_table::{DataColumn, DataTable, SortDirection, SortState};
use icss::widgets::tab_bar::{Tab, TabBar, TabBarAction, TabBarStyle, TabDragState, TabId};
use icss::widgets::tile_grid::{TileGrid, TileLayout};

mod color_picker;
mod generate;
mod persist;

/// Loaded before iced starts so we can set the font.
static INITIAL_VARS: OnceLock<ThemeVars> = OnceLock::new();

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let vars = persist::load().unwrap_or_default();
    let font = vars.font_family.to_iced_font();
    let _ = INITIAL_VARS.set(vars);

    iced::daemon(boot, update, view)
        .title(title)
        .subscription(subscription)
        .theme(theme)
        .default_font(font)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .run()
}

fn title(state: &State, window_id: window::Id) -> String {
    if window_id == state.main_window_id {
        "ICSS Theme Showcase".into()
    } else {
        state
            .windows
            .get(&window_id)
            .and_then(|ws| ws.tabs.iter().find(|t| t.id == ws.active_tab))
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "Showcase".into())
    }
}

fn subscription(state: &State) -> iced::Subscription<Msg> {
    let any_running = state.anim_fade.is_running()
        || state.anim_slide_left.is_running()
        || state.anim_slide_top.is_running()
        || state.anim_slide_right.is_running()
        || state.anim_slide_bottom.is_running();

    let mut subs: Vec<iced::Subscription<Msg>> = vec![
        window::close_requests().map(Msg::WindowCloseRequested),
        // Real-time Moved events — fires on every frame during OS drag.
        // This replaces 100ms polling for responsive merge detection.
        window::events().map(|(id, ev)| match ev {
            iced::window::Event::Moved(pt) => Msg::WindowMoved(id, pt),
            iced::window::Event::Resized(sz) => Msg::WindowResized(id, sz),
            _ => Msg::Noop,
        }),
    ];

    if any_running {
        subs.push(
            iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| Msg::AnimTick(std::time::Instant::now())),
        );
    }

    // Also keep low-frequency polling as a safety net for the initial
    // position (needed because the Moved event only fires on CHANGE —
    // a freshly opened window at a fixed position emits nothing).
    if !state.windows.is_empty() {
        subs.push(
            iced::time::every(std::time::Duration::from_millis(250)).map(|_| Msg::PollPositions),
        );
    }

    iced::Subscription::batch(subs)
}

fn theme(_state: &State, _window_id: window::Id) -> IcedTheme {
    IcedTheme::Dark
}

// ── State ──

/// Per-window tab state.
struct WindowState {
    tabs: Vec<Tab>,
    active_tab: TabId,
    tab_drag: TabDragState,
    /// Offset of the cursor within this window at the moment it was detached
    /// — used to compute screen-space cursor position during post-detach
    /// window::drag() (which no longer fires widget-level DragMove events).
    /// None for windows that were never detached.
    grab_offset: Option<(f32, f32)>,
}

struct State {
    main_window_id: window::Id,
    windows: HashMap<window::Id, WindowState>,
    next_tab_id: TabId,
    theme: Theme,
    sidebar_theme: Theme,
    dims: icss::engine::dims::DimTokens,
    neutral_palette: Vec<[f32; 3]>,
    family_steps: Vec<(&'static str, icss::engine::semantic::SurfaceSteps)>,
    vars: ThemeVars,
    // Color picker state
    active_color: ColorField,
    picker_hue: f32,
    picker_sat: f32,
    picker_val: f32,
    // Widget state
    text_value: String,
    error_value: String,
    editor_content: iced::widget::text_editor::Content,
    combo_state: combo_box::State<String>,
    combo_value: Option<String>,
    slider_value: f32,
    check_a: bool,
    check_b: bool,
    check_c: bool,
    toggle_a: bool,
    toggle_b: bool,
    radio_choice: Option<RadioOpt>,
    pick_choice: Option<String>,
    // Tile grid state
    tile_selected: HashSet<usize>,
    tile_layout: TileLayout,
    // Button group state
    btn_group_active: usize,
    // Button demo state
    buttons_disabled: bool,
    gradient_hover: Option<usize>, // which emphasized button is hovered
    gradient_pressed: Option<usize>,
    // Animation demos
    anim_fade: icss::widgets::Animation,
    anim_slide_left: icss::widgets::Animation,
    anim_slide_top: icss::widgets::Animation,
    anim_slide_right: icss::widgets::Animation,
    anim_slide_bottom: icss::widgets::Animation,
    // Data table state
    dt_contacts: Vec<Contact>,
    dt_filtered: Vec<Contact>,
    dt_selected: HashSet<usize>,
    dt_sort: Option<SortState>,
    dt_page: usize,
    dt_page_size: usize,
    dt_search: String,
    // Chat textarea state
    chat_textarea_content: iced::widget::text_editor::Content,
    // Scroll tracking for sticky table header
    page_scroll_y: f32,
    // Window position tracking for tab merge
    window_positions: HashMap<window::Id, Point>,
    window_sizes: HashMap<window::Id, Size>,
    /// Set on each WindowMoved — merge check fires after window stops moving.
    merge_pending: Option<(window::Id, std::time::Instant)>,
    /// Target window whose tab bar should show the red drop highlight.
    merge_highlight: Option<window::Id>,
    /// When true, theme edits persist `theme-{dark,light}.icss` +
    /// `showcase-vars.conf` to disk. Unchecking lets the user experiment
    /// with the visuals without modifying any files. Toggled from the tab bar.
    save_icss: bool,
}

/// Theme variables editable in the sidebar.
#[derive(Clone)]
struct ThemeVars {
    primary: String,
    secondary: String,
    tertiary: String,
    quaternary: String,
    neutral: String,
    link: String,
    increment: f32,
    font_increment: f32,
    radius_factor: f32,
    font_family: FontFamily,
    dark_mode: bool,
    surface_lightness: f32,
    gamma: f32,
    text_spread: f32,
    // Per-mode stored values (restored on dark/light toggle)
    dark_surface_lightness: f32,
    dark_gamma: f32,
    dark_text_spread: f32,
    light_surface_lightness: f32,
    light_gamma: f32,
    light_text_spread: f32,
    // Manual signal-color overrides. Empty `String` → fall back to the
    // auto-derived value (re-computed from the 4 chromatic colors via
    // `derive_signals`).
    success_override: String,
    danger_override: String,
    warning_override: String,
    // Resolved signal colors (override or derived). Read-only; updated
    // on every theme regeneration so swatches reflect what's actually in
    // use.
    derived_success: String,
    derived_danger: String,
    derived_warning: String,
}

impl Default for ThemeVars {
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
            // Default to the platform's system font — guarantees every
            // weight (Light/Regular/Medium/Semibold/Bold) resolves within
            // the same typeface. Roboto, in particular, ships only
            // Regular on many macOS installs, so Bold falls back to a
            // different family and the weight sample looks mismatched.
            #[cfg(target_os = "macos")]
            font_family: FontFamily::SFPro,
            #[cfg(target_os = "windows")]
            font_family: FontFamily::SegoeUI,
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            font_family: FontFamily::Roboto,
            dark_mode: true,
            surface_lightness: 5.0,
            gamma: 1.0,
            text_spread: 1.0,
            dark_surface_lightness: 5.0,
            dark_gamma: 1.0,
            dark_text_spread: 1.0,
            light_surface_lightness: 95.0,
            light_gamma: 1.0,
            light_text_spread: 1.0,
            success_override: String::new(),
            danger_override: String::new(),
            warning_override: String::new(),
            derived_success: String::new(),
            derived_danger: String::new(),
            derived_warning: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FontFamily {
    SFPro,
    SegoeUI,
    Roboto,
}

impl FontFamily {
    pub fn to_iced_font(self) -> Font {
        match self {
            Self::SFPro => Font::with_name("SF Pro"),
            Self::SegoeUI => Font::with_name("Segoe UI"),
            Self::Roboto => Font::with_name("Roboto"),
        }
    }

    /// Return the family with a specific weight applied. Keeps the family
    /// base so Bold/Semibold/Light all resolve within the chosen typeface
    /// instead of falling back to iced's Font::DEFAULT (which otherwise
    /// substitutes a mono fallback for weights not present in Fira Sans).
    pub fn weighted(self, weight: iced::font::Weight) -> Font {
        Font {
            weight,
            ..self.to_iced_font()
        }
    }
}

impl std::fmt::Display for FontFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SFPro => write!(f, "SF Pro (macOS)"),
            Self::SegoeUI => write!(f, "Segoe UI (Windows)"),
            Self::Roboto => write!(f, "Roboto (Android)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorField {
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
    Neutral,
    Link,
    /// Signal overrides — empty value means "auto-derive from P/S/T/Q".
    Success,
    Danger,
    Warning,
}

impl ColorField {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Tertiary => "Tertiary",
            Self::Quaternary => "Quaternary",
            Self::Neutral => "Neutral",
            Self::Link => "Link",
            Self::Success => "Success",
            Self::Danger => "Danger",
            Self::Warning => "Warning",
        }
    }

    /// Effective value used in the picker. For signal fields, falls back
    /// to the resolved (derived) color when the override is empty so the
    /// swatch shows the in-use color rather than a black void.
    fn get(self, vars: &ThemeVars) -> &str {
        match self {
            Self::Primary => &vars.primary,
            Self::Secondary => &vars.secondary,
            Self::Tertiary => &vars.tertiary,
            Self::Quaternary => &vars.quaternary,
            Self::Neutral => &vars.neutral,
            Self::Link => &vars.link,
            Self::Success => {
                if vars.success_override.is_empty() {
                    &vars.derived_success
                } else {
                    &vars.success_override
                }
            }
            Self::Danger => {
                if vars.danger_override.is_empty() {
                    &vars.derived_danger
                } else {
                    &vars.danger_override
                }
            }
            Self::Warning => {
                if vars.warning_override.is_empty() {
                    &vars.derived_warning
                } else {
                    &vars.warning_override
                }
            }
        }
    }

    fn set(self, vars: &mut ThemeVars, val: String) {
        match self {
            Self::Primary => vars.primary = val,
            Self::Secondary => vars.secondary = val,
            Self::Tertiary => vars.tertiary = val,
            Self::Quaternary => vars.quaternary = val,
            Self::Neutral => vars.neutral = val,
            Self::Link => vars.link = val,
            Self::Success => vars.success_override = val,
            Self::Danger => vars.danger_override = val,
            Self::Warning => vars.warning_override = val,
        }
    }

    /// Whether this field has a manual override that can be cleared. Used
    /// by the UI to show a Reset button only on signal rows.
    fn is_signal(self) -> bool {
        matches!(self, Self::Success | Self::Danger | Self::Warning)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RadioOpt {
    Alpha,
    Beta,
    Gamma,
}

impl std::fmt::Display for RadioOpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alpha => write!(f, "Alpha"),
            Self::Beta => write!(f, "Beta"),
            Self::Gamma => write!(f, "Gamma"),
        }
    }
}

#[derive(Debug, Clone)]
struct Contact {
    name: String,
    detail: String,
    email: String,
    role: String,
    status: String,
    location: String,
}

fn demo_contacts() -> Vec<Contact> {
    vec![
        Contact {
            name: "Alice Chen".into(),
            detail: "Senior Backend Engineer\nRust, Go, Distributed Systems".into(),
            email: "alice@example.com".into(),
            role: "Engineer".into(),
            status: "Online".into(),
            location: "San Francisco".into(),
        },
        Contact {
            name: "Bob Martinez".into(),
            detail: "".into(),
            email: "bob@example.com".into(),
            role: "Designer".into(),
            status: "Away".into(),
            location: "New York".into(),
        },
        Contact {
            name: "Carol Williams".into(),
            detail: "Product Lead\nMobile & Desktop Clients".into(),
            email: "carol@example.com".into(),
            role: "PM".into(),
            status: "Online".into(),
            location: "London".into(),
        },
        Contact {
            name: "David Kim".into(),
            detail: "".into(),
            email: "david@example.com".into(),
            role: "Engineer".into(),
            status: "Offline".into(),
            location: "Seoul".into(),
        },
        Contact {
            name: "Elena Popov".into(),
            detail: "Infrastructure & CI/CD\nKubernetes, Terraform".into(),
            email: "elena@example.com".into(),
            role: "DevOps".into(),
            status: "Online".into(),
            location: "Berlin".into(),
        },
        Contact {
            name: "Frank Tanaka".into(),
            detail: "".into(),
            email: "frank@example.com".into(),
            role: "Engineer".into(),
            status: "Online".into(),
            location: "Tokyo".into(),
        },
        Contact {
            name: "Grace Liu".into(),
            detail: "Design Systems Lead\nFigma, Component Libraries".into(),
            email: "grace@example.com".into(),
            role: "Designer".into(),
            status: "Away".into(),
            location: "Shanghai".into(),
        },
        Contact {
            name: "Hector Ruiz".into(),
            detail: "".into(),
            email: "hector@example.com".into(),
            role: "QA".into(),
            status: "Online".into(),
            location: "Madrid".into(),
        },
        Contact {
            name: "Irene Costa".into(),
            detail: "Networking & Protocol\nQUIC, WebRTC, Transport".into(),
            email: "irene@example.com".into(),
            role: "Engineer".into(),
            status: "Offline".into(),
            location: "Lisbon".into(),
        },
        Contact {
            name: "James O'Brien".into(),
            detail: "".into(),
            email: "james@example.com".into(),
            role: "PM".into(),
            status: "Online".into(),
            location: "Dublin".into(),
        },
        Contact {
            name: "Kira Patel".into(),
            detail: "Video Codec Engineer\nH.264, VP9, Hardware Encode".into(),
            email: "kira@example.com".into(),
            role: "Engineer".into(),
            status: "Online".into(),
            location: "Mumbai".into(),
        },
        Contact {
            name: "Liam Foster".into(),
            detail: "".into(),
            email: "liam@example.com".into(),
            role: "DevOps".into(),
            status: "Away".into(),
            location: "Sydney".into(),
        },
    ]
}

#[derive(Debug, Clone)]
enum Msg {
    // Sidebar variable edits
    PrimaryChanged(String),
    SecondaryChanged(String),
    TertiaryChanged(String),
    QuaternaryChanged(String),
    NeutralChanged(String),
    LinkChanged(String),
    /// Manual signal-color overrides. Empty value clears the override
    /// and falls back to the auto-derived colour.
    SuccessChanged(String),
    DangerChanged(String),
    WarningChanged(String),
    /// Reset a signal field to its auto-derived value.
    SignalReset(ColorField),
    IncrementChanged(f32),
    FontIncrementChanged(f32),
    RadiusFactorChanged(f32),
    FontFamilySelected(FontFamily),
    DarkModeToggled(bool),
    /// Tab-bar toggle: persist theme edits to disk vs. preview-only.
    SaveIcssToggled(bool),
    SurfaceLightnessChanged(f32),
    GammaChanged(f32),
    TextSpreadChanged(f32),
    RestartApp,
    // Color picker
    SelectColorField(ColorField),
    PickerHueChanged(f32),
    PickerSvChanged(f32, f32),
    // Widget interactions
    TextChanged(String),
    ErrorTextChanged(String),
    SliderChanged(f32),
    CheckA(bool),
    CheckB(bool),
    CheckC(bool),
    ToggleA(bool),
    ToggleB(bool),
    RadioSelected(RadioOpt),
    PickSelected(String),
    ComboSelected(String),
    EditorAction(iced::widget::text_editor::Action),
    ChatTextareaAction(iced::widget::text_editor::Action),
    ChatSend,
    ChatAttach,
    // Tile grid
    TilePressed(usize),
    TileLayoutChanged(String),
    // Data table
    DtRowPressed(usize),
    DtSort(String),
    DtSelect(usize),
    DtSelectAll,
    DtPageChanged(usize),
    DtPageSizeChanged(usize),
    DtSearchChanged(String),
    // Button demo
    BtnGroupChanged(usize),
    ButtonsDisabledToggle(bool),
    GradientEnter(usize),
    GradientExit(usize),
    GradientPress(usize),
    GradientRelease(usize),
    // Animations
    AnimTick(std::time::Instant),
    AnimFadeToggle,
    AnimSlide(icss::widgets::Edge),
    // Tab bar (per-window)
    TabAction(window::Id, TabBarAction),
    // macOS titlebar reshape (delayed after window creation)
    #[cfg(target_os = "macos")]
    ReshapeTitlebar(String, u8),
    // Window lifecycle
    WindowCloseRequested(window::Id),
    WindowMoved(window::Id, Point),
    WindowResized(window::Id, Size),
    PollPositions,
    GotPosition(window::Id, Option<Point>),
    Noop,
}

fn boot() -> (State, Task<Msg>) {
    let mut vars = INITIAL_VARS.get().cloned().unwrap_or_default();
    let output = generate::generate_icss(&mut vars, true);
    // Make a default run produce a complete on-disk snapshot — vars + the
    // dark/light .icss artifacts. Without this, `cargo run -p icss-showcase`
    // followed by an immediate quit (no edits) wouldn't leave anything for
    // `rl-desktop`'s build.rs to pick up.
    persist::save(&vars);
    let theme = Theme::load(&output.icss).unwrap();
    let dims = output.dims;
    let neutral_palette = output.neutral_palette;
    let family_steps = output.family_steps;
    let sidebar_theme = Theme::load(generate::SIDEBAR_THEME).unwrap();

    let hsv = color_picker::HsvColor::from_hex(&vars.primary).unwrap_or(color_picker::HsvColor {
        h: 0.0,
        s: 0.8,
        v: 0.9,
    });

    // Chrome-style tabs-in-titlebar: fullsize_content_view extends content
    // behind the transparent titlebar so the tab bar IS the titlebar area.
    // Traffic lights float over the left_pad region of the tab bar.
    let mut settings = window::Settings {
        size: Size::new(1400.0, 900.0),
        min_size: Some(Size::new(800.0, 500.0)),
        decorations: true,
        ..Default::default()
    };
    #[cfg(target_os = "macos")]
    {
        settings.platform_specific.title_hidden = true;
        settings.platform_specific.titlebar_transparent = true;
        settings.platform_specific.fullsize_content_view = true;
    }
    #[cfg(not(target_os = "macos"))]
    {
        settings.decorations = false;
    }
    let (main_id, open_task) = window::open(settings);

    let main_tabs = WindowState {
        tabs: vec![
            Tab {
                id: 0,
                title: "ICSS Controls".into(),
                closable: true,
                icon: None,
            },
            Tab {
                id: 1,
                title: "Primitives".into(),
                closable: true,
                icon: None,
            },
        ],
        active_tab: 0,
        tab_drag: TabDragState::new(),
        grab_offset: None,
    };

    (
        State {
            main_window_id: main_id,
            windows: HashMap::from([(main_id, main_tabs)]),
            next_tab_id: 2,
            theme,
            sidebar_theme,
            dims,
            neutral_palette,
            family_steps,
            vars,
            active_color: ColorField::Primary,
            picker_hue: hsv.h,
            picker_sat: hsv.s,
            picker_val: hsv.v,
            text_value: String::new(),
            error_value: "invalid input".into(),
            editor_content: iced::widget::text_editor::Content::with_text(
                "Multi-line text editor.\nEdit this text to test styling.",
            ),
            chat_textarea_content: iced::widget::text_editor::Content::new(),
            combo_state: combo_box::State::new(vec![
                "Rust".into(),
                "Python".into(),
                "TypeScript".into(),
                "Go".into(),
                "C++".into(),
                "Swift".into(),
                "Kotlin".into(),
            ]),
            combo_value: None,
            slider_value: 0.4,
            check_a: true,
            check_b: false,
            check_c: true,
            toggle_a: true,
            toggle_b: false,
            radio_choice: Some(RadioOpt::Alpha),
            pick_choice: None,
            // Tile grid
            tile_selected: HashSet::new(),
            tile_layout: TileLayout::Flow {
                min_tile_width: 200.0,
            },
            // Data table
            dt_contacts: demo_contacts(),
            dt_filtered: demo_contacts(),
            dt_selected: HashSet::new(),
            dt_sort: None,
            dt_page: 0,
            dt_page_size: 10,
            dt_search: String::new(),
            page_scroll_y: 0.0,
            window_positions: HashMap::new(),
            window_sizes: HashMap::from([(main_id, Size::new(1400.0, 900.0))]),
            merge_pending: None,
            merge_highlight: None,
            btn_group_active: 0,
            buttons_disabled: false,
            gradient_hover: None,
            gradient_pressed: None,
            anim_fade: icss::widgets::Animation::new(),
            anim_slide_left: icss::widgets::Animation::new(),
            anim_slide_top: icss::widgets::Animation::new(),
            anim_slide_right: icss::widgets::Animation::new(),
            anim_slide_bottom: icss::widgets::Animation::new(),
            save_icss: true,
        },
        {
            let mut tasks: Vec<Task<Msg>> = vec![open_task.discard()];
            #[cfg(target_os = "macos")]
            tasks.push(Task::perform(
                async { tokio::time::sleep(std::time::Duration::from_millis(200)).await },
                |_| Msg::ReshapeTitlebar("ICSS Theme Showcase".into(), 5),
            ));
            Task::batch(tasks)
        },
    )
}

fn refilter_contacts(state: &mut State) {
    let q = state.dt_search.to_lowercase();
    state.dt_filtered = state
        .dt_contacts
        .iter()
        .filter(|c| {
            if q.is_empty() {
                return true;
            }
            c.name.to_lowercase().contains(&q)
                || c.detail.to_lowercase().contains(&q)
                || c.email.to_lowercase().contains(&q)
                || c.role.to_lowercase().contains(&q)
                || c.status.to_lowercase().contains(&q)
                || c.location.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    state.dt_selected.clear();
}

fn sync_picker_from_field(state: &mut State) {
    let hex = state.active_color.get(&state.vars);
    if let Some(hsv) = color_picker::HsvColor::from_hex(hex) {
        state.picker_hue = hsv.h;
        state.picker_sat = hsv.s;
        state.picker_val = hsv.v;
    }
}

fn update(state: &mut State, msg: Msg) -> Task<Msg> {
    let mut regen = false;
    match msg {
        // Sidebar hex inputs
        Msg::PrimaryChanged(v) => {
            state.vars.primary = v;
            if state.active_color == ColorField::Primary {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::SecondaryChanged(v) => {
            state.vars.secondary = v;
            if state.active_color == ColorField::Secondary {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::TertiaryChanged(v) => {
            state.vars.tertiary = v;
            if state.active_color == ColorField::Tertiary {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::QuaternaryChanged(v) => {
            state.vars.quaternary = v;
            if state.active_color == ColorField::Quaternary {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::NeutralChanged(v) => {
            state.vars.neutral = v;
            if state.active_color == ColorField::Neutral {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::LinkChanged(v) => {
            state.vars.link = v;
            if state.active_color == ColorField::Link {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::SuccessChanged(v) => {
            state.vars.success_override = v;
            if state.active_color == ColorField::Success {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::DangerChanged(v) => {
            state.vars.danger_override = v;
            if state.active_color == ColorField::Danger {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::WarningChanged(v) => {
            state.vars.warning_override = v;
            if state.active_color == ColorField::Warning {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::SignalReset(f) => {
            f.set(&mut state.vars, String::new());
            if state.active_color == f {
                sync_picker_from_field(state);
            }
            regen = true;
        }
        Msg::IncrementChanged(v) => {
            state.vars.increment = v;
            regen = true;
        }
        Msg::FontIncrementChanged(v) => {
            state.vars.font_increment = v;
            regen = true;
        }
        Msg::RadiusFactorChanged(v) => {
            state.vars.radius_factor = v;
            regen = true;
        }
        Msg::FontFamilySelected(f) => {
            state.vars.font_family = f;
            regen = true;
        }
        Msg::DarkModeToggled(v) => {
            // Save current per-mode values before switching
            if state.vars.dark_mode {
                state.vars.dark_surface_lightness = state.vars.surface_lightness;
                state.vars.dark_gamma = state.vars.gamma;
                state.vars.dark_text_spread = state.vars.text_spread;
            } else {
                state.vars.light_surface_lightness = state.vars.surface_lightness;
                state.vars.light_gamma = state.vars.gamma;
                state.vars.light_text_spread = state.vars.text_spread;
            }
            state.vars.dark_mode = v;
            // Restore the target mode's values
            if v {
                state.vars.surface_lightness = state.vars.dark_surface_lightness;
                state.vars.gamma = state.vars.dark_gamma;
                state.vars.text_spread = state.vars.dark_text_spread;
            } else {
                state.vars.surface_lightness = state.vars.light_surface_lightness;
                state.vars.gamma = state.vars.light_gamma;
                state.vars.text_spread = state.vars.light_text_spread;
            }
            regen = true;
        }
        Msg::SaveIcssToggled(v) => {
            state.save_icss = v;
            // Re-checking captures the current visuals to disk immediately,
            // so the toggle reflects the on-disk state going forward.
            if v {
                generate::generate_icss(&mut state.vars, true);
                persist::save(&state.vars);
            }
        }
        Msg::SurfaceLightnessChanged(v) => {
            state.vars.surface_lightness = v;
            regen = true;
        }
        Msg::GammaChanged(v) => {
            state.vars.gamma = v;
            regen = true;
        }
        Msg::TextSpreadChanged(v) => {
            state.vars.text_spread = v;
            regen = true;
        }
        Msg::RestartApp => {
            if state.save_icss {
                persist::save(&state.vars);
            }
            std::process::exit(0);
        }
        // Color picker
        Msg::SelectColorField(field) => {
            state.active_color = field;
            sync_picker_from_field(state);
        }
        Msg::PickerHueChanged(h) => {
            state.picker_hue = h;
            let hex = color_picker::HsvColor {
                h,
                s: state.picker_sat,
                v: state.picker_val,
            }
            .to_hex();
            state.active_color.set(&mut state.vars, hex);
            regen = true;
        }
        Msg::PickerSvChanged(s, v) => {
            state.picker_sat = s;
            state.picker_val = v;
            let hex = color_picker::HsvColor {
                h: state.picker_hue,
                s,
                v,
            }
            .to_hex();
            state.active_color.set(&mut state.vars, hex);
            regen = true;
        }
        // Widgets
        Msg::TextChanged(v) => state.text_value = v,
        Msg::ErrorTextChanged(v) => state.error_value = v,
        Msg::SliderChanged(v) => state.slider_value = v,
        Msg::CheckA(v) => state.check_a = v,
        Msg::CheckB(v) => state.check_b = v,
        Msg::CheckC(v) => state.check_c = v,
        Msg::ToggleA(v) => state.toggle_a = v,
        Msg::ToggleB(v) => state.toggle_b = v,
        Msg::RadioSelected(v) => state.radio_choice = Some(v),
        Msg::PickSelected(v) => state.pick_choice = Some(v),
        Msg::ComboSelected(v) => {
            state.combo_value = Some(v);
        }
        Msg::EditorAction(action) => {
            state.editor_content.perform(action);
        }
        Msg::ChatTextareaAction(action) => {
            state.chat_textarea_content.perform(action);
        }
        Msg::ChatSend => {
            state.chat_textarea_content = iced::widget::text_editor::Content::new();
        }
        Msg::ChatAttach => {} // demo only
        // Tile grid
        Msg::TilePressed(i) => {
            if state.tile_selected.contains(&i) {
                state.tile_selected.remove(&i);
            } else {
                state.tile_selected.insert(i);
            }
        }
        Msg::TileLayoutChanged(mode) => {
            state.tile_layout = match mode.as_str() {
                "Horizontal" => TileLayout::Horizontal,
                "Vertical" => TileLayout::Vertical,
                _ => TileLayout::Flow {
                    min_tile_width: 200.0,
                },
            };
        }
        // Data table
        Msg::DtRowPressed(i) => {
            if state.dt_selected.contains(&i) {
                state.dt_selected.remove(&i);
            } else {
                state.dt_selected.insert(i);
            }
        }
        Msg::DtSort(key) => {
            state.dt_sort = Some(match &state.dt_sort {
                Some(s) if s.key == key => SortState {
                    key,
                    direction: s.direction.toggle(),
                },
                _ => SortState {
                    key,
                    direction: SortDirection::Ascending,
                },
            });
            // Sort the contacts
            if let Some(ref sort) = state.dt_sort {
                let asc = sort.direction == SortDirection::Ascending;
                match sort.key.as_str() {
                    "name" => state.dt_contacts.sort_by(|a, b| {
                        if asc {
                            a.name.cmp(&b.name)
                        } else {
                            b.name.cmp(&a.name)
                        }
                    }),
                    "email" => state.dt_contacts.sort_by(|a, b| {
                        if asc {
                            a.email.cmp(&b.email)
                        } else {
                            b.email.cmp(&a.email)
                        }
                    }),
                    "role" => state.dt_contacts.sort_by(|a, b| {
                        if asc {
                            a.role.cmp(&b.role)
                        } else {
                            b.role.cmp(&a.role)
                        }
                    }),
                    "status" => state.dt_contacts.sort_by(|a, b| {
                        if asc {
                            a.status.cmp(&b.status)
                        } else {
                            b.status.cmp(&a.status)
                        }
                    }),
                    "location" => state.dt_contacts.sort_by(|a, b| {
                        if asc {
                            a.location.cmp(&b.location)
                        } else {
                            b.location.cmp(&a.location)
                        }
                    }),
                    _ => {}
                }
            }
            refilter_contacts(state);
        }
        Msg::DtSelect(i) => {
            if state.dt_selected.contains(&i) {
                state.dt_selected.remove(&i);
            } else {
                state.dt_selected.insert(i);
            }
        }
        Msg::DtSelectAll => {
            let total = state.dt_contacts.len();
            if state.dt_selected.len() == total {
                state.dt_selected.clear();
            } else {
                state.dt_selected = (0..total).collect();
            }
        }
        Msg::DtPageChanged(p) => state.dt_page = p,
        Msg::DtPageSizeChanged(s) => {
            state.dt_page_size = s;
            state.dt_page = 0;
        }
        Msg::DtSearchChanged(q) => {
            state.dt_search = q;
            refilter_contacts(state);
        }
        Msg::BtnGroupChanged(i) => {
            state.btn_group_active = i;
        }
        Msg::ButtonsDisabledToggle(v) => {
            state.buttons_disabled = v;
        }
        Msg::GradientEnter(i) => {
            state.gradient_hover = Some(i);
        }
        Msg::GradientExit(i) => {
            if state.gradient_hover == Some(i) {
                state.gradient_hover = None;
            }
            state.gradient_pressed = None;
        }
        Msg::GradientPress(i) => {
            state.gradient_pressed = Some(i);
        }
        Msg::GradientRelease(_) => {
            state.gradient_pressed = None;
        }
        Msg::AnimTick(now) => {
            state.anim_fade.tick(now);
            state.anim_slide_left.tick(now);
            state.anim_slide_top.tick(now);
            state.anim_slide_right.tick(now);
            state.anim_slide_bottom.tick(now);
        }
        Msg::AnimFadeToggle => {
            use std::time::Duration;
            let dur = Duration::from_millis(500);
            if state.anim_fade.is_idle() || state.anim_fade.is_done() {
                // Toggle direction
                let kind = if state.anim_fade.value() > 0.5 {
                    icss::widgets::AnimKind::FadeOut
                } else {
                    icss::widgets::AnimKind::FadeIn
                };
                state.anim_fade.start(kind, dur);
            }
        }
        Msg::AnimSlide(edge) => {
            use std::time::Duration;
            let dur = Duration::from_millis(500);
            let anim = match edge {
                icss::widgets::Edge::Left => &mut state.anim_slide_left,
                icss::widgets::Edge::Top => &mut state.anim_slide_top,
                icss::widgets::Edge::Right => &mut state.anim_slide_right,
                icss::widgets::Edge::Bottom => &mut state.anim_slide_bottom,
            };
            if anim.is_idle() || anim.is_done() {
                let kind = if anim.value() > 0.5 {
                    icss::widgets::AnimKind::SlideOut(edge)
                } else {
                    icss::widgets::AnimKind::SlideIn(edge)
                };
                anim.start(kind, dur);
            }
        }
        // macOS: reshape the titlebar to only cover traffic lights
        #[cfg(target_os = "macos")]
        Msg::ReshapeTitlebar(title, retries) => {
            unsafe extern "C" {
                fn rl_showcase_configure_titlebar(
                    title_utf8: *const std::ffi::c_char,
                    bar_height: f32,
                    bg_r: f32,
                    bg_g: f32,
                    bg_b: f32,
                ) -> i32;
            }
            let bg = state
                .theme
                .color_var("surface-s0")
                .unwrap_or(iced::Color::from_rgb(0.05, 0.05, 0.06));
            let c_title = std::ffi::CString::new(title.clone()).unwrap();
            let ok =
                unsafe { rl_showcase_configure_titlebar(c_title.as_ptr(), 36.0, bg.r, bg.g, bg.b) };
            if ok == 0 && retries > 0 {
                return Task::perform(
                    async { tokio::time::sleep(std::time::Duration::from_millis(200)).await },
                    move |_| Msg::ReshapeTitlebar(title, retries - 1),
                );
            }
        }
        // Tab bar (per-window)
        Msg::TabAction(wid, action) => {
            // Helper: apply regen if needed before returning a task
            macro_rules! maybe_regen {
                () => {
                    if regen {
                        let output = generate::generate_icss(&mut state.vars, state.save_icss);
                        if let Ok(new_theme) = Theme::load(&output.icss) {
                            state.theme = new_theme;
                        }
                        state.dims = output.dims;
                        state.neutral_palette = output.neutral_palette;
                        state.family_steps = output.family_steps;
                        if state.save_icss {
                            persist::save(&state.vars);
                        }
                        #[allow(unused_assignments)]
                        {
                            regen = false;
                        }
                    }
                };
            }

            if let Some(ws) = state.windows.get_mut(&wid) {
                match action {
                    TabBarAction::Select(id) => {
                        ws.active_tab = id;
                    }
                    TabBarAction::Close(id) => {
                        ws.tabs.retain(|t| t.id != id);
                        if ws.active_tab == id {
                            ws.active_tab = ws.tabs.first().map(|t| t.id).unwrap_or(0);
                        }
                        // If this window has no tabs left and isn't main, close it
                        if ws.tabs.is_empty() && wid != state.main_window_id {
                            state.windows.remove(&wid);
                            maybe_regen!();
                            return window::close(wid);
                        }
                    }
                    TabBarAction::New => {
                        let id = state.next_tab_id;
                        state.next_tab_id += 1;
                        ws.tabs.push(Tab {
                            id,
                            title: format!("Tab {}", id),
                            closable: true,
                            icon: None,
                        });
                        ws.active_tab = id;
                    }
                    TabBarAction::DragStart { tab, x, y } => {
                        if ws.tabs.len() <= 1 {
                            // Single tab — drag the whole window immediately
                            return window::drag(wid);
                        }
                        ws.tab_drag.dragging = Some(tab);
                        ws.tab_drag.drag_start_x = x;
                        ws.tab_drag.drag_start_y = y;
                        ws.tab_drag.drag_current_x = x;
                        ws.tab_drag.drag_current_y = y;
                        ws.tab_drag.detached = false;
                        if let Some(idx) = ws.tabs.iter().position(|t| t.id == tab) {
                            ws.tab_drag.drag_origin_idx = idx;
                        }
                    }
                    TabBarAction::DragMove { x, y } => {
                        ws.tab_drag.drag_current_x = x;
                        ws.tab_drag.drag_current_y = y;

                        let dy = (y - ws.tab_drag.drag_start_y).abs();
                        if dy > 20.0
                            && let Some(tab_id) = ws.tab_drag.dragging
                            && let Some(tab_data) = ws.tabs.iter().find(|t| t.id == tab_id).cloned()
                        {
                            // Capture grab info BEFORE clearing drag state.
                            let drag_start_x = ws.tab_drag.drag_start_x;
                            let drag_start_y = ws.tab_drag.drag_start_y;
                            let origin_idx = ws.tab_drag.drag_origin_idx;

                            // Remove tab from source window
                            ws.tabs.retain(|t| t.id != tab_id);
                            if ws.active_tab == tab_id {
                                ws.active_tab = ws.tabs.first().map(|t| t.id).unwrap_or(0);
                            }
                            ws.tab_drag = TabDragState::new();

                            // Position new window so cursor stays at the same
                            // offset within the tab as when the drag started.
                            // In source: tab was at slot origin_idx, so the grab's
                            // x-offset within that tab is (drag_start_x - old_tab_x).
                            // In new window: tab is at slot 0, so the new window
                            // should be positioned such that cursor lands at the
                            // same offset within the first-slot tab.
                            //   new_win_x = screen_cursor_x - drag_start_x + origin_idx * tab_w
                            //   new_win_y = screen_cursor_y - drag_start_y
                            // (The left_pad/top_pad constants cancel out.)
                            let win_pos = state
                                .window_positions
                                .get(&wid)
                                .copied()
                                .unwrap_or(Point::ORIGIN);
                            let screen_cursor_x = win_pos.x + x;
                            let screen_cursor_y = win_pos.y + y;
                            let tab_w = 241.0_f32; // tab_max_width (240) + tab_gap (1)
                            let new_win_x =
                                screen_cursor_x - drag_start_x + (origin_idx as f32) * tab_w;
                            let new_win_y = screen_cursor_y - drag_start_y;

                            let mut detach_settings = window::Settings {
                                size: Size::new(900.0, 600.0),
                                position: window::Position::Specific(Point::new(
                                    new_win_x, new_win_y,
                                )),
                                decorations: true,
                                ..Default::default()
                            };
                            #[cfg(target_os = "macos")]
                            {
                                detach_settings.platform_specific.title_hidden = true;
                                detach_settings.platform_specific.titlebar_transparent = true;
                                detach_settings.platform_specific.fullsize_content_view = true;
                            }
                            #[cfg(not(target_os = "macos"))]
                            {
                                detach_settings.decorations = false;
                            }
                            let (new_id, open_task) = window::open(detach_settings);
                            // Cursor inside the NEW window at detach time:
                            // after shifting, the grabbed tab lives at slot 0,
                            // so the x-offset is (drag_start_x - origin_idx*tab_w).
                            // Y-offset stays the same.
                            let grab_x = drag_start_x - (origin_idx as f32) * tab_w;
                            let grab_y = drag_start_y;
                            #[cfg(target_os = "macos")]
                            let detach_title = tab_data.title.clone();
                            state.windows.insert(
                                new_id,
                                WindowState {
                                    tabs: vec![tab_data],
                                    active_tab: tab_id,
                                    tab_drag: TabDragState::new(),
                                    grab_offset: Some((grab_x, grab_y)),
                                },
                            );
                            state.window_sizes.insert(new_id, Size::new(900.0, 600.0));
                            state
                                .window_positions
                                .insert(new_id, Point::new(new_win_x, new_win_y));

                            // Chain: open → OS drag so user seamlessly
                            // continues moving the new window.
                            // Also schedule titlebar reshape for the new window.
                            maybe_regen!();
                            let mut tasks = vec![open_task.discard().chain(window::drag(new_id))];
                            #[cfg(target_os = "macos")]
                            {
                                tasks.push(Task::perform(
                                    async {
                                        tokio::time::sleep(std::time::Duration::from_millis(200))
                                            .await
                                    },
                                    move |_| Msg::ReshapeTitlebar(detach_title, 5),
                                ));
                            }
                            return Task::batch(tasks);
                        }
                    }
                    TabBarAction::DragEnd => {
                        if let Some(dragging_id) = ws.tab_drag.dragging {
                            let dx = ws.tab_drag.drag_current_x - ws.tab_drag.drag_start_x;
                            let tab_w = 241.0;
                            let shift = (dx / tab_w).round() as i32;
                            if let Some(from) = ws.tabs.iter().position(|t| t.id == dragging_id) {
                                let to = (from as i32 + shift).clamp(0, ws.tabs.len() as i32 - 1)
                                    as usize;
                                if from != to {
                                    let tab = ws.tabs.remove(from);
                                    ws.tabs.insert(to, tab);
                                }
                            }
                        }
                        ws.tab_drag = TabDragState::new();
                    }
                    TabBarAction::Detach { .. } => {}
                    TabBarAction::WindowDrag => {
                        return window::drag(wid);
                    }
                    TabBarAction::WindowClose => {
                        state.windows.remove(&wid);
                        state.window_positions.remove(&wid);
                        state.window_sizes.remove(&wid);
                        if wid == state.main_window_id {
                            return window::close(wid).chain(iced::exit());
                        } else {
                            return window::close(wid);
                        }
                    }
                    TabBarAction::WindowMinimize => {
                        return window::minimize(wid, true);
                    }
                    TabBarAction::WindowMaximize => {
                        return window::toggle_maximize(wid);
                    }
                    TabBarAction::Home
                    | TabBarAction::AppMenu
                    | TabBarAction::ToggleTheme
                    | TabBarAction::ConnectionInfo => {}
                }
            }
        }
        // Window lifecycle
        Msg::WindowCloseRequested(id) => {
            state.windows.remove(&id);
            state.window_positions.remove(&id);
            state.window_sizes.remove(&id);
            if id == state.main_window_id {
                if regen {
                    let output = generate::generate_icss(&mut state.vars, state.save_icss);
                    if let Ok(new_theme) = Theme::load(&output.icss) {
                        state.theme = new_theme;
                    }
                    state.dims = output.dims;
                    state.neutral_palette = output.neutral_palette;
                    state.family_steps = output.family_steps;
                    if state.save_icss {
                        persist::save(&state.vars);
                    }
                }
                return window::close(id).chain(iced::exit());
            } else {
                if regen {
                    let output = generate::generate_icss(&mut state.vars, state.save_icss);
                    if let Ok(new_theme) = Theme::load(&output.icss) {
                        state.theme = new_theme;
                    }
                    state.dims = output.dims;
                    state.neutral_palette = output.neutral_palette;
                    state.family_steps = output.family_steps;
                    if state.save_icss {
                        persist::save(&state.vars);
                    }
                }
                return window::close(id);
            }
        }
        Msg::WindowMoved(id, pt) | Msg::GotPosition(id, Some(pt)) => {
            let prev = state.window_positions.insert(id, pt);
            // Any non-main window can merge into any other window — even if it
            // has multiple tabs (all of them transfer on merge).
            if id != state.main_window_id && prev != Some(pt) {
                state.merge_highlight = find_merge_target(
                    id,
                    &state.window_positions,
                    &state.window_sizes,
                    &state.windows,
                );
                state.merge_pending = Some((id, std::time::Instant::now()));
            }
            // Check pending merge (300ms debounce)
            if let Some((mid, t)) = state.merge_pending
                && t.elapsed() >= std::time::Duration::from_millis(100)
            {
                state.merge_pending = None;
                if let Some(target_id) = find_merge_target(
                    mid,
                    &state.window_positions,
                    &state.window_sizes,
                    &state.windows,
                ) {
                    state.merge_highlight = None;
                    if let Some(src_ws) = state.windows.remove(&mid)
                        && let Some(target_ws) = state.windows.get_mut(&target_id)
                    {
                        let last_tab = src_ws.tabs.last().map(|t| t.id);
                        target_ws.tabs.extend(src_ws.tabs);
                        if let Some(tid) = last_tab {
                            target_ws.active_tab = tid;
                        }
                    }
                    state.window_positions.remove(&mid);
                    state.window_sizes.remove(&mid);
                    return window::close(mid);
                }
            }
        }
        Msg::WindowResized(id, sz) => {
            state.window_sizes.insert(id, sz);
        }
        Msg::PollPositions => {
            let tasks: Vec<Task<Msg>> = state
                .windows
                .keys()
                .map(|&wid| window::position(wid).map(move |opt_pt| Msg::GotPosition(wid, opt_pt)))
                .collect();
            return Task::batch(tasks);
        }
        Msg::GotPosition(_, None) => {}
        Msg::Noop => {}
    }

    if regen {
        let output = generate::generate_icss(&mut state.vars, state.save_icss);
        if let Ok(new_theme) = Theme::load(&output.icss) {
            state.theme = new_theme;
        }
        state.dims = output.dims;
        state.neutral_palette = output.neutral_palette;
        state.family_steps = output.family_steps;
        if state.save_icss {
            persist::save(&state.vars);
        }

        // Re-apply titlebar background to match updated theme.
        #[cfg(target_os = "macos")]
        {
            let mut tasks: Vec<Task<Msg>> = Vec::new();
            for wid in state.windows.keys() {
                let title = title(state, *wid);
                tasks.push(Task::perform(async {}, move |_| {
                    Msg::ReshapeTitlebar(title, 1)
                }));
            }
            if !tasks.is_empty() {
                return Task::batch(tasks);
            }
        }
    }

    Task::none()
}

/// With fullsize_content_view, content extends behind the titlebar, so there
/// is no offset between the window frame origin and the content area.
const NATIVE_TITLEBAR_H: f32 = 0.0;

/// Compute screen-space cursor position from a dragged window's origin +
/// the offset captured at detach. Returns None for windows that were never
/// detached (main window) — those can't be drag-sources for a merge.
fn screen_cursor(
    id: window::Id,
    positions: &HashMap<window::Id, Point>,
    windows: &HashMap<window::Id, WindowState>,
) -> Option<Point> {
    let &pt = positions.get(&id)?;
    let (gx, gy) = windows.get(&id)?.grab_offset?;
    // window::position returns outer frame origin. The cursor offset
    // (gx, gy) is in content coords, so add the titlebar height on Y.
    Some(Point::new(pt.x + gx, pt.y + NATIVE_TITLEBAR_H + gy))
}

/// Check if the cursor is inside any other window's tab bar.
fn find_merge_target(
    id: window::Id,
    positions: &HashMap<window::Id, Point>,
    sizes: &HashMap<window::Id, Size>,
    windows: &HashMap<window::Id, WindowState>,
) -> Option<window::Id> {
    let cursor = screen_cursor(id, positions, windows)?;
    let tab_bar_h = 40.0;

    windows
        .keys()
        .filter(|&&oid| oid != id)
        .find(|&&oid| {
            if let (Some(&op), Some(&os)) = (positions.get(&oid), sizes.get(&oid)) {
                let ob = iced::Rectangle {
                    x: op.x,
                    y: op.y + NATIVE_TITLEBAR_H,
                    width: os.width,
                    height: tab_bar_h,
                };
                ob.contains(cursor)
            } else {
                false
            }
        })
        .copied()
}

fn view(state: &State, window_id: window::Id) -> Element<'_, Msg> {
    let ws = match state.windows.get(&window_id) {
        Some(ws) => ws,
        None => return text("Window not found").into(),
    };

    let wid = window_id;
    let _is_drop_target = state.merge_highlight == Some(window_id);

    #[cfg(target_os = "macos")]
    let tab_layout = TabBarStyle {
        left_pad: 78.0,
        top_pad: 3.0,
        ..Default::default()
    };
    #[cfg(not(target_os = "macos"))]
    let tab_layout = TabBarStyle {
        top_pad: 3.0,
        ..Default::default()
    };

    let tab_bar_widget: Element<'_, Msg> = TabBar::new(
        ws.tabs.clone(),
        ws.active_tab,
        &ws.tab_drag,
        &state.theme,
        move |action| Msg::TabAction(wid, action),
    )
    .layout_style(tab_layout)
    .into();

    // Right-aligned "Save .icss" toggle living in the tab-bar strip. When
    // checked, theme edits persist to disk; unchecked is preview-only.
    let sm = state.theme.sizing(&["sz-sm"]);
    let save_toggle: Element<'_, Msg> = container(
        checkbox(state.save_icss)
            .label("Save .icss")
            .size(sm.font_size)
            .text_size(sm.font_size)
            .spacing(sm.gap)
            .on_toggle(Msg::SaveIcssToggled)
            .style(state.theme.checkbox(&["checkbox", "sz-sm"])),
    )
    .height(Length::Fixed(40.0))
    .align_y(iced::alignment::Vertical::Center)
    .padding(Padding::ZERO.right(state.dims.space_200))
    .into();

    let tab_bar: Element<'_, Msg> =
        row![container(tab_bar_widget).width(Length::Fill), save_toggle,]
            .height(Length::Fixed(40.0))
            .into();

    let showcase = window_content_view(state, ws);

    column![tab_bar, showcase]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn window_content_view<'a>(state: &'a State, ws: &'a WindowState) -> Element<'a, Msg> {
    let t = &state.theme;
    let d = &state.dims;

    let sidebar = sidebar_view(state);
    let page_content = tab_content(state, ws.active_tab);

    let content = column![page_content]
        .spacing(d.space_250)
        .padding(Padding::from([d.space_400, d.space_400]));

    // Each tab gets its own scrollable ID so scroll positions are independent.
    let page = container(
        scrollable(content)
            .id(iced::widget::Id::from(format!(
                "tab-scroll-{}",
                ws.active_tab
            )))
            .style(t.scrollable(&["scroll"]))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(t.container(&["page"]));

    row![sidebar, page]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── Sidebar ──

fn sidebar_view(state: &State) -> Element<'_, Msg> {
    let vars = &state.vars;
    let ts = &state.sidebar_theme;

    // Color picker (SV square + hue bar + preview)
    let preview_color = color_picker::HsvColor {
        h: state.picker_hue,
        s: state.picker_sat,
        v: state.picker_val,
    }
    .to_rgb();

    let sv: Element<'_, Msg> =
        color_picker::sv_square(state.picker_hue, state.picker_sat, state.picker_val).map(|msg| {
            match msg {
                color_picker::SvMsg::Changed(s, v) => Msg::PickerSvChanged(s, v),
            }
        });
    let hue: Element<'_, Msg> = color_picker::hue_bar(state.picker_hue).map(|msg| match msg {
        color_picker::HueMsg::Changed(h) => Msg::PickerHueChanged(h),
    });

    let picker = column![
        text(format!("Picker: {}", state.active_color.label())).size(12),
        sv,
        hue,
        container("")
            .width(Length::Fill)
            .height(20)
            .style(move |_theme: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(preview_color)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .spacing(6);

    // Color fields with swatch buttons
    let color_fields = column![
        text("Theme Colors").size(14),
        color_row(ColorField::Primary, vars, ts, state.active_color),
        color_row(ColorField::Secondary, vars, ts, state.active_color),
        color_row(ColorField::Tertiary, vars, ts, state.active_color),
        color_row(ColorField::Quaternary, vars, ts, state.active_color),
        color_row(ColorField::Neutral, vars, ts, state.active_color),
        color_row(ColorField::Link, vars, ts, state.active_color),
    ]
    .spacing(4);

    // Signal colors — clickable rows. Empty override → swatch shows the
    // derived colour and the row label gets a small "(auto)" suffix.
    let signal_display = column![
        text("Signal Colors").size(14),
        signal_row(ColorField::Success, vars, ts, state.active_color),
        signal_row(ColorField::Danger, vars, ts, state.active_color),
        signal_row(ColorField::Warning, vars, ts, state.active_color),
    ]
    .spacing(4);

    let dim_fields = column![
        text("Dimensions").size(14),
        dim_slider(
            "Increment",
            vars.increment,
            4.0..=16.0,
            1.0,
            Msg::IncrementChanged
        ),
        dim_slider(
            "Font base",
            vars.font_increment,
            6.0..=14.0,
            1.0,
            Msg::FontIncrementChanged
        ),
        dim_slider(
            "Radius",
            vars.radius_factor,
            0.0..=3.0,
            0.1,
            Msg::RadiusFactorChanged
        ),
    ]
    .spacing(6);

    let font_options = vec![FontFamily::SFPro, FontFamily::SegoeUI, FontFamily::Roboto];
    let font_picker = column![
        text("Font Family").size(14),
        pick_list(
            font_options,
            Some(vars.font_family),
            Msg::FontFamilySelected
        )
        .placeholder("Select..."),
    ]
    .spacing(6);

    let mode_toggle = column![
        text("Mode").size(14),
        toggler(vars.dark_mode)
            .label(if vars.dark_mode { "Dark" } else { "Light" })
            .on_toggle(Msg::DarkModeToggled),
    ]
    .spacing(6);

    let surface_slider = column![
        text("Surface").size(14),
        dim_slider(
            "Lightness",
            vars.surface_lightness,
            0.0..=100.0,
            1.0,
            Msg::SurfaceLightnessChanged
        ),
    ]
    .spacing(6);

    let gamma_slider = column![
        text("Gamma").size(14),
        dim_slider("Gamma", vars.gamma, 0.3..=3.0, 0.05, Msg::GammaChanged),
        text("Text Spread").size(14),
        dim_slider(
            "Stretch",
            vars.text_spread,
            0.1..=5.0,
            0.1,
            Msg::TextSpreadChanged,
        ),
    ]
    .spacing(6);

    let restart_btn = button(text("Restart (apply font)").size(12))
        .on_press(Msg::RestartApp)
        .style(ts.button(&["button", "ghost"]));

    let tonal: Element<'_, Msg> =
        color_picker::tonal_bar(&state.neutral_palette).map(|_| Msg::Noop);

    let sidebar_content = column![
        text("Theme Variables").size(18),
        column![text("Neutral tonal ladder (0\u{2013}100)").size(10), tonal].spacing(2),
        rule::horizontal(1).style(ts.rule(&["divider"])),
        picker,
        rule::horizontal(1).style(ts.rule(&["divider"])),
        color_fields,
        signal_display,
        rule::horizontal(1).style(ts.rule(&["divider"])),
        dim_fields,
        rule::horizontal(1).style(ts.rule(&["divider"])),
        surface_slider,
        rule::horizontal(1).style(ts.rule(&["divider"])),
        gamma_slider,
        rule::horizontal(1).style(ts.rule(&["divider"])),
        mode_toggle,
        rule::horizontal(1).style(ts.rule(&["divider"])),
        font_picker,
        restart_btn,
    ]
    .spacing(12)
    .padding(Padding::from([16, 16]));

    container(scrollable(sidebar_content).height(Length::Fill))
        .width(280)
        .height(Length::Fill)
        .style(ts.container(&["sidebar"]))
        .into()
}

fn color_row<'a>(
    field: ColorField,
    vars: &'a ThemeVars,
    t: &'a Theme,
    active: ColorField,
) -> Element<'a, Msg> {
    let hex = field.get(vars);
    let on_change: Box<dyn Fn(String) -> Msg + 'a> = match field {
        ColorField::Primary => Box::new(Msg::PrimaryChanged),
        ColorField::Secondary => Box::new(Msg::SecondaryChanged),
        ColorField::Tertiary => Box::new(Msg::TertiaryChanged),
        ColorField::Quaternary => Box::new(Msg::QuaternaryChanged),
        ColorField::Neutral => Box::new(Msg::NeutralChanged),
        ColorField::Link => Box::new(Msg::LinkChanged),
        ColorField::Success => Box::new(Msg::SuccessChanged),
        ColorField::Danger => Box::new(Msg::DangerChanged),
        ColorField::Warning => Box::new(Msg::WarningChanged),
    };

    // Parse hex to show a color swatch.
    let swatch_color = color_picker::HsvColor::from_hex(hex)
        .map(|h| h.to_rgb())
        .unwrap_or(iced::Color::TRANSPARENT);

    let is_active = field == active;
    let border_color = if is_active {
        iced::Color::WHITE
    } else {
        iced::Color::TRANSPARENT
    };

    row![
        // Clickable swatch
        iced::widget::mouse_area(container("").width(16).height(16).style(
            move |_theme: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(swatch_color)),
                border: iced::Border {
                    radius: 3.0.into(),
                    width: 1.5,
                    color: border_color,
                },
                ..Default::default()
            }
        ),)
        .on_press(Msg::SelectColorField(field)),
        // Label
        container(text(field.label()).size(10)).width(60),
        // Hex input
        text_input("#hex", hex)
            .on_input(on_change)
            .padding(4)
            .size(11)
            .style(t.text_input(&["sidebar-input"])),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Signal-colour row with the same picker mechanics as `color_row`, plus
/// a tiny "Reset" button when a manual override is in effect (clears the
/// override and lets the engine re-derive from P/S/T/Q).
fn signal_row<'a>(
    field: ColorField,
    vars: &'a ThemeVars,
    t: &'a Theme,
    active: ColorField,
) -> Element<'a, Msg> {
    debug_assert!(field.is_signal());

    let override_value: &str = match field {
        ColorField::Success => &vars.success_override,
        ColorField::Danger => &vars.danger_override,
        ColorField::Warning => &vars.warning_override,
        _ => "",
    };
    let is_auto = override_value.trim().is_empty();
    let display_hex = field.get(vars); // resolves override → derived

    let on_change: Box<dyn Fn(String) -> Msg + 'a> = match field {
        ColorField::Success => Box::new(Msg::SuccessChanged),
        ColorField::Danger => Box::new(Msg::DangerChanged),
        ColorField::Warning => Box::new(Msg::WarningChanged),
        _ => Box::new(Msg::PrimaryChanged), // unreachable
    };

    let swatch_color = color_picker::HsvColor::from_hex(display_hex)
        .map(|h| h.to_rgb())
        .unwrap_or(iced::Color::TRANSPARENT);

    let is_active = field == active;
    let border_color = if is_active {
        iced::Color::WHITE
    } else {
        iced::Color::TRANSPARENT
    };

    let label_str = if is_auto {
        format!("{} (auto)", field.label())
    } else {
        field.label().to_string()
    };

    let mut r = row![
        iced::widget::mouse_area(container("").width(16).height(16).style(
            move |_theme: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(swatch_color)),
                border: iced::Border {
                    radius: 3.0.into(),
                    width: 1.5,
                    color: border_color,
                },
                ..Default::default()
            }
        ),)
        .on_press(Msg::SelectColorField(field)),
        container(text(label_str).size(10)).width(80),
        text_input("auto", override_value)
            .on_input(on_change)
            .padding(4)
            .size(11)
            .style(t.text_input(&["sidebar-input"])),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    if !is_auto {
        let reset_btn = button(text("×").size(10))
            .on_press(Msg::SignalReset(field))
            .padding([2, 6]);
        r = r.push(reset_btn);
    }

    r.into()
}

fn signal_swatch<'a>(label: &'a str, hex: &'a str) -> Element<'a, Msg> {
    let color = color_picker::HsvColor::from_hex(hex)
        .map(|h| h.to_rgb())
        .unwrap_or(iced::Color::from_rgb(0.5, 0.5, 0.5));

    row![
        container("")
            .width(16)
            .height(16)
            .style(move |_theme: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(color)),
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        text(label).size(10),
        text(hex)
            .size(10)
            .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.4)),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}

fn dim_slider<'a>(
    label: &'a str,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    step: f32,
    on_change: impl Fn(f32) -> Msg + 'a,
) -> Element<'a, Msg> {
    let display = if step >= 1.0 {
        format!("{}: {}", label, value as i32)
    } else {
        format!("{}: {:.1}", label, value)
    };
    column![
        text(display).size(11),
        slider(range, value, on_change).step(step),
    ]
    .spacing(2)
    .into()
}

// ── Showcase ──

fn tab_content<'a>(state: &'a State, tab_id: TabId) -> Element<'a, Msg> {
    let d = &state.dims;
    match tab_id {
        0 => icss_controls_page(state),
        1 => primitives_page(state),
        _ => {
            // Find the tab title across all windows
            let tab_title = state
                .windows
                .values()
                .flat_map(|ws| ws.tabs.iter())
                .find(|t| t.id == tab_id)
                .map(|t| t.title.as_str())
                .unwrap_or("Unknown");
            container(
                column![
                    text(tab_title).size(d.font_title_medium),
                    text("New tab — add your content here").size(d.font_body_medium),
                ]
                .spacing(d.space_200),
            )
            .padding(d.space_400)
            .into()
        }
    }
}

fn components_page(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let d = &state.dims;

    column![
        buttons_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        inputs_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        button_group_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        control_group_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        controls_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        sliders_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        progress_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        pick_list_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        editor_tooltip_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        chat_textarea_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        tile_grid_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        data_table_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        typography_section(state),
        rule::horizontal(1).style(t.rule(&["divider"])),
        text_colors_section(state),
    ]
    .spacing(d.space_250)
    .into()
}

/// ICSS Controls page — every widget styled exactly per COMPONENT-CATALOG.md.
/// Uses one-call builders (`t.btn()`, `t.input()`, etc.) and `icss::widgets::protect()`
/// for min-size enforcement. This is how the desktop app should build its controls.
///
/// Mirrors the 14 sections in `components_page` in the same order.
fn icss_controls_page(state: &State) -> Element<'_, Msg> {
    use iced::widget::svg;
    let t = &state.theme;
    let d = &state.dims;

    // ── 1. Buttons ──────────────────────────────────────────────────
    // Every button is a direct t.btn() / t.btn_icon() / t.btn_sq() call.
    // No wrapper closures — copy any line directly into app code.
    let btn_section = {
        use icss::widgets::mdi;

        // Resolve icon tint from the button's text color.
        let icon_tint = |classes: &[&str]| -> iced::Color {
            let computed = t.resolve(classes, None);
            icss::theme::Theme::resolve_color(&computed, "color")
                .map(|c| c.to_iced())
                .unwrap_or(iced::Color::WHITE)
        };

        // MDI icon sized to icon_size with accent-color tint — for icon+label buttons.
        let icon_box = |icon_size: f32, classes: &[&str]| -> Element<'_, Msg> {
            let c = icon_tint(classes);
            svg(mdi::icon_handle(mdi::SETTINGS))
                .width(icon_size)
                .height(icon_size)
                .style(move |_, _| iced::widget::svg::Style { color: Some(c) })
                .into()
        };
        // MDI icon sized to icon_size with accent-color tint — for icon-only buttons.
        let sq_icon = |icon_size: f32, classes: &[&str]| -> Element<'_, Msg> {
            let c = icon_tint(classes);
            svg(mdi::icon_handle(mdi::SETTINGS))
                .width(icon_size)
                .height(icon_size)
                .style(move |_, _| iced::widget::svg::Style { color: Some(c) })
                .into()
        };

        let dis = state.buttons_disabled;
        let press = if dis { None } else { Some(Msg::Noop) };

        // ── Primary ──
        let primary_cat = t
            .column(&["stack-tight"])
            .push(t.text("Primary", &["label-small"]))
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "primary", "sz-md"]),
                        {
                            let mut b = t.btn("Label", &["button", "primary", "sz-md"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "primary", "sz-sm"]),
                        {
                            let mut b = t.btn("Label", &["button", "primary", "sz-sm"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "primary", "sz-xs"]),
                        {
                            let mut b = t.btn("Label", &["button", "primary", "sz-xs"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "primary", "sz-md"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-md"]).icon_size,
                                    &["button", "primary", "sz-md"],
                                ),
                                "Label",
                                &["button", "primary", "sz-md"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "primary", "sz-sm"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-sm"]).icon_size,
                                    &["button", "primary", "sz-sm"],
                                ),
                                "Label",
                                &["button", "primary", "sz-sm"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "primary", "sz-xs"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-xs"]).icon_size,
                                    &["button", "primary", "sz-xs"],
                                ),
                                "Label",
                                &["button", "primary", "sz-xs"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "primary", "sz-md"],
                            ),
                            &["button", "primary", "sz-md"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "primary", "sz-sm"],
                            ),
                            &["button", "primary", "sz-sm"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "primary", "sz-xs"],
                            ),
                            &["button", "primary", "sz-xs"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "primary", "sz-md", "round"],
                            ),
                            &["button", "primary", "sz-md", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "primary", "sz-sm", "round"],
                            ),
                            &["button", "primary", "sz-sm", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "primary", "sz-xs", "round"],
                            ),
                            &["button", "primary", "sz-xs", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            );

        // ── Success ──
        let success_cat = t
            .column(&["stack-tight"])
            .push(t.text("Success", &["label-small"]))
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "success", "sz-md"]),
                        {
                            let mut b = t.btn("Label", &["button", "success", "sz-md"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "success", "sz-sm"]),
                        {
                            let mut b = t.btn("Label", &["button", "success", "sz-sm"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "success", "sz-xs"]),
                        {
                            let mut b = t.btn("Label", &["button", "success", "sz-xs"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "success", "sz-md"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-md"]).icon_size,
                                    &["button", "success", "sz-md"],
                                ),
                                "Label",
                                &["button", "success", "sz-md"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "success", "sz-sm"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-sm"]).icon_size,
                                    &["button", "success", "sz-sm"],
                                ),
                                "Label",
                                &["button", "success", "sz-sm"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "success", "sz-xs"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-xs"]).icon_size,
                                    &["button", "success", "sz-xs"],
                                ),
                                "Label",
                                &["button", "success", "sz-xs"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "success", "sz-md"],
                            ),
                            &["button", "success", "sz-md"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "success", "sz-sm"],
                            ),
                            &["button", "success", "sz-sm"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "success", "sz-xs"],
                            ),
                            &["button", "success", "sz-xs"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "success", "sz-md", "round"],
                            ),
                            &["button", "success", "sz-md", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "success", "sz-sm", "round"],
                            ),
                            &["button", "success", "sz-sm", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "success", "sz-xs", "round"],
                            ),
                            &["button", "success", "sz-xs", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            );

        // ── Danger ──
        let danger_cat = t
            .column(&["stack-tight"])
            .push(t.text("Danger", &["label-small"]))
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "danger", "sz-md"]),
                        {
                            let mut b = t.btn("Label", &["button", "danger", "sz-md"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "danger", "sz-sm"]),
                        {
                            let mut b = t.btn("Label", &["button", "danger", "sz-sm"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "danger", "sz-xs"]),
                        {
                            let mut b = t.btn("Label", &["button", "danger", "sz-xs"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "danger", "sz-md"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-md"]).icon_size,
                                    &["button", "danger", "sz-md"],
                                ),
                                "Label",
                                &["button", "danger", "sz-md"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "danger", "sz-sm"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-sm"]).icon_size,
                                    &["button", "danger", "sz-sm"],
                                ),
                                "Label",
                                &["button", "danger", "sz-sm"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "danger", "sz-xs"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-xs"]).icon_size,
                                    &["button", "danger", "sz-xs"],
                                ),
                                "Label",
                                &["button", "danger", "sz-xs"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "danger", "sz-md"],
                            ),
                            &["button", "danger", "sz-md"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "danger", "sz-sm"],
                            ),
                            &["button", "danger", "sz-sm"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "danger", "sz-xs"],
                            ),
                            &["button", "danger", "sz-xs"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "danger", "sz-md", "round"],
                            ),
                            &["button", "danger", "sz-md", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "danger", "sz-sm", "round"],
                            ),
                            &["button", "danger", "sz-sm", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "danger", "sz-xs", "round"],
                            ),
                            &["button", "danger", "sz-xs", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            );

        // ── Default ──
        let default_cat = t
            .column(&["stack-tight"])
            .push(t.text("Default", &["label-small"]))
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "default", "sz-md"]),
                        {
                            let mut b = t.btn("Label", &["button", "default", "sz-md"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "default", "sz-sm"]),
                        {
                            let mut b = t.btn("Label", &["button", "default", "sz-sm"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "default", "sz-xs"]),
                        {
                            let mut b = t.btn("Label", &["button", "default", "sz-xs"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "default", "sz-md"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-md"]).icon_size,
                                    &["button", "default", "sz-md"],
                                ),
                                "Label",
                                &["button", "default", "sz-md"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "default", "sz-sm"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-sm"]).icon_size,
                                    &["button", "default", "sz-sm"],
                                ),
                                "Label",
                                &["button", "default", "sz-sm"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "default", "sz-xs"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-xs"]).icon_size,
                                    &["button", "default", "sz-xs"],
                                ),
                                "Label",
                                &["button", "default", "sz-xs"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "default", "sz-md"],
                            ),
                            &["button", "default", "sz-md"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "default", "sz-sm"],
                            ),
                            &["button", "default", "sz-sm"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "default", "sz-xs"],
                            ),
                            &["button", "default", "sz-xs"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "default", "sz-md", "round"],
                            ),
                            &["button", "default", "sz-md", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "default", "sz-sm", "round"],
                            ),
                            &["button", "default", "sz-sm", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "default", "sz-xs", "round"],
                            ),
                            &["button", "default", "sz-xs", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            );

        // ── Ghost ──
        let ghost_cat = t
            .column(&["stack-tight"])
            .push(t.text("Ghost", &["label-small"]))
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "ghost", "sz-md"]),
                        {
                            let mut b = t.btn("Label", &["button", "ghost", "sz-md"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "ghost", "sz-sm"]),
                        {
                            let mut b = t.btn("Label", &["button", "ghost", "sz-sm"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "ghost", "sz-xs"]),
                        {
                            let mut b = t.btn("Label", &["button", "ghost", "sz-xs"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "ghost", "sz-md"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-md"]).icon_size,
                                    &["button", "ghost", "sz-md"],
                                ),
                                "Label",
                                &["button", "ghost", "sz-md"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "ghost", "sz-sm"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-sm"]).icon_size,
                                    &["button", "ghost", "sz-sm"],
                                ),
                                "Label",
                                &["button", "ghost", "sz-sm"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "ghost", "sz-xs"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-xs"]).icon_size,
                                    &["button", "ghost", "sz-xs"],
                                ),
                                "Label",
                                &["button", "ghost", "sz-xs"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "ghost", "sz-md"],
                            ),
                            &["button", "ghost", "sz-md"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "ghost", "sz-sm"],
                            ),
                            &["button", "ghost", "sz-sm"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "ghost", "sz-xs"],
                            ),
                            &["button", "ghost", "sz-xs"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "ghost", "sz-md", "round"],
                            ),
                            &["button", "ghost", "sz-md", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "ghost", "sz-sm", "round"],
                            ),
                            &["button", "ghost", "sz-sm", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "ghost", "sz-xs", "round"],
                            ),
                            &["button", "ghost", "sz-xs", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            );

        // ── Outlined ──
        let outlined_cat = t
            .column(&["stack-tight"])
            .push(t.text("Outlined", &["label-small"]))
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "outlined", "sz-md"]),
                        {
                            let mut b = t.btn("Label", &["button", "outlined", "sz-md"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "outlined", "sz-sm"]),
                        {
                            let mut b = t.btn("Label", &["button", "outlined", "sz-sm"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "outlined", "sz-xs"]),
                        {
                            let mut b = t.btn("Label", &["button", "outlined", "sz-xs"]);
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "outlined", "sz-md"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-md"]).icon_size,
                                    &["button", "outlined", "sz-md"],
                                ),
                                "Label",
                                &["button", "outlined", "sz-md"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "outlined", "sz-sm"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-sm"]).icon_size,
                                    &["button", "outlined", "sz-sm"],
                                ),
                                "Label",
                                &["button", "outlined", "sz-sm"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .push(icss::widgets::protect(
                        &t.sizing(&["button", "outlined", "sz-xs"]),
                        {
                            let mut b = t.btn_icon(
                                icon_box(
                                    t.sizing(&["sz-xs"]).icon_size,
                                    &["button", "outlined", "sz-xs"],
                                ),
                                "Label",
                                &["button", "outlined", "sz-xs"],
                            );
                            if let Some(ref m) = press {
                                b = b.on_press(m.clone());
                            }
                            b
                        },
                    ))
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "outlined", "sz-md"],
                            ),
                            &["button", "outlined", "sz-md"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "outlined", "sz-sm"],
                            ),
                            &["button", "outlined", "sz-sm"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "outlined", "sz-xs"],
                            ),
                            &["button", "outlined", "sz-xs"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["cluster"])
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-md"]).icon_size,
                                &["button", "outlined", "sz-md", "round"],
                            ),
                            &["button", "outlined", "sz-md", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-sm"]).icon_size,
                                &["button", "outlined", "sz-sm", "round"],
                            ),
                            &["button", "outlined", "sz-sm", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .push({
                        let mut b = t.btn_sq(
                            sq_icon(
                                t.sizing(&["sz-xs"]).icon_size,
                                &["button", "outlined", "sz-xs", "round"],
                            ),
                            &["button", "outlined", "sz-xs", "round"],
                        );
                        if let Some(ref m) = press {
                            b = b.on_press(m.clone());
                        }
                        b
                    })
                    .align_y(iced::Alignment::Center),
            );

        // ── Emphasized (gradient outline) — mirrors the Components tab ──
        // AD-HOC: gradient buttons require inline container::Style because ICSS
        // has no gradient support. Colors come from theme vars, only the
        // Background::Gradient construction is custom.
        let cv = |name: &str| t.color_var(name).unwrap_or(iced::Color::WHITE);
        let c_cont_s4 = cv("surface-primary-container-s4");
        let c_s0 = cv("surface-primary-s0");
        let c_s1 = cv("surface-primary-s1");
        let c_s2 = cv("surface-primary-s2");
        let c_s3 = cv("surface-primary-s3");
        let c_on = cv("on-surface-primary");
        let pi = std::f32::consts::PI;
        let sb = state
            .vars
            .font_family
            .weighted(iced::font::Weight::Semibold);

        let hover_idx = state.gradient_hover;
        let press_idx = state.gradient_pressed;
        let mut emph_counter: usize = 0;

        let mut emph_btn = |sz_cls: &[&str], icon: bool, square: bool| -> Element<'_, Msg> {
            let idx = emph_counter;
            emph_counter += 1;
            let hovered = hover_idx == Some(idx);
            let pressed = press_idx == Some(idx);
            let sz = t.sizing(sz_cls);
            let r = d.radius_100;

            let line_h = 1.3_f32;
            let sq_size = (sz.font_size * line_h).ceil() + 2.0 * sz.pad_v;

            let content: Element<'_, Msg> = if square {
                container(
                    iced_fonts::bootstrap::play_fill()
                        .size(sz.font_size)
                        .color(c_on),
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(sq_size - 4.0)
                .height(sq_size - 4.0)
                .into()
            } else if icon {
                container(
                    row![
                        iced_fonts::bootstrap::play_fill()
                            .size(sz.font_size)
                            .color(c_on),
                        text("Label").size(sz.font_size).font(sb).color(c_on)
                    ]
                    .spacing(sz.gap)
                    .align_y(iced::Alignment::Center),
                )
                .center_x(Length::Shrink)
                .into()
            } else {
                text("Label").size(sz.font_size).font(sb).color(c_on).into()
            };

            let inner_pad = if square { Padding::ZERO } else { sz.padding() };

            let s0 = c_s0;
            let s1 = c_s1;
            let s2 = c_s2;
            let s3 = c_s3;
            let cont4 = c_cont_s4;
            let alpha = if dis { 0.5 } else { 1.0 };
            let (it, ib) = if dis {
                (s0, s0)
            } else if pressed {
                (s2, s1)
            } else if hovered {
                (s1, s2)
            } else {
                (s0, s1)
            };
            let outline_c = if pressed || hovered { s3 } else { s2 };
            let (ot, om, ob) = if dis {
                (s0, s0, s0)
            } else if pressed {
                (s1, s1, s1)
            } else {
                (cont4, s0, s0)
            };

            let a = alpha;
            let apply_a = move |c: iced::Color| iced::Color { a: c.a * a, ..c };

            mouse_area(
                container(
                    container(content)
                        .padding(inner_pad)
                        .center_x(Length::Shrink)
                        .style(move |_: &IcedTheme| iced::widget::container::Style {
                            background: Some(iced::Background::Gradient(
                                iced::gradient::Linear::new(pi)
                                    .add_stop(0.0, apply_a(it))
                                    .add_stop(1.0, apply_a(ib))
                                    .into(),
                            )),
                            border: iced::Border {
                                radius: (r - 2.0).max(0.0).into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                )
                .padding(2)
                .style(move |_: &IcedTheme| iced::widget::container::Style {
                    background: Some(iced::Background::Gradient(
                        iced::gradient::Linear::new(pi)
                            .add_stop(0.0, apply_a(ot))
                            .add_stop(0.4, apply_a(om))
                            .add_stop(1.0, apply_a(ob))
                            .into(),
                    )),
                    border: iced::Border {
                        radius: r.into(),
                        width: 1.0,
                        color: apply_a(outline_c),
                    },
                    ..Default::default()
                }),
            )
            .on_enter(Msg::GradientEnter(idx))
            .on_exit(Msg::GradientExit(idx))
            .on_press(Msg::GradientPress(idx))
            .on_release(Msg::GradientRelease(idx))
            .into()
        };

        let emph_cat: Element<'_, Msg> = column![
            text("Primary Emphasized").size(d.font_label_small),
            row![
                emph_btn(&["sz-md"], true, false),
                emph_btn(&["sz-sm"], true, false),
                emph_btn(&["sz-xs"], true, false),
            ]
            .spacing(d.space_75)
            .align_y(iced::Alignment::Center),
            row![
                emph_btn(&["sz-md"], false, true),
                emph_btn(&["sz-sm"], false, true),
                emph_btn(&["sz-xs"], false, true),
            ]
            .spacing(d.space_75)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(d.space_50)
        .into();

        // Disabled toggle
        let disabled_toggle: Element<'_, Msg> = icss::widgets::protect(
            &t.sizing(&["toggle", "sz-md"]),
            t.toggle("Disabled", state.buttons_disabled, &["toggle", "sz-md"])
                .on_toggle(Msg::ButtonsDisabledToggle)
                .width(Length::Shrink),
        );

        let header = t
            .row(&["row-loose"])
            .push(t.text("Buttons", &["title-small"]))
            .push(disabled_toggle)
            .align_y(iced::Alignment::Center);

        t.frame(
            t.column(&["stack"])
                .push(header)
                .push(primary_cat)
                .push(success_cat)
                .push(danger_cat)
                .push(default_cat)
                .push(ghost_cat)
                .push(outlined_cat)
                .push(emph_cat),
            &["section", "section-body"],
        )
        .width(Length::Fill)
    };

    // ── 2. Text Inputs ──────────────────────────────────────────────
    // Uses t.input() builder + protect() for all sizes and states.
    let input_section = {
        let sz_md = t.sizing(&["input", "sz-md"]);
        let sz_sm = t.sizing(&["input", "sz-sm"]);
        let sz_xs = t.sizing(&["input", "sz-xs"]);

        let three_sizes = t
            .row(&["row"])
            .push(icss::widgets::protect(
                &sz_md,
                t.input("Medium input...", &state.text_value, &["input", "sz-md"])
                    .on_input(Msg::TextChanged),
            ))
            .push(icss::widgets::protect(
                &sz_sm,
                t.input("Small input...", &state.text_value, &["input", "sz-sm"])
                    .on_input(Msg::TextChanged),
            ))
            .push(icss::widgets::protect(
                &sz_xs,
                t.input("Tiny input...", &state.text_value, &["input", "sz-xs"])
                    .on_input(Msg::TextChanged),
            ))
            .align_y(iced::Alignment::Center);

        let states = t
            .row(&["row"])
            .push(
                t.column(&["field-col"])
                    .push(icss::widgets::protect(
                        &sz_md,
                        t.input(
                            "Required field",
                            &state.error_value,
                            &["input", "error", "sz-md"],
                        )
                        .on_input(Msg::ErrorTextChanged),
                    ))
                    .push(t.text("This field is required", &["label-micro", "text-danger"]))
                    .width(Length::Fill),
            )
            .push(icss::widgets::protect(
                &sz_md,
                t.input("Disabled", "Cannot edit", &["input", "sz-md"]),
            ));

        // With icons — use IconInput (same as Components tab)
        use iced_fonts::bootstrap;
        use icss::widgets::icon_input::IconInput;

        let ic =
            |_sz: f32| -> iced::Color { t.color_var("text-soft").unwrap_or(iced::Color::WHITE) };
        let icon_sz = |font_sz: f32| -> f32 { (font_sz * 0.85).round() };
        let md = t.sizing(&["sz-md"]);
        let sm = t.sizing(&["sz-sm"]);
        let xs = t.sizing(&["sz-xs"]);

        let search_md = IconInput::new("Search...", &state.text_value)
            .leading(
                bootstrap::search()
                    .size(icon_sz(md.font_size))
                    .color(ic(0.0)),
            )
            .trailing(bootstrap::x_lg().size(icon_sz(md.font_size)).color(ic(0.0)))
            .on_input(Msg::TextChanged)
            .input_style(&["input"])
            .sizing(&["sz-md"])
            .view(t);

        let search_sm = IconInput::new("Search...", &state.text_value)
            .leading(
                bootstrap::search()
                    .size(icon_sz(sm.font_size))
                    .color(ic(0.0)),
            )
            .trailing(bootstrap::x_lg().size(icon_sz(sm.font_size)).color(ic(0.0)))
            .on_input(Msg::TextChanged)
            .input_style(&["input"])
            .sizing(&["sz-sm"])
            .view(t);

        let search_xs = IconInput::new("Search...", &state.text_value)
            .leading(
                bootstrap::search()
                    .size(icon_sz(xs.font_size))
                    .color(ic(0.0)),
            )
            .on_input(Msg::TextChanged)
            .input_style(&["input"])
            .sizing(&["sz-xs"])
            .view(t);

        t.frame(
            t.column(&["subsection"])
                .push(t.text("Text Inputs", &["title-small"]))
                .push(t.text("Three sizes", &["label-small"]))
                .push(three_sizes)
                .push(t.text("States", &["label-small"]))
                .push(states)
                .push(t.text("With icons", &["label-small"]))
                .push(
                    t.row(&["row"])
                        .push(search_md)
                        .align_y(iced::Alignment::Center),
                )
                .push(
                    t.row(&["row"])
                        .push(search_sm)
                        .align_y(iced::Alignment::Center),
                )
                .push(
                    t.row(&["row"])
                        .push(search_xs)
                        .align_y(iced::Alignment::Center),
                ),
            &["section", "section-body"],
        )
        .width(Length::Fill)
    };

    // ── 3. Button Group ─────────────────────────────────────────────
    // Uses btn_group_row helper (custom widget, no builder equivalent).
    let btn_group = button_group_section(state);

    // ── 4. Control Group ────────────────────────────────────────────
    // Uses ControlGroup + Menu custom widgets — call existing section.
    let ctrl_group = control_group_section(state);

    // ── 5. Controls (checkboxes, togglers, radios) ──────────────────
    // Uses t.check(), t.toggle(), t.radio_btn() builders + protect().
    let controls: Element<'_, Msg> = {
        let chk_md = t.sizing(&["checkbox", "sz-md"]);
        let chk_sm = t.sizing(&["checkbox", "sz-sm"]);
        let chk_xs = t.sizing(&["checkbox", "sz-xs"]);

        let checks = t
            .row(&["row-loose"])
            .push(
                t.column(&["field-col"]).push(icss::widgets::protect(
                    &chk_md,
                    t.check(state.check_a, "Medium", &["checkbox", "sz-md"])
                        .on_toggle(Msg::CheckA),
                )),
            )
            .push(
                t.column(&["field-col"]).push(icss::widgets::protect(
                    &chk_sm,
                    t.check(state.check_b, "Small", &["checkbox", "sz-sm"])
                        .on_toggle(Msg::CheckB),
                )),
            )
            .push(
                t.column(&["field-col"]).push(icss::widgets::protect(
                    &chk_xs,
                    t.check(state.check_c, "Tiny", &["checkbox", "sz-xs"])
                        .on_toggle(Msg::CheckC),
                )),
            );

        let toggles = t
            .row(&["row-loose"])
            .push(
                t.column(&["field-col"]).push(icss::widgets::protect(
                    &chk_md,
                    t.toggle("Medium", state.toggle_a, &["toggle", "sz-md"])
                        .on_toggle(Msg::ToggleA),
                )),
            )
            .push(
                t.column(&["field-col"]).push(icss::widgets::protect(
                    &chk_sm,
                    t.toggle("Small", state.toggle_b, &["toggle", "sz-sm"])
                        .on_toggle(Msg::ToggleB),
                )),
            );

        let radios = t
            .row(&["row-loose"])
            .push(t.column(&["field-col"]).push(icss::widgets::protect(
                &chk_md,
                t.radio_btn(
                    "Medium",
                    RadioOpt::Alpha,
                    state.radio_choice,
                    Msg::RadioSelected,
                    &["radio", "sz-md"],
                ),
            )))
            .push(t.column(&["field-col"]).push(icss::widgets::protect(
                &chk_sm,
                t.radio_btn(
                    "Small",
                    RadioOpt::Beta,
                    state.radio_choice,
                    Msg::RadioSelected,
                    &["radio", "sz-sm"],
                ),
            )))
            .push(t.column(&["field-col"]).push(icss::widgets::protect(
                &chk_xs,
                t.radio_btn(
                    "Tiny",
                    RadioOpt::Gamma,
                    state.radio_choice,
                    Msg::RadioSelected,
                    &["radio", "sz-xs"],
                ),
            )));

        t.frame(
            t.column(&["subsection"])
                .push(t.text("Checkboxes, Togglers & Radios", &["title-small"]))
                .push(t.text("Checkboxes \u{2014} 3 sizes", &["label-small"]))
                .push(checks)
                .push(t.text("Togglers \u{2014} 2 sizes", &["label-small"]))
                .push(toggles)
                .push(t.text("Radios \u{2014} 3 sizes", &["label-small"]))
                .push(radios),
            &["section", "section-body"],
        )
        .width(Length::Fill)
        .into()
    };

    // ── 6. Sliders ──────────────────────────────────────────────────
    // Uses t.slide() builder.
    let sliders: Element<'_, Msg> = {
        let pct = (state.slider_value * 100.0) as u32;
        t.frame(
            t.column(&["subsection"])
                .push(t.text("Slider", &["title-small"]))
                .push(
                    t.column(&["field-col"])
                        .push(t.text(format!("Value: {pct}%"), &["label-small"]))
                        .push(
                            t.slide(
                                0.0..=1.0,
                                state.slider_value,
                                Msg::SliderChanged,
                                &["slider"],
                            )
                            .step(0.01),
                        ),
                ),
            &["section", "section-body"],
        )
        .width(Length::Fill)
        .into()
    };

    // ── 7. Progress Bars ────────────────────────────────────────────
    // Uses t.progress() builder.
    let progress: Element<'_, Msg> = {
        let bar_h = 5.0;
        t.frame(
            t.column(&["stack-tight"])
                .push(t.text("Progress Bars", &["title-small"]))
                .push(
                    t.column(&["field-col"])
                        .push(t.text("Default 40%", &["label-small"]))
                        .push(t.progress(0.0..=1.0, 0.4, &["progress"]).girth(bar_h)),
                )
                .push(
                    t.column(&["field-col"])
                        .push(t.text("Success 75%", &["label-small"]))
                        .push(
                            t.progress(0.0..=1.0, 0.75, &["progress", "success"])
                                .girth(bar_h),
                        ),
                )
                .push(
                    t.column(&["field-col"])
                        .push(t.text("Danger 90%", &["label-small"]))
                        .push(
                            t.progress(0.0..=1.0, 0.9, &["progress", "danger"])
                                .girth(bar_h),
                        ),
                )
                .push(
                    t.column(&["field-col"])
                        .push(t.text("Warning 55%", &["label-small"]))
                        .push(
                            t.progress(0.0..=1.0, 0.55, &["progress", "warning"])
                                .girth(bar_h),
                        ),
                ),
            &["section", "section-body"],
        )
        .width(Length::Fill)
        .into()
    };

    // ── 8. Pick List ────────────────────────────────────────────────
    // Uses t.select() builder + protect().
    let pick: Element<'_, Msg> = {
        let options = vec![
            "English".to_string(),
            "Spanish".to_string(),
            "French".to_string(),
            "German".to_string(),
            "Japanese".to_string(),
        ];
        let sz_md = t.sizing(&["select", "sz-md"]);
        let sz_sm = t.sizing(&["select", "sz-sm"]);
        let sz_xs = t.sizing(&["select", "sz-xs"]);

        t.frame(
            t.column(&["subsection"])
                .push(t.text("Pick List & Combo Box", &["title-small"]))
                .push(t.text("Pick List \u{2014} three sizes", &["label-small"]))
                .push(
                    t.row(&["row"])
                        .push(icss::widgets::protect(
                            &sz_md,
                            t.select(
                                options.clone(),
                                state.pick_choice.clone(),
                                Msg::PickSelected,
                                &["select", "sz-md"],
                            ),
                        ))
                        .push(icss::widgets::protect(
                            &sz_sm,
                            t.select(
                                options.clone(),
                                state.pick_choice.clone(),
                                Msg::PickSelected,
                                &["select", "sz-sm"],
                            ),
                        ))
                        .push(icss::widgets::protect(
                            &sz_xs,
                            t.select(
                                options,
                                state.pick_choice.clone(),
                                Msg::PickSelected,
                                &["select", "sz-xs"],
                            ),
                        ))
                        .align_y(iced::Alignment::Center),
                )
                .push(t.text("Combo Box (searchable)", &["label-small"]))
                .push(icss::widgets::protect(
                    &sz_md,
                    combo_box(
                        &state.combo_state,
                        "Search language...",
                        state.combo_value.as_ref(),
                        Msg::ComboSelected,
                    )
                    .input_style(t.text_input(&["input", "sz-md"]))
                    .menu_style(t.menu(&["select-menu", "sz-md"]))
                    .size(sz_md.font_size)
                    .padding([sz_md.pad_v, sz_md.pad_h]),
                )),
            &["section", "section-body"],
        )
        .width(Length::Fill)
        .into()
    };

    // ── 9. Editor + Tooltip ─────────────────────────────────────────
    // Uses t.editor() builder for the editor; tooltip section uses manual wiring.
    let editor_tooltip: Element<'_, Msg> = {
        let md = t.sizing(&["sz-md"]);
        let xs = t.sizing(&["sz-xs"]);

        let editor_col = t
            .column(&["field-col"])
            .push(t.text("Text Editor", &["title-small"]))
            .push(t.editor(
                &state.editor_content,
                Msg::EditorAction,
                &["editor", "sz-md"],
            ))
            .width(Length::Fill);

        let tooltip_col = t
            .column(&["field-col"])
            .push(t.text("Tooltip", &["title-small"]))
            .push(
                tooltip(
                    icss::widgets::protect(
                        &md,
                        t.btn("Hover me for tooltip", &["button", "primary", "sz-md"])
                            .on_press(Msg::Noop),
                    ),
                    container(text("This is a styled tooltip").size(xs.font_size))
                        .padding(xs.padding())
                        .style(t.tooltip(&["tooltip"])),
                    tooltip::Position::Bottom,
                )
                .gap(xs.gap),
            );

        t.frame(
            t.row(&["row-loose"]).push(editor_col).push(tooltip_col),
            &["section", "section-body"],
        )
        .width(Length::Fill)
        .into()
    };

    // ── 9b. Toasts ──────────────────────────────────────────────────
    let toasts: Element<'_, Msg> = {
        use icss::widgets::mdi;

        // Default toast — ghost button for close (normal surface)
        let default_close = &["button", "ghost", "sz-sm"];
        let default_toast = t
            .frame(
                t.row(&["toast"])
                    .push(
                        t.column(&["toast"])
                            .push(t.text("File received", &["label-large"]))
                            .push(t.text(
                                "presentation.pdf was saved to Downloads",
                                &["body-medium", "text-default"],
                            ))
                            .width(Length::Fill),
                    )
                    .push(
                        t.btn_sq(mdi_icon(t, mdi::X, default_close), default_close)
                            .on_press(Msg::Noop),
                    )
                    .align_y(iced::Alignment::Start),
                &["toast"],
            )
            .width(320);

        // Primary toast — ghost-overlay button (opacity-based, for colored surface)
        let primary_close = &["button", "ghost-overlay", "sz-sm"];
        let primary_toast = t
            .frame(
                t.row(&["toast"])
                    .push(
                        t.column(&["toast"])
                            .push(t.text(
                                "Connection established",
                                &["label-large", "text-on-primary"],
                            ))
                            .push(t.text(
                                "You are now connected to dev-station",
                                &["body-medium", "text-on-primary-soft"],
                            ))
                            .width(Length::Fill),
                    )
                    .push(
                        t.btn_sq(mdi_icon(t, mdi::X, primary_close), primary_close)
                            .on_press(Msg::Noop),
                    )
                    .align_y(iced::Alignment::Start),
                &["toast-primary"],
            )
            .width(320);

        t.frame(
            t.column(&["subsection"])
                .push(t.text("Toasts", &["title-small"]))
                .push(t.text("Default (surface-s0)", &["label-small"]))
                .push(default_toast)
                .push(t.text("Primary (surface-primary-s0)", &["label-small"]))
                .push(primary_toast),
            &["section", "section-body"],
        )
        .width(Length::Fill)
        .into()
    };

    // ── 10. Chat Textarea ───────────────────────────────────────────
    // Complex custom widget — call existing section function.
    let chat_textarea = chat_textarea_section(state);

    // ── 11. Tile Grid ───────────────────────────────────────────────
    // Custom TileGrid widget — call existing section function.
    let tile_grid = tile_grid_section(state);

    // ── 12. Data Table ──────────────────────────────────────────────
    // Custom DataTable widget — call existing section function.
    let data_table = data_table_section(state);

    // ── 13. Typography ──────────────────────────────────────────────
    // No interactive controls — call existing section function.
    let typography = typography_section(state);

    // ── 14. Text Colors ─────────────────────────────────────────────
    // No interactive controls — call existing section function.
    let text_colors = text_colors_section(state);

    // ── Assemble page (same order as components_page) ───────────────
    let page = column![
        btn_section,
        t.divider(1.0, &["divider"]),
        input_section,
        t.divider(1.0, &["divider"]),
        btn_group,
        t.divider(1.0, &["divider"]),
        ctrl_group,
        t.divider(1.0, &["divider"]),
        controls,
        t.divider(1.0, &["divider"]),
        sliders,
        t.divider(1.0, &["divider"]),
        progress,
        t.divider(1.0, &["divider"]),
        pick,
        t.divider(1.0, &["divider"]),
        editor_tooltip,
        t.divider(1.0, &["divider"]),
        toasts,
        t.divider(1.0, &["divider"]),
        chat_textarea,
        t.divider(1.0, &["divider"]),
        tile_grid,
        t.divider(1.0, &["divider"]),
        data_table,
        t.divider(1.0, &["divider"]),
        typography,
        t.divider(1.0, &["divider"]),
        text_colors,
    ]
    .spacing(d.space_250);

    scrollable(container(page).padding(d.space_200).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

fn primitives_page(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let d = &state.dims;
    let sb = state
        .vars
        .font_family
        .weighted(iced::font::Weight::Semibold);

    // Helper: labeled row of swatches (no step numbers)
    let color_group =
        |label: &str, items: Vec<(&'static str, &'static [&'static str])>| -> Element<'_, Msg> {
            let swatches: Vec<Element<'_, Msg>> = items
                .into_iter()
                .map(|(name, cls)| prim_swatch(t, d, name, cls))
                .collect();
            column![
                text(label.to_owned()).size(d.font_label_small),
                Row::with_children(swatches).spacing(d.space_25),
            ]
            .spacing(2)
            .into()
        };

    // Helper: labeled row of swatches with step numbers
    let color_group_steps = |label: &str,
                             items: Vec<(&'static str, &'static [&'static str])>,
                             steps: &[usize]|
     -> Element<'_, Msg> {
        let swatches: Vec<Element<'_, Msg>> = items
            .into_iter()
            .enumerate()
            .map(|(i, (name, cls))| {
                let step_label = steps.get(i).map(|s| format!("{s}")).unwrap_or_default();
                prim_swatch_step(t, d, name, cls, &step_label)
            })
            .collect();
        column![
            text(label.to_owned()).size(d.font_label_small),
            Row::with_children(swatches).spacing(d.space_25),
        ]
        .spacing(2)
        .into()
    };

    // Look up step indices for a family by name
    let find_steps = |name: &str| -> Option<&icss::engine::semantic::SurfaceSteps> {
        state
            .family_steps
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, s)| s)
    };

    // Helper: builds a full family swatch group (surfaces + on-text + the
    // family's own outlines) with step labels on the first two rows.
    let family_swatches_fn = |prefix: &str,
                              surface_items: Vec<(&'static str, &'static [&'static str])>,
                              text_items: Vec<(&'static str, &'static [&'static str])>,
                              outline_items: Vec<(&'static str, &'static [&'static str])>|
     -> Element<'_, Msg> {
        let surface_label = format!("surface-{prefix}");
        let text_label = format!("on-surface-{prefix}");
        let outline_label = format!("outline-{prefix}");
        let steps = find_steps(&surface_label);
        let surface_steps: &[usize] = steps.map(|s| &s.surface[..]).unwrap_or(&[]);
        let text_steps: &[usize] = steps.map(|s| &s.text[..]).unwrap_or(&[]);
        column![
            color_group_steps(&surface_label, surface_items, surface_steps),
            color_group_steps(&text_label, text_items, text_steps),
            color_group(&outline_label, outline_items),
        ]
        .spacing(d.space_50)
        .into()
    };

    macro_rules! family_swatches {
        ($label:expr, $prefix:literal) => {
            family_swatches_fn(
                $prefix,
                vec![
                    ("s0", &[concat!("sw-surface-", $prefix, "-0")][..]),
                    ("s1", &[concat!("sw-surface-", $prefix, "-1")]),
                    ("s2", &[concat!("sw-surface-", $prefix, "-2")]),
                    ("s3", &[concat!("sw-surface-", $prefix, "-3")]),
                    ("s4", &[concat!("sw-surface-", $prefix, "-4")]),
                ],
                vec![
                    ("text", &[concat!("sw-on-surface-", $prefix)][..]),
                    ("default", &[concat!("sw-on-surface-", $prefix, "-default")]),
                    ("soft", &[concat!("sw-on-surface-", $prefix, "-soft")]),
                    ("muted", &[concat!("sw-on-surface-", $prefix, "-muted")]),
                    (
                        "disabled",
                        &[concat!("sw-on-surface-", $prefix, "-disabled")],
                    ),
                    ("faint", &[concat!("sw-on-surface-", $prefix, "-faint")]),
                ],
                vec![
                    ("subtle", &[concat!("sw-outline-", $prefix, "-subtle")][..]),
                    ("soft", &[concat!("sw-outline-", $prefix, "-soft")]),
                    ("middle", &[concat!("sw-outline-", $prefix, "-middle")]),
                    ("strong", &[concat!("sw-outline-", $prefix, "-strong")]),
                    ("heavy", &[concat!("sw-outline-", $prefix, "-heavy")]),
                    ("solid", &[concat!("sw-outline-", $prefix, "-solid")]),
                ],
            )
        };
    }

    // ── Colors ──
    let colors_section = column![
        text("Colors").size(d.font_title_small).font(sb),
        // Neutral surface family (special: has s5)
        {
            let srf_steps = find_steps("surface");
            let mut s_steps: Vec<usize> = srf_steps.map(|s| s.surface.to_vec()).unwrap_or_default();
            // s5 = s4 + step_size (approximate for display)
            if let Some(last) = s_steps.last().copied() {
                s_steps.push(last + 3);
            }
            let t_steps: Vec<usize> = srf_steps.map(|s| s.text.to_vec()).unwrap_or_default();
            let c: Element<'_, Msg> = column![
                color_group_steps(
                    "surface",
                    vec![
                        ("s0", &["sw-surface-0"][..]),
                        ("s1", &["sw-surface-1"]),
                        ("s2", &["sw-surface-2"]),
                        ("s3", &["sw-surface-3"]),
                        ("s4", &["sw-surface-4"]),
                        ("s5", &["sw-surface-5"]),
                    ],
                    &s_steps
                ),
                color_group_steps(
                    "on-surface",
                    vec![
                        ("text", &["sw-on-surface"][..]),
                        ("default", &["sw-on-surface-default"]),
                        ("soft", &["sw-on-surface-soft"]),
                        ("muted", &["sw-on-surface-muted"]),
                        ("disabled", &["sw-on-surface-disabled"]),
                        ("faint", &["sw-on-surface-faint"]),
                    ],
                    &t_steps
                ),
            ]
            .spacing(d.space_50)
            .into();
            c
        },
        color_group(
            "outline",
            vec![
                ("subtle", &["sw-outline-subtle"]),
                ("soft", &["sw-outline-soft"]),
                ("middle", &["sw-outline-middle"]),
                ("strong", &["sw-outline-strong"]),
                ("heavy", &["sw-outline-heavy"]),
                ("solid", &["sw-outline-solid"]),
            ]
        ),
        rule::horizontal(1).style(t.rule(&["divider"])),
        // Variant neutral surfaces
        family_swatches!("Tint", "tint"),
        family_swatches!("Dark tint", "dark-tint"),
        family_swatches!("Black", "black"),
        rule::horizontal(1).style(t.rule(&["divider"])),
        // Chromatic families
        family_swatches!("Primary", "primary"),
        family_swatches!("Primary container", "primary-container"),
        family_swatches!("Secondary", "secondary"),
        family_swatches!("Secondary container", "secondary-container"),
        family_swatches!("Tertiary", "tertiary"),
        family_swatches!("Tertiary container", "tertiary-container"),
        family_swatches!("Quaternary", "quaternary"),
        family_swatches!("Quaternary container", "quaternary-container"),
        rule::horizontal(1).style(t.rule(&["divider"])),
        // Signal families
        family_swatches!("Success", "success"),
        family_swatches!("Success container", "success-container"),
        family_swatches!("Danger", "danger"),
        family_swatches!("Danger container", "danger-container"),
        family_swatches!("Warning", "warning"),
        family_swatches!("Warning container", "warning-container"),
        rule::horizontal(1).style(t.rule(&["divider"])),
        // Accent on neutral surfaces
        color_group(
            "on-surface-accent",
            vec![
                ("primary", &["sw-accent-primary"]),
                ("secondary", &["sw-accent-secondary"]),
                ("tertiary", &["sw-accent-tertiary"]),
                ("quaternary", &["sw-accent-quaternary"]),
                ("link", &["sw-accent-link"]),
                ("success", &["sw-accent-success"]),
                ("danger", &["sw-accent-danger"]),
                ("warning", &["sw-accent-warning"]),
            ]
        ),
        // Shadow colors (as swatches)
        color_group(
            "shadow",
            vec![
                ("soft", &["sw-shadow-soft"]),
                ("medium", &["sw-shadow-medium"]),
                ("default", &["sw-shadow"]),
            ]
        ),
        rule::horizontal(1).style(t.rule(&["divider"])),
        color_wheel_section(state),
        closest_path_section(state),
    ]
    .spacing(d.space_75);

    // ── Shadows (rendered) ──
    let shadows_section = column![
        text("Shadows").size(d.font_title_small).font(sb),
        text("1px no blur").size(d.font_label_small),
        row![
            prim_shadow(t, d, "soft", &["sh-1px-soft"]),
            prim_shadow(t, d, "medium", &["sh-1px-medium"]),
            prim_shadow(t, d, "default", &["sh-1px-default"]),
        ]
        .spacing(d.space_150),
        text("1px with blur").size(d.font_label_small),
        row![
            prim_shadow(t, d, "soft", &["shadow-soft-demo"]),
            prim_shadow(t, d, "medium", &["shadow-medium-demo"]),
            prim_shadow(t, d, "default", &["shadow-demo"]),
        ]
        .spacing(d.space_150),
        text("elevated — soft").size(d.font_label_small),
        row![
            prim_shadow(t, d, "sp-50", &["sh-elevated-50-soft"]),
            prim_shadow(t, d, "sp-75", &["sh-elevated-75-soft"]),
            prim_shadow(t, d, "sp-100", &["sh-elevated-100-soft"]),
            prim_shadow(t, d, "sp-150", &["sh-elevated-150-soft"]),
            prim_shadow(t, d, "sp-200", &["sh-elevated-200-soft"]),
        ]
        .spacing(d.space_150),
        text("elevated — medium").size(d.font_label_small),
        row![
            prim_shadow(t, d, "sp-50", &["sh-elevated-50-medium"]),
            prim_shadow(t, d, "sp-75", &["sh-elevated-75-medium"]),
            prim_shadow(t, d, "sp-100", &["sh-elevated-100-medium"]),
            prim_shadow(t, d, "sp-150", &["sh-elevated-150-medium"]),
            prim_shadow(t, d, "sp-200", &["sh-elevated-200-medium"]),
        ]
        .spacing(d.space_150),
        text("elevated — default").size(d.font_label_small),
        row![
            prim_shadow(t, d, "sp-50", &["sh-elevated-50-default"]),
            prim_shadow(t, d, "sp-75", &["sh-elevated-75-default"]),
            prim_shadow(t, d, "sp-100", &["sh-elevated-100-default"]),
            prim_shadow(t, d, "sp-150", &["sh-elevated-150-default"]),
            prim_shadow(t, d, "sp-200", &["sh-elevated-200-default"]),
        ]
        .spacing(d.space_150),
    ]
    .spacing(d.space_100);

    // ── Typography ──
    // Base every weight variant on the user-selected family so Bold /
    // Medium / Light all resolve within that typeface rather than falling
    // back to iced's Font::DEFAULT (Fira Sans), which substitutes a mono
    // fallback for weights it doesn't carry.
    let regular = state.vars.font_family.weighted(iced::font::Weight::Normal);
    let hn = state.vars.font_family.weighted(iced::font::Weight::Light);
    let bold = state.vars.font_family.weighted(iced::font::Weight::Bold);
    let semi_light = state.vars.font_family.weighted(iced::font::Weight::Medium);

    let fonts_section = column![
        text("Typography").size(d.font_title_small).font(sb),
        text("headline (300/400)").size(d.font_label_small),
        Row::with_children(vec![
            prim_type_sample(t, d, "xxxl", d.font_headline_xxxlarge, regular),
            prim_type_sample(t, d, "xxl", d.font_headline_xxlarge, regular),
            prim_type_sample(t, d, "xl", d.font_headline_xlarge, hn),
            prim_type_sample(t, d, "large", d.font_headline_large, hn),
            prim_type_sample(t, d, "medium", d.font_headline_medium, hn),
            prim_type_sample(t, d, "small", d.font_headline_small, hn),
            prim_type_sample(t, d, "micro", d.font_headline_micro, hn),
        ])
        .spacing(d.space_50)
        .wrap(),
        text("title (600 semibold)").size(d.font_label_small),
        Row::with_children(vec![
            prim_type_sample(t, d, "large", d.font_title_large, sb),
            prim_type_sample(t, d, "medium", d.font_title_medium, sb),
            prim_type_sample(t, d, "small", d.font_title_small, sb),
        ])
        .spacing(d.space_50)
        .wrap(),
        text("label (600 semibold)").size(d.font_label_small),
        Row::with_children(vec![
            prim_type_sample(t, d, "large", d.font_label_large, sb),
            prim_type_sample(t, d, "medium", d.font_label_medium, sb),
            prim_type_sample(t, d, "small", d.font_label_small, sb),
            prim_type_sample(t, d, "micro", d.font_label_micro, sb),
        ])
        .spacing(d.space_50)
        .wrap(),
        text("body (400 regular)").size(d.font_label_small),
        Row::with_children(vec![
            prim_type_sample(t, d, "large", d.font_body_large, regular),
            prim_type_sample(t, d, "medium", d.font_body_medium, regular),
            prim_type_sample(t, d, "med-sm", d.font_body_medium_small, regular),
            prim_type_sample(t, d, "small", d.font_body_small, regular),
            prim_type_sample(t, d, "micro", d.font_body_micro, regular),
        ])
        .spacing(d.space_50)
        .wrap(),
        text("font weight").size(d.font_label_small),
        Row::with_children(vec![
            prim_type_sample(t, d, "Regular", d.font_label_large, regular),
            prim_type_sample(t, d, "Semibold", d.font_label_large, sb),
            prim_type_sample(t, d, "Bold", d.font_label_large, bold),
            prim_type_sample(t, d, "Semilight", d.font_label_large, semi_light),
            prim_type_sample(t, d, "Light", d.font_label_large, hn),
        ])
        .spacing(d.space_50)
        .wrap(),
    ]
    .spacing(d.space_75);

    // ── Sizes & Spacings ──
    let spacing_section = column![
        text("Sizes & Spacings").size(d.font_title_small).font(sb),
        text("spacings").size(d.font_label_small),
        prim_space_bar(t, d, "sp-25", d.space_25),
        prim_space_bar(t, d, "sp-50", d.space_50),
        prim_space_bar(t, d, "sp-75", d.space_75),
        prim_space_bar(t, d, "sp-100", d.space_100),
        prim_space_bar(t, d, "sp-150", d.space_150),
        prim_space_bar(t, d, "sp-200", d.space_200),
        prim_space_bar(t, d, "sp-250", d.space_250),
        prim_space_bar(t, d, "sp-300", d.space_300),
        prim_space_bar(t, d, "sp-400", d.space_400),
        text("radii").size(d.font_label_small),
        row![
            radius_demo(t, d, "r-25", d.radius_25),
            radius_demo(t, d, "r-50", d.radius_50),
            radius_demo(t, d, "r-75", d.radius_75),
            radius_demo(t, d, "r-100", d.radius_100),
            radius_demo(t, d, "r-150", d.radius_150),
            radius_demo(t, d, "r-inf", d.radius_infinite.min(24.0)),
        ]
        .spacing(d.space_75),
    ]
    .spacing(d.space_75);

    // ── Animations ──
    let anim_text_base = t.color_var("text").unwrap_or(iced::Color::WHITE);
    let fade_alpha = state.anim_fade.alpha();
    let fade_label = if state.anim_fade.is_running() {
        format!("alpha: {:.2}", fade_alpha)
    } else if fade_alpha > 0.5 {
        "visible (click to fade out)".into()
    } else {
        "hidden (click to fade in)".into()
    };

    let atb = anim_text_base;
    let fade_demo: Element<'_, Msg> = container(
        text("Fade")
            .size(d.font_label_medium)
            .color(iced::Color::from_rgba(atb.r, atb.g, atb.b, fade_alpha)),
    )
    .padding([d.space_100, d.space_200])
    .style(move |_theme: &IcedTheme| iced::widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            atb.r,
            atb.g,
            atb.b,
            0.08 * fade_alpha,
        ))),
        border: iced::Border {
            radius: d.radius_75.into(),
            width: 1.0,
            color: iced::Color::from_rgba(atb.r, atb.g, atb.b, 0.15 * fade_alpha),
        },
        ..Default::default()
    })
    .into();

    let slide_demo =
        |label: &'static str, edge: icss::widgets::Edge, anim: &Animation| -> Element<'_, Msg> {
            let offset = anim.offset();
            let slide_dist = 60.0;
            let px = offset * slide_dist;
            let visible = slide_dist - px;
            let padding = match edge {
                icss::widgets::Edge::Left => Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: visible,
                },
                icss::widgets::Edge::Right => Padding {
                    top: 0.0,
                    right: visible,
                    bottom: 0.0,
                    left: 0.0,
                },
                icss::widgets::Edge::Top => Padding {
                    top: visible,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                },
                icss::widgets::Edge::Bottom => Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: visible,
                    left: 0.0,
                },
            };
            let alpha = 1.0 - offset;
            button(
                container(
                    text(label)
                        .size(d.font_label_medium)
                        .color(iced::Color::from_rgba(atb.r, atb.g, atb.b, alpha)),
                )
                .padding(padding),
            )
            .on_press(Msg::AnimSlide(edge))
            .style(t.button(&["button", "ghost"]))
            .into()
        };

    let animations_section = column![
        text("Animations").size(d.font_title_small).font(sb),
        text("fade in/out (0.5s ease-in-out)").size(d.font_label_small),
        row![
            button(fade_demo)
                .on_press(Msg::AnimFadeToggle)
                .style(t.button(&["button", "ghost"])),
            text(fade_label).size(d.font_label_micro),
        ]
        .spacing(d.space_100)
        .align_y(iced::Alignment::Center),
        text("slide from edge (0.5s ease-in-out)").size(d.font_label_small),
        row![
            slide_demo("Left", icss::widgets::Edge::Left, &state.anim_slide_left),
            slide_demo("Top", icss::widgets::Edge::Top, &state.anim_slide_top),
            slide_demo("Right", icss::widgets::Edge::Right, &state.anim_slide_right),
            slide_demo(
                "Bottom",
                icss::widgets::Edge::Bottom,
                &state.anim_slide_bottom
            ),
        ]
        .spacing(d.space_100),
    ]
    .spacing(d.space_75);

    column![
        colors_section,
        rule::horizontal(1).style(t.rule(&["divider"])),
        shadows_section,
        rule::horizontal(1).style(t.rule(&["divider"])),
        fonts_section,
        rule::horizontal(1).style(t.rule(&["divider"])),
        spacing_section,
        rule::horizontal(1).style(t.rule(&["divider"])),
        animations_section,
    ]
    .spacing(d.space_250)
    .into()
}

fn btn_category<'a>(
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    sb: Font,
    md: &icss::theme::resolve::sizing::ComponentSize,
    sm: &icss::theme::resolve::sizing::ComponentSize,
    xs: &icss::theme::resolve::sizing::ComponentSize,
    sq_md: f32,
    sq_sm: f32,
    sq_xs: f32,
    label: &'a str,
    base_classes: &'a [&'a str],
    disabled: bool,
) -> Element<'a, Msg> {
    let press = if disabled { None } else { Some(Msg::Noop) };

    // Per-size class lists must outlive the button style closures (which
    // borrow for 'a). A locally-built Vec cannot; so pick fully-static
    // slices based on the variant found in base_classes. Per-size
    // border-radius falls out of the compound selectors in compose.rs
    // (.button.sz-md → r-100, etc.).
    let variant: &str = base_classes
        .iter()
        .copied()
        .find(|c| {
            matches!(
                *c,
                "primary" | "success" | "danger" | "warning" | "default" | "ghost" | "outlined"
            )
        })
        .unwrap_or("default");
    let (cls_md, cls_sm, cls_xs): (
        &'static [&'static str],
        &'static [&'static str],
        &'static [&'static str],
    ) = match variant {
        "primary" => (
            &["button", "primary", "sz-md"],
            &["button", "primary", "sz-sm"],
            &["button", "primary", "sz-xs"],
        ),
        "success" => (
            &["button", "success", "sz-md"],
            &["button", "success", "sz-sm"],
            &["button", "success", "sz-xs"],
        ),
        "danger" => (
            &["button", "danger", "sz-md"],
            &["button", "danger", "sz-sm"],
            &["button", "danger", "sz-xs"],
        ),
        "warning" => (
            &["button", "warning", "sz-md"],
            &["button", "warning", "sz-sm"],
            &["button", "warning", "sz-xs"],
        ),
        "default" => (
            &["button", "default", "sz-md"],
            &["button", "default", "sz-sm"],
            &["button", "default", "sz-xs"],
        ),
        "ghost" => (
            &["button", "ghost", "sz-md"],
            &["button", "ghost", "sz-sm"],
            &["button", "ghost", "sz-xs"],
        ),
        "outlined" => (
            &["button", "outlined", "sz-md"],
            &["button", "outlined", "sz-sm"],
            &["button", "outlined", "sz-xs"],
        ),
        _ => (
            &["button", "sz-md"],
            &["button", "sz-sm"],
            &["button", "sz-xs"],
        ),
    };

    // Icon color from the resolved button style's accent-color property.
    let _icon_tint = {
        let cs = t.resolve(base_classes, None);
        Theme::resolve_color(&cs, "accent-color")
            .map(|c| c.to_iced())
            .unwrap_or(iced::Color::WHITE)
    };

    // Debug: solid red square at font_size × font_size — shows exact icon bounds.
    let icon_box = |sz: &icss::theme::resolve::sizing::ComponentSize| -> Element<'a, Msg> {
        let s = sz.font_size;
        container(text(""))
            .width(s)
            .height(s)
            .style(|_: &_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    1.0, 0.0, 0.0,
                ))),
                ..Default::default()
            })
            .into()
    };

    // icon+label button
    let il_btn = |lbl: &'a str,
                  sz: &icss::theme::resolve::sizing::ComponentSize,
                  cls: &'a [&'a str]|
     -> Element<'a, Msg> {
        let lh = iced::widget::text::LineHeight::Absolute(iced::Pixels(sz.font_size));
        let content = container(
            row![
                icon_box(sz),
                text(lbl).size(sz.font_size).line_height(lh).font(sb),
            ]
            .spacing(sz.gap)
            .align_y(iced::Alignment::Center),
        )
        .center_x(Length::Shrink);
        let mut btn = button(content).padding(sz.padding()).style(t.button(cls));
        if let Some(ref msg) = press {
            btn = btn.on_press(msg.clone());
        }
        icss::widgets::protect(sz, btn)
    };

    // icon-only square button (fixed size — red debug square as icon)
    let sq_btn = |sq_size: f32, cls: &'a [&'a str]| -> Element<'a, Msg> {
        let icon_sz = 12.0;
        let icon: Element<'a, Msg> = container(text(""))
            .width(icon_sz)
            .height(icon_sz)
            .style(|_: &_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    1.0, 0.0, 0.0,
                ))),
                ..Default::default()
            })
            .into();
        let content = container(icon)
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        let mut btn = button(content)
            .padding(Padding::ZERO)
            .width(sq_size)
            .height(sq_size)
            .style(t.button(cls));
        if let Some(ref msg) = press {
            btn = btn.on_press(msg.clone());
        }
        btn.into()
    };

    // text-only button (no icon)
    let txt_btn = |lbl: &'a str,
                   sz: &icss::theme::resolve::sizing::ComponentSize,
                   cls: &'a [&'a str]|
     -> Element<'a, Msg> {
        let lh = iced::widget::text::LineHeight::Absolute(iced::Pixels(sz.font_size));
        let mut btn = button(text(lbl).size(sz.font_size).line_height(lh).font(sb))
            .padding(sz.padding())
            .style(t.button(cls));
        if let Some(ref msg) = press {
            btn = btn.on_press(msg.clone());
        }
        icss::widgets::protect(sz, btn)
    };

    column![
        text(label).size(d.font_label_small),
        // text-only: md, sm, xs
        row![
            txt_btn("Label", md, cls_md),
            txt_btn("Label", sm, cls_sm),
            txt_btn("Label", xs, cls_xs),
        ]
        .spacing(d.space_75)
        .align_y(iced::Alignment::Center),
        // icon+label: md, sm, xs
        row![
            il_btn("Label", md, cls_md),
            il_btn("Label", sm, cls_sm),
            il_btn("Label", xs, cls_xs),
        ]
        .spacing(d.space_75)
        .align_y(iced::Alignment::Center),
        // icon-only: md, sm, xs
        row![
            sq_btn(sq_md, cls_md),
            sq_btn(sq_sm, cls_sm),
            sq_btn(sq_xs, cls_xs),
        ]
        .spacing(d.space_75)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(d.space_50)
    .into()
}

/// Create a tinted MDI SVG icon sized from ICSS, colored from button's `color` property.
fn mdi_icon<'a>(
    t: &'a Theme,
    icon: icss::widgets::mdi::IconData,
    btn_classes: &[&str],
) -> iced::widget::Svg<'a, IcedTheme> {
    use iced::widget::svg;
    use icss::widgets::mdi;
    let sz = t.sizing(btn_classes);
    let c = {
        let computed = t.resolve(btn_classes, None);
        Theme::resolve_color(&computed, "color")
            .map(|c| c.to_iced())
            .unwrap_or(iced::Color::WHITE)
    };
    svg(mdi::icon_handle(icon))
        .width(sz.icon_size)
        .height(sz.icon_size)
        .style(move |_, _| iced::widget::svg::Style { color: Some(c) })
}

fn btn_group_row<'a>(
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    _sb: Font,
    labels: &[&'a str],
    active: usize,
    size_classes: &'a [&'a str],
) -> Element<'a, Msg> {
    let r = d.radius_75;
    let count = labels.len();
    let outline_color = t
        .color_var("outline-subtle")
        .unwrap_or(iced::Color::TRANSPARENT);

    let mut group = row![].spacing(0);
    for (i, label) in labels.iter().enumerate() {
        let is_active = i == active;
        let active_fn = t.button(&["button", "primary"]);
        let default_fn = t.button(&["button", "default"]);

        // Position-based radius
        let (tl, tr, bl, br) = if count == 1 {
            (r, r, r, r)
        } else if i == 0 {
            (r, 0.0, r, 0.0)
        } else if i == count - 1 {
            (0.0, r, 0.0, r)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        let oc = outline_color;

        // Use t.btn() for content/padding/sizing, then override style
        // with custom per-position radius logic.
        group = group.push(
            t.btn(*label, size_classes)
                .on_press(Msg::BtnGroupChanged(i))
                .style(move |iced_theme: &IcedTheme, status| {
                    let mut s = if is_active {
                        active_fn(iced_theme, status)
                    } else {
                        let mut ds = default_fn(iced_theme, status);
                        ds.border.color = oc;
                        ds
                    };
                    s.border.radius = iced::border::Radius {
                        top_left: tl,
                        top_right: tr,
                        bottom_left: bl,
                        bottom_right: br,
                    };
                    s
                }),
        );
    }

    group.into()
}

fn prim_space_bar<'a>(
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    label: &'a str,
    width: f32,
) -> Element<'a, Msg> {
    row![
        text(label).size(d.font_label_micro).width(60),
        container("")
            .width(Length::Fixed(width))
            .height(d.space_100)
            .style(t.container(&["primary-swatch"])),
        text(format!("{}px", width)).size(d.font_label_micro),
    ]
    .spacing(d.space_50)
    .align_y(iced::Alignment::Center)
    .into()
}

fn prim_type_sample<'a>(
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    label: &'a str,
    size: f32,
    font: Font,
) -> Element<'a, Msg> {
    container(text(label).size(size).font(font))
        .padding([d.space_25, d.space_75])
        .style(t.container(&["section"]))
        .into()
}

fn prim_swatch<'a>(
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    label: &'a str,
    classes: &'a [&'a str],
) -> Element<'a, Msg> {
    column![
        container("")
            .width(48)
            .height(48)
            .style(t.container(classes)),
        text(label).size(d.font_label_micro),
    ]
    .spacing(2)
    .align_x(iced::Alignment::Center)
    .into()
}

fn prim_swatch_step<'a>(
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    label: &'a str,
    classes: &'a [&'a str],
    step: &str,
) -> Element<'a, Msg> {
    let mut col = column![
        container("")
            .width(48)
            .height(48)
            .style(t.container(classes)),
        text(label).size(d.font_label_micro),
    ]
    .spacing(2)
    .align_x(iced::Alignment::Center);
    if !step.is_empty() {
        col = col.push(
            text(step.to_owned())
                .size(d.font_label_micro)
                .color(t.color_var("text-default").unwrap_or(iced::Color::WHITE)),
        );
    }
    col.into()
}

fn prim_shadow<'a>(
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    label: &'a str,
    classes: &'a [&'a str],
) -> Element<'a, Msg> {
    column![
        container("")
            .width(80)
            .height(48)
            .style(t.container(classes)),
        text(label).size(d.font_label_micro),
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center)
    .into()
}

fn radius_demo<'a>(
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    label: &'a str,
    radius: f32,
) -> Element<'a, Msg> {
    // Use radius-demo class for border styling, then override border-radius per sample
    let base_style = t.container(&["radius-demo"]);
    column![
        container("")
            .width(48)
            .height(48)
            .style(move |theme: &IcedTheme| {
                let mut s = base_style(theme);
                s.border.radius = radius.into();
                s
            }),
        text(label).size(d.font_label_micro),
    ]
    .spacing(2)
    .align_x(iced::Alignment::Center)
    .into()
}

/// Label badge for a control ID.
fn id<'a>(label: &str, t: &Theme) -> Element<'a, Msg> {
    t.text(label.to_owned(), &["label-micro", "text-faint"])
        .into()
}

// ── Sections ──

fn buttons_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let d = &state.dims;
    use iced::font::Weight;

    let sb = state.vars.font_family.weighted(Weight::Semibold);

    let md = t.sizing(&["sz-md"]);
    let sm = t.sizing(&["sz-sm"]);
    let xs = t.sizing(&["sz-xs"]);

    let line_h = 1.3_f32;
    let sq_md = (md.font_size * line_h).ceil() + 2.0 * md.pad_v;
    let sq_sm = (sm.font_size * line_h).ceil() + 2.0 * sm.pad_v;
    let sq_xs = (xs.font_size * line_h).ceil() + 2.0 * xs.pad_v;

    // Icon + label helper (used in button catalog below)
    #[allow(unused_macros)]
    macro_rules! il {
        ($icon:expr, $label:expr, $sz:ident) => {
            container(
                row![
                    $icon.size($sz.font_size),
                    text($label).size($sz.font_size).font(sb)
                ]
                .spacing($sz.gap)
                .align_y(iced::Alignment::Center),
            )
            .center_x(Length::Fill)
        };
    }
    // Square icon helper (used in button catalog below)
    #[allow(unused_macros)]
    macro_rules! sq {
        ($icon:expr, $sz:ident) => {
            container($icon.size($sz.font_size))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        };
    }

    let dis = state.buttons_disabled;
    let primary_cat = btn_category(
        t,
        d,
        sb,
        &md,
        &sm,
        &xs,
        sq_md,
        sq_sm,
        sq_xs,
        "Primary",
        &["button", "primary"],
        dis,
    );
    let success_cat = btn_category(
        t,
        d,
        sb,
        &md,
        &sm,
        &xs,
        sq_md,
        sq_sm,
        sq_xs,
        "Success",
        &["button", "success"],
        dis,
    );
    let danger_cat = btn_category(
        t,
        d,
        sb,
        &md,
        &sm,
        &xs,
        sq_md,
        sq_sm,
        sq_xs,
        "Danger",
        &["button", "danger"],
        dis,
    );
    let default_cat = btn_category(
        t,
        d,
        sb,
        &md,
        &sm,
        &xs,
        sq_md,
        sq_sm,
        sq_xs,
        "Default",
        &["button", "default"],
        dis,
    );
    let ghost_cat = btn_category(
        t,
        d,
        sb,
        &md,
        &sm,
        &xs,
        sq_md,
        sq_sm,
        sq_xs,
        "Ghost",
        &["button", "ghost"],
        dis,
    );
    let outlined_cat = btn_category(
        t,
        d,
        sb,
        &md,
        &sm,
        &xs,
        sq_md,
        sq_sm,
        sq_xs,
        "Outlined",
        &["button", "outlined"],
        dis,
    );

    // ── Emphasized (gradient outline) — primary only (WIP) ──
    let cv = |name: &str| t.color_var(name).unwrap_or(iced::Color::WHITE);
    let c_cont_s4 = cv("surface-primary-container-s4");
    let c_s0 = cv("surface-primary-s0");
    let c_s1 = cv("surface-primary-s1");
    let c_s2 = cv("surface-primary-s2");
    let c_s3 = cv("surface-primary-s3");
    let c_on = cv("on-surface-primary");
    let pi = std::f32::consts::PI;

    let hover_idx = state.gradient_hover;
    let press_idx = state.gradient_pressed;
    let mut emph_counter: usize = 0;

    // AD-HOC: gradient buttons require inline container::Style because ICSS
    // has no gradient support. Colors come from theme vars, only the
    // Background::Gradient construction is custom.
    let mut emph_btn = |label: &'static str,
                        sz: &icss::theme::resolve::sizing::ComponentSize,
                        radius: f32,
                        icon: Option<iced::widget::Text<'static>>,
                        sq_size: Option<f32>|
     -> Element<'_, Msg> {
        let idx = emph_counter;
        emph_counter += 1;
        let hovered = hover_idx == Some(idx);
        let pressed = press_idx == Some(idx);
        let content: Element<'_, Msg> = match (icon, sq_size, label.is_empty()) {
            (Some(ic), Some(sq), _) => container(ic.size(sz.font_size).color(c_on))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(sq - 4.0)
                .height(sq - 4.0)
                .into(),
            (Some(ic), None, _) => container(
                row![
                    ic.size(sz.font_size).color(c_on),
                    text(label).size(sz.font_size).font(sb).color(c_on)
                ]
                .spacing(sz.gap)
                .align_y(iced::Alignment::Center),
            )
            .center_x(Length::Shrink)
            .into(),
            _ => text(label).size(sz.font_size).font(sb).color(c_on).into(),
        };

        let inner_pad = match sq_size {
            Some(_) => Padding::ZERO,
            None => sz.padding(),
        };
        let r = radius;
        let s0 = c_s0;
        let s1 = c_s1;
        let s2 = c_s2;
        let s3 = c_s3;
        let cont4 = c_cont_s4;

        // Pick colors based on state
        let alpha = if dis { 0.5 } else { 1.0 };
        let (it, ib) = if dis {
            (s0, s0)
        } else if pressed {
            (s2, s1)
        } else if hovered {
            (s1, s2)
        } else {
            (s0, s1)
        };
        let outline_c = if pressed || hovered { s3 } else { s2 };
        let (ot, om, ob) = if dis {
            (s0, s0, s0)
        } else if pressed {
            (s1, s1, s1)
        } else {
            (cont4, s0, s0)
        };

        let a = alpha;
        let apply_a = move |c: iced::Color| iced::Color { a: c.a * a, ..c };

        mouse_area(
            container(
                container(content)
                    .padding(inner_pad)
                    .center_x(Length::Shrink)
                    .style(move |_: &IcedTheme| iced::widget::container::Style {
                        background: Some(iced::Background::Gradient(
                            iced::gradient::Linear::new(pi)
                                .add_stop(0.0, apply_a(it))
                                .add_stop(1.0, apply_a(ib))
                                .into(),
                        )),
                        border: iced::Border {
                            radius: (r - 2.0).max(0.0).into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            )
            .padding(2)
            .style(move |_: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Gradient(
                    iced::gradient::Linear::new(pi)
                        .add_stop(0.0, apply_a(ot))
                        .add_stop(0.4, apply_a(om))
                        .add_stop(1.0, apply_a(ob))
                        .into(),
                )),
                border: iced::Border {
                    radius: r.into(),
                    width: 1.0,
                    color: apply_a(outline_c),
                },
                ..Default::default()
            }),
        )
        .on_enter(Msg::GradientEnter(idx))
        .on_exit(Msg::GradientExit(idx))
        .on_press(Msg::GradientPress(idx))
        .on_release(Msg::GradientRelease(idx))
        .into()
    };

    let r100 = d.radius_100;
    let emph_cat: Element<'_, Msg> = column![
        text("Primary Emphasized").size(d.font_label_small),
        row![
            emph_btn(
                "Label",
                &md,
                r100,
                Some(iced_fonts::bootstrap::play_fill()),
                None
            ),
            emph_btn(
                "Label",
                &sm,
                r100,
                Some(iced_fonts::bootstrap::play_fill()),
                None
            ),
            emph_btn(
                "Label",
                &xs,
                r100,
                Some(iced_fonts::bootstrap::play_fill()),
                None
            ),
        ]
        .spacing(d.space_75)
        .align_y(iced::Alignment::Center),
        row![
            emph_btn(
                "",
                &md,
                r100,
                Some(iced_fonts::bootstrap::play_fill()),
                Some(sq_md)
            ),
            emph_btn(
                "",
                &sm,
                r100,
                Some(iced_fonts::bootstrap::play_fill()),
                Some(sq_sm)
            ),
            emph_btn(
                "",
                &xs,
                r100,
                Some(iced_fonts::bootstrap::play_fill()),
                Some(sq_xs)
            ),
        ]
        .spacing(d.space_75)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(d.space_50)
    .into();

    // Disabled toggle
    let disabled_toggle: Element<'_, Msg> = toggler(state.buttons_disabled)
        .label("Disabled")
        .on_toggle(Msg::ButtonsDisabledToggle)
        .text_size(d.font_label_small)
        .width(Length::Shrink)
        .style(t.toggler(&["toggle"]))
        .into();

    let header = t
        .row(&["row-loose"])
        .push(t.text("Buttons", &["title-small"]))
        .push(disabled_toggle)
        .align_y(iced::Alignment::Center);

    t.frame(
        t.column(&["stack"])
            .push(header)
            .push(primary_cat)
            .push(success_cat)
            .push(danger_cat)
            .push(default_cat)
            .push(ghost_cat)
            .push(outlined_cat)
            .push(emph_cat),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn inputs_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let _d = &state.dims;
    let md = t.sizing(&["sz-md"]);
    let sm = t.sizing(&["sz-sm"]);
    let xs = t.sizing(&["sz-xs"]);

    use iced_fonts::bootstrap;
    use icss::widgets::icon_input::IconInput;

    let ic = |_sz: f32| -> iced::Color { t.color_var("text-soft").unwrap_or(iced::Color::WHITE) };
    let icon_sz = |font_sz: f32| -> f32 { (font_sz * 0.85).round() };

    // Medium
    let search_md = IconInput::new("Search...", &state.text_value)
        .leading(
            bootstrap::search()
                .size(icon_sz(md.font_size))
                .color(ic(0.0)),
        )
        .trailing(bootstrap::x_lg().size(icon_sz(md.font_size)).color(ic(0.0)))
        .on_input(Msg::TextChanged)
        .input_style(&["input"])
        .sizing(&["sz-md"])
        .view(t);

    let combo_md = IconInput::new("Select option...", "")
        .trailing(
            bootstrap::chevron_down()
                .size(icon_sz(md.font_size))
                .color(ic(0.0)),
        )
        .input_style(&["input"])
        .sizing(&["sz-md"])
        .view(t);

    // Small
    let search_sm = IconInput::new("Search...", &state.text_value)
        .leading(
            bootstrap::search()
                .size(icon_sz(sm.font_size))
                .color(ic(0.0)),
        )
        .trailing(bootstrap::x_lg().size(icon_sz(sm.font_size)).color(ic(0.0)))
        .on_input(Msg::TextChanged)
        .input_style(&["input"])
        .sizing(&["sz-sm"])
        .view(t);

    let combo_sm = IconInput::new("Select option...", "")
        .trailing(
            bootstrap::chevron_down()
                .size(icon_sz(sm.font_size))
                .color(ic(0.0)),
        )
        .input_style(&["input"])
        .sizing(&["sz-sm"])
        .view(t);

    // Tiny
    let search_xs = IconInput::new("Search...", &state.text_value)
        .leading(
            bootstrap::search()
                .size(icon_sz(xs.font_size))
                .color(ic(0.0)),
        )
        .on_input(Msg::TextChanged)
        .input_style(&["input"])
        .sizing(&["sz-xs"])
        .view(t);

    let combo_xs = IconInput::new("Select...", "")
        .trailing(
            bootstrap::chevron_down()
                .size(icon_sz(xs.font_size))
                .color(ic(0.0)),
        )
        .input_style(&["input"])
        .sizing(&["sz-xs"])
        .view(t);

    let three_sizes = t
        .row(&["row"])
        .push(icss::widgets::protect(
            &md,
            text_input("Medium input...", &state.text_value)
                .on_input(Msg::TextChanged)
                .padding(md.padding())
                .size(md.font_size)
                .style(t.text_input(&["input", "sz-md"])),
        ))
        .push(icss::widgets::protect(
            &sm,
            text_input("Small input...", &state.text_value)
                .on_input(Msg::TextChanged)
                .padding(sm.padding())
                .size(sm.font_size)
                .style(t.text_input(&["input", "sz-sm"])),
        ))
        .push(icss::widgets::protect(
            &xs,
            text_input("Tiny input...", &state.text_value)
                .on_input(Msg::TextChanged)
                .padding(xs.padding())
                .size(xs.font_size)
                .style(t.text_input(&["input", "sz-xs"])),
        ))
        .align_y(iced::Alignment::Center);

    let states = t
        .row(&["row"])
        .push(
            t.column(&["field-col"])
                .push(icss::widgets::protect(
                    &md,
                    text_input("Error state", &state.error_value)
                        .on_input(Msg::ErrorTextChanged)
                        .padding(md.padding())
                        .size(md.font_size)
                        .style(t.text_input(&["input", "error"])),
                ))
                .push(t.text("This field is required", &["label-micro", "text-danger"]))
                .width(Length::Fill),
        )
        .push(icss::widgets::protect(
            &md,
            text_input("Disabled", "Cannot edit")
                .padding(md.padding())
                .size(md.font_size)
                .width(Length::Fill)
                .style(t.text_input(&["input"])),
        ));

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Text Inputs", &["title-small"]))
            .push(t.text("Three sizes", &["label-small"]))
            .push(three_sizes)
            .push(t.text("States", &["label-small"]))
            .push(states)
            .push(t.text("With icons", &["label-small"]))
            .push(
                t.row(&["row"])
                    .push(search_md)
                    .push(combo_md)
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["row"])
                    .push(search_sm)
                    .push(combo_sm)
                    .align_y(iced::Alignment::Center),
            )
            .push(
                t.row(&["row"])
                    .push(search_xs)
                    .push(combo_xs)
                    .align_y(iced::Alignment::Center),
            ),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn button_group_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let d = &state.dims;
    let sb = state
        .vars
        .font_family
        .weighted(iced::font::Weight::Semibold);
    // (icss::widgets::button_group::ButtonGroup unused here; sample uses btn_group_row helper.)

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Button Group", &["title-small"]))
            .push(t.text("Three sizes", &["label-small"]))
            .push(
                t.row(&["row-loose"])
                    .push(btn_group_row(
                        t,
                        d,
                        sb,
                        &["List", "Grid", "Board", "Table"],
                        state.btn_group_active,
                        &["sz-md"],
                    ))
                    .push(btn_group_row(
                        t,
                        d,
                        sb,
                        &["Day", "Week", "Month"],
                        state.btn_group_active.min(2),
                        &["sz-sm"],
                    ))
                    .push(btn_group_row(
                        t,
                        d,
                        sb,
                        &["On", "Off"],
                        state.btn_group_active.min(1),
                        &["sz-xs"],
                    ))
                    .align_y(iced::Alignment::Center),
            ),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn control_group_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let d = &state.dims;
    use icss::widgets::control_group::{ControlGroup, ControlLayout};
    use icss::widgets::mdi;

    // Horizontal 3 sizes: Label + Input + Button
    let cg_md = ControlGroup::new(
        t.input("Enter email...", &state.text_value, &["input", "sz-md"])
            .on_input(Msg::TextChanged),
    )
    .label("Email")
    .trailing(
        t.btn_sq(
            mdi_icon(t, mdi::SEND, &["button", "primary", "sz-md"]),
            &["button", "primary", "sz-md"],
        )
        .on_press(Msg::Noop),
    )
    .layout(ControlLayout::Horizontal)
    .gap(d.space_75)
    .font_size(d.font_label_large)
    .view(t);

    let cg_sm = ControlGroup::new(
        t.input("Search...", &state.text_value, &["input", "sz-sm"])
            .on_input(Msg::TextChanged),
    )
    .label("Search")
    .trailing(
        t.btn_sq(
            mdi_icon(t, mdi::SCAN, &["button", "primary", "sz-sm"]),
            &["button", "primary", "sz-sm"],
        )
        .on_press(Msg::Noop),
    )
    .layout(ControlLayout::Horizontal)
    .gap(d.space_50)
    .font_size(d.font_label_medium)
    .view(t);

    let cg_xs = ControlGroup::new(
        t.input("Filter...", &state.text_value, &["input", "sz-xs"])
            .on_input(Msg::TextChanged),
    )
    .label("Filter")
    .trailing(
        t.btn_sq(
            mdi_icon(t, mdi::X, &["button", "ghost", "sz-xs"]),
            &["button", "ghost", "sz-xs"],
        )
        .on_press(Msg::Noop),
    )
    .layout(ControlLayout::Horizontal)
    .gap(d.space_25)
    .font_size(d.font_label_small)
    .view(t);

    // Vertical with error
    let cg_v = ControlGroup::new(
        t.input("Username", &state.error_value, &["input", "error", "sz-sm"])
            .on_input(Msg::ErrorTextChanged),
    )
    .label("Username")
    .error("Username must be at least 3 characters")
    .layout(ControlLayout::Vertical)
    .gap(d.space_50)
    .font_size(d.font_label_medium)
    .view(t);

    // Zoom control: [-] 100% [+]
    let zoom_control = ControlGroup::new(t.text("100%", &["label-medium"]))
        .leading(
            t.btn_sq(
                mdi_icon(t, mdi::MINIMIZE, &["button", "default", "sz-sm"]),
                &["button", "default", "sz-sm"],
            )
            .on_press(Msg::Noop),
        )
        .trailing(
            t.btn_sq(
                mdi_icon(t, mdi::MAXIMIZE, &["button", "default", "sz-sm"]),
                &["button", "default", "sz-sm"],
            )
            .on_press(Msg::Noop),
        )
        .label("Zoom")
        .layout(ControlLayout::Horizontal)
        .gap(d.space_50)
        .font_size(d.font_label_medium)
        .view(t);

    // Menu demo
    use icss::widgets::menu::{Menu, MenuItem};
    let menu_demo = Menu::new()
        .item(MenuItem::icon_label(
            mdi_icon(t, mdi::FILE_TEXT, &["button", "ghost", "sz-sm"]),
            "New File",
            Msg::Noop,
        ))
        .item(MenuItem::icon_label(
            mdi_icon(t, mdi::FOLDER, &["button", "ghost", "sz-sm"]),
            "Open",
            Msg::Noop,
        ))
        .item(MenuItem::icon_label(
            mdi_icon(t, mdi::ARROW_UP_RIGHT, &["button", "ghost", "sz-sm"]),
            "Save",
            Msg::Noop,
        ))
        .divider()
        .item(MenuItem::check(
            "Word Wrap",
            state.check_a,
            Msg::CheckA(true),
        ))
        .item(MenuItem::check(
            "Line Numbers",
            state.check_b,
            Msg::CheckB(true),
        ))
        .divider()
        .item(MenuItem::submenu("Recent Files", Msg::Noop))
        .item(MenuItem::submenu("Encoding", Msg::Noop))
        .divider()
        .item(MenuItem::custom(
            t.row(&["cluster"])
                .push(
                    t.btn_sq(
                        mdi_icon(t, mdi::MINIMIZE, &["button", "ghost", "sz-sm"]),
                        &["button", "ghost", "sz-sm"],
                    )
                    .on_press(Msg::Noop),
                )
                .push(t.text("100%", &["label-medium"]))
                .push(
                    t.btn_sq(
                        mdi_icon(t, mdi::MAXIMIZE, &["button", "ghost", "sz-sm"]),
                        &["button", "ghost", "sz-sm"],
                    )
                    .on_press(Msg::Noop),
                )
                .align_y(iced::Alignment::Center),
        ))
        .divider()
        .item(MenuItem::custom(
            t.row(&["row"])
                .push(t.text("Opacity", &["label-small"]))
                .push(
                    t.slide(
                        0.0..=1.0,
                        state.slider_value,
                        Msg::SliderChanged,
                        &["slider"],
                    )
                    .width(120),
                )
                .align_y(iced::Alignment::Center),
        ))
        .view(t);

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Control Group", &["title-small"]))
            .push(t.text("Horizontal: 3 sizes", &["label-small"]))
            .push(cg_md)
            .push(cg_sm)
            .push(cg_xs)
            .push(t.text("Vertical: label + input + error", &["label-small"]))
            .push(cg_v)
            .push(t.text("Inline: label + button + value + button", &["label-small"]))
            .push(zoom_control)
            .push(rule::horizontal(1).style(t.rule(&["divider"])))
            .push(t.text("Menu", &["title-small"]))
            .push(menu_demo),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn controls_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let md = t.sizing(&["sz-md"]);
    let sm = t.sizing(&["sz-sm"]);
    let xs = t.sizing(&["sz-xs"]);

    // Component-specific sizing for inline controls (checkbox/radio/toggle)
    // — these get a line-height-only min-height, not the padded button height.
    let chk_md = t.sizing(&["checkbox", "sz-md"]);
    let chk_sm = t.sizing(&["checkbox", "sz-sm"]);
    let chk_xs = t.sizing(&["checkbox", "sz-xs"]);

    // Checkboxes — three sizes (md / sm / xs). Box size tracks the label
    // font so they scale together. spacing() is the gap between box and
    // label. The `.sz-*` class triggers the compound radius rule in
    // compose.rs so small/tiny checkboxes use r-25 instead of r-50.
    let checks = t
        .row(&["row-loose"])
        .push(
            t.column(&["field-col"])
                .push(id("C01", t))
                .push(icss::widgets::protect(
                    &chk_md,
                    checkbox(state.check_a)
                        .label("Medium")
                        .size(md.font_size)
                        .text_size(md.font_size)
                        .spacing(md.gap)
                        .on_toggle(Msg::CheckA)
                        .style(t.checkbox(&["checkbox", "sz-md"])),
                )),
        )
        .push(
            t.column(&["field-col"])
                .push(id("C01", t))
                .push(icss::widgets::protect(
                    &chk_sm,
                    checkbox(state.check_b)
                        .label("Small")
                        .size(sm.font_size)
                        .text_size(sm.font_size)
                        .spacing(sm.gap)
                        .on_toggle(Msg::CheckB)
                        .style(t.checkbox(&["checkbox", "sz-sm"])),
                )),
        )
        .push(
            t.column(&["field-col"])
                .push(id("C02", t))
                .push(icss::widgets::protect(
                    &chk_xs,
                    checkbox(state.check_c)
                        .label("Tiny")
                        .size(xs.font_size)
                        .text_size(xs.font_size)
                        .spacing(xs.gap)
                        .on_toggle(Msg::CheckC)
                        .style(t.checkbox(&["checkbox", "sz-xs"])),
                )),
        );

    // Togglers — iced toggler doesn't expose a direct size knob, but the
    // text_size + row height shift with font_size. Track width scales too.
    let toggles = t
        .row(&["row-loose"])
        .push(
            t.column(&["field-col"])
                .push(id("T01", t))
                .push(icss::widgets::protect(
                    &chk_md,
                    toggler(state.toggle_a)
                        .label("Medium")
                        .size(md.font_size)
                        .text_size(md.font_size)
                        .spacing(md.gap)
                        .on_toggle(Msg::ToggleA)
                        .style(t.toggler(&["toggle"])),
                )),
        )
        .push(
            t.column(&["field-col"])
                .push(id("T02", t))
                .push(icss::widgets::protect(
                    &chk_sm,
                    toggler(state.toggle_b)
                        .label("Small")
                        .size(sm.font_size)
                        .text_size(sm.font_size)
                        .spacing(sm.gap)
                        .on_toggle(Msg::ToggleB)
                        .style(t.toggler(&["toggle"])),
                )),
        );

    // Radios. The box size (`.size(...)`) is snapped to an even integer so
    // iced's radio `draw()` — which computes `dot_size = size / 2.0` and
    // positions the dot at `bounds.x + dot_size / 2.0` — lands on the pixel
    // grid. Odd box sizes produce a .5 offset that rounds inconsistently
    // and shifts the dot visibly. Text stays at the unmodified font size.
    let even = |v: f32| -> f32 {
        let n = v.round() as i32;
        (if n & 1 == 1 { n + 1 } else { n }) as f32
    };
    let radios = t
        .row(&["row-loose"])
        .push(
            t.column(&["field-col"])
                .push(id("R01", t))
                .push(icss::widgets::protect(
                    &chk_md,
                    radio(
                        "Medium",
                        RadioOpt::Alpha,
                        state.radio_choice,
                        Msg::RadioSelected,
                    )
                    .size(even(md.font_size))
                    .text_size(md.font_size)
                    .spacing(md.gap)
                    .style(t.radio(&["radio"])),
                )),
        )
        .push(
            t.column(&["field-col"])
                .push(id("R02", t))
                .push(icss::widgets::protect(
                    &chk_sm,
                    radio(
                        "Small",
                        RadioOpt::Beta,
                        state.radio_choice,
                        Msg::RadioSelected,
                    )
                    .size(even(sm.font_size))
                    .text_size(sm.font_size)
                    .spacing(sm.gap)
                    .style(t.radio(&["radio"])),
                )),
        )
        .push(
            t.column(&["field-col"])
                .push(id("R02", t))
                .push(icss::widgets::protect(
                    &chk_xs,
                    radio(
                        "Tiny",
                        RadioOpt::Gamma,
                        state.radio_choice,
                        Msg::RadioSelected,
                    )
                    .size(even(xs.font_size))
                    .text_size(xs.font_size)
                    .spacing(xs.gap)
                    .style(t.radio(&["radio"])),
                )),
        );

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Checkboxes, Togglers & Radios", &["title-small"]))
            .push(t.text("Checkboxes — 3 sizes", &["label-small"]))
            .push(checks)
            .push(t.text("Togglers — 2 sizes", &["label-small"]))
            .push(toggles)
            .push(t.text("Radios — 3 sizes", &["label-small"]))
            .push(radios),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn sliders_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let pct = (state.slider_value * 100.0) as u32;

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Slider", &["title-small"]))
            .push(
                t.column(&["field-col"])
                    .push(id("S01", t))
                    .push(t.text(format!("Value: {pct}%"), &["label-small"]))
                    .push(
                        slider(0.0..=1.0, state.slider_value, Msg::SliderChanged)
                            .step(0.01)
                            .style(t.slider(&["slider"])),
                    ),
            ),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn progress_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;

    let bar_h = 5.0;
    let bar = |id_label: &'static str,
               label: &'static str,
               value: f32,
               classes: &'static [&'static str]| {
        t.column(&["field-col"])
            .push(id(id_label, t))
            .push(t.text(label, &["label-small"]))
            .push(
                progress_bar(0.0..=1.0, value)
                    .girth(bar_h)
                    .style(t.progress_bar(classes)),
            )
    };

    t.frame(
        t.column(&["stack-tight"])
            .push(t.text("Progress Bars", &["title-small"]))
            .push(bar("PB01", "Default 40%", 0.4, &["progress"]))
            .push(bar("PB02", "Success 75%", 0.75, &["progress", "success"]))
            .push(bar("PB03", "Danger 90%", 0.9, &["progress", "danger"]))
            .push(bar("PB04", "Warning 55%", 0.55, &["progress", "warning"])),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn pick_list_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let _d = &state.dims;
    let md = t.sizing(&["sz-md"]);
    let sm = t.sizing(&["sz-sm"]);
    let xs = t.sizing(&["sz-xs"]);

    let options = vec![
        "English".to_string(),
        "Spanish".to_string(),
        "French".to_string(),
        "German".to_string(),
        "Japanese".to_string(),
    ];

    let pick_col = t
        .column(&["field-col"])
        .push(t.text("Pick List — three sizes", &["label-small"]))
        .push(
            t.row(&["row"])
                .push(icss::widgets::protect(
                    &md,
                    pick_list(
                        options.clone(),
                        state.pick_choice.clone(),
                        Msg::PickSelected,
                    )
                    .placeholder("Medium...")
                    .text_size(md.font_size)
                    .padding([md.pad_v, md.pad_h])
                    .style(t.pick_list(&["select", "sz-md"]))
                    .menu_style(t.menu(&["select-menu", "sz-md"])),
                ))
                .push(icss::widgets::protect(
                    &sm,
                    pick_list(
                        options.clone(),
                        state.pick_choice.clone(),
                        Msg::PickSelected,
                    )
                    .placeholder("Small...")
                    .text_size(sm.font_size)
                    .padding([sm.pad_v, sm.pad_h])
                    .style(t.pick_list(&["select", "sz-sm"]))
                    .menu_style(t.menu(&["select-menu", "sz-sm"])),
                ))
                .push(icss::widgets::protect(
                    &xs,
                    pick_list(options, state.pick_choice.clone(), Msg::PickSelected)
                        .placeholder("Tiny...")
                        .text_size(xs.font_size)
                        .padding([xs.pad_v, xs.pad_h])
                        .style(t.pick_list(&["select", "sz-xs"]))
                        .menu_style(t.menu(&["select-menu", "sz-xs"])),
                ))
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill);

    let combo_col = t
        .column(&["field-col"])
        .push(t.text("Combo Box (searchable)", &["label-small"]))
        .push(id("CB01", t))
        .push(icss::widgets::protect(
            &md,
            combo_box(
                &state.combo_state,
                "Search language...",
                state.combo_value.as_ref(),
                Msg::ComboSelected,
            )
            .input_style(t.text_input(&["input", "sz-md"]))
            .menu_style(t.menu(&["select-menu", "sz-md"]))
            .size(md.font_size)
            .padding([md.pad_v, md.pad_h]),
        ))
        .width(Length::Fill);

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Pick List & Combo Box", &["title-small"]))
            .push(t.row(&["row-loose"]).push(pick_col).push(combo_col)),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn editor_tooltip_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let _d = &state.dims;
    let md = t.sizing(&["sz-md"]);
    let xs = t.sizing(&["sz-xs"]);

    let editor_col = t
        .column(&["field-col"])
        .push(t.text("Text Editor", &["title-small"]))
        .push(
            text_editor(&state.editor_content)
                .on_action(Msg::EditorAction)
                .padding(md.padding())
                .size(md.font_size)
                .style(t.text_editor(&["editor", "sz-md"]))
                .height(100),
        )
        .width(Length::Fill);

    let tooltip_col = t
        .column(&["field-col"])
        .push(t.text("Tooltip", &["title-small"]))
        .push(
            tooltip(
                button(text("Hover me for tooltip").size(md.font_size))
                    .padding(md.padding())
                    .on_press(Msg::Noop)
                    .style(t.button(&["button", "primary"])),
                container(text("This is a styled tooltip").size(xs.font_size))
                    .padding(xs.padding())
                    .style(t.tooltip(&["tooltip"])),
                tooltip::Position::Bottom,
            )
            .gap(xs.gap),
        );

    t.frame(
        t.row(&["row-loose"]).push(editor_col).push(tooltip_col),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn chat_textarea_section(state: &State) -> Element<'_, Msg> {
    use icss::widgets::mdi;

    let t = &state.theme;
    let d = &state.dims;
    let md = t.sizing(&["sz-md"]);

    // Resolve input styling for the outer container
    let input_cls: &[&str] = &["input", "sz-md"];
    let computed = t.resolve(input_cls, None);
    let bg_color = Theme::resolve_color(&computed, "background-color")
        .map(|c| c.to_iced())
        .unwrap_or(iced::Color::TRANSPARENT);
    let border_radius = computed.length("border-radius").unwrap_or(0.0);
    let border_width = computed.length("border-width").unwrap_or(0.0);
    let border_color = Theme::resolve_color(&computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(iced::Color::TRANSPARENT);
    let text_color = Theme::resolve_color(&computed, "color")
        .map(|c| c.to_iced())
        .unwrap_or(iced::Color::WHITE);
    let placeholder_color = Theme::resolve_color(&computed, "placeholder-color")
        .map(|c| c.to_iced())
        .unwrap_or(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.4));
    let selection = Theme::resolve_color(&computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(iced::Color::from_rgba(0.0, 0.4, 0.8, 0.3));

    // Dynamic editor height: grows from 1 to 5 visual lines.
    // line_count() only counts \n-separated lines, not visual wraps.
    // Estimate wrapped lines from text length and available width.
    let line_h = md.font_size * 1.4;
    let pad = md.pad_v;
    let container_w = 400.0_f32;
    let available_w = container_w - 2.0 * md.pad_h;
    let avg_char_w = md.font_size * 0.55; // proportional font estimate
    let chars_per_line = (available_w / avg_char_w).floor().max(1.0);
    let content_text = state.chat_textarea_content.text();
    let visual_lines: f32 = content_text.split('\n').fold(0.0, |acc, line| {
        acc + (line.len() as f32 / chars_per_line).ceil().max(1.0)
    });
    let line_count = visual_lines.clamp(1.0, 5.0);
    let editor_h = line_h * line_count + pad * 2.0;

    // Borderless transparent text editor
    let tc = text_color;
    let pc = placeholder_color;
    let sel = selection;
    let editor = text_editor(&state.chat_textarea_content)
        .on_action(Msg::ChatTextareaAction)
        .placeholder("Type a message...")
        .padding(Padding::from([pad, md.pad_h]))
        .size(md.font_size)
        .style(
            move |_: &IcedTheme, _status| iced::widget::text_editor::Style {
                background: iced::Background::Color(iced::Color::TRANSPARENT),
                border: iced::Border::default(),
                placeholder: pc,
                value: tc,
                selection: sel,
            },
        )
        .height(editor_h);

    // Ghost button (attachment) — icon only, 1:1 square
    let ghost_btn = t
        .btn_sq(
            mdi_icon(t, mdi::PAPERCLIP, &["button", "ghost", "tiny"]),
            &["button", "ghost", "tiny"],
        )
        .on_press(Msg::ChatAttach);

    // Primary button (send) — icon only, 1:1 square
    let primary_btn = t
        .btn_sq(
            mdi_icon(t, mdi::SEND, &["button", "primary", "tiny"]),
            &["button", "primary", "tiny"],
        )
        .on_press(Msg::ChatSend);

    // Button row — bottom-right
    let btn_row = t
        .row(&["cluster"])
        .push(ghost_btn)
        .push(primary_btn)
        .align_y(iced::Alignment::Center);

    // Compose: editor on top, buttons bottom-right
    let inner = column![
        editor,
        container(btn_row)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .padding(Padding::ZERO.bottom(d.space_75).right(d.space_75)),
    ];

    // Outer container styled as the input field
    let bg = bg_color;
    let bc = border_color;
    let bw = border_width;
    let br = border_radius;
    let chat_textarea =
        container(inner)
            .width(400)
            .style(move |_: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    color: bc,
                    width: bw,
                    radius: br.into(),
                },
                ..Default::default()
            });

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Chat Textarea", &["title-small"]))
            .push(
                t.column(&["field-col"])
                    .push(t.text(
                        "Auto-growing textarea (1\u{2013}5 lines) with inline action buttons",
                        &["text-soft"],
                    ))
                    .push(chat_textarea),
            ),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn tile_grid_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let d = &state.dims;

    // Device tile data
    struct DevTile {
        name: &'static str,
        os: &'static str,
        status: &'static str,
        latency: &'static str,
    }
    let devices = [
        DevTile {
            name: "Alice Carter",
            os: "Windows 11",
            status: "Online",
            latency: "12ms",
        },
        DevTile {
            name: "Ben Morales",
            os: "macOS 14.2",
            status: "Online",
            latency: "8ms",
        },
        DevTile {
            name: "Chloe Watson",
            os: "Ubuntu 24.04",
            status: "Away",
            latency: "45ms",
        },
        DevTile {
            name: "Daniel Reyes",
            os: "Windows Server",
            status: "Offline",
            latency: "\u{2014}",
        },
        DevTile {
            name: "Emma Lindqvist",
            os: "iPadOS 17",
            status: "Online",
            latency: "22ms",
        },
        DevTile {
            name: "Farid Hassan",
            os: "Android 14",
            status: "Online",
            latency: "31ms",
        },
    ];

    let status_color = |s: &str| -> iced::Color {
        match s {
            "Online" => t
                .color_var("surface-success-s0")
                .unwrap_or(iced::Color::from_rgb(0.2, 0.8, 0.4)),
            "Away" => t
                .color_var("surface-warning-s0")
                .unwrap_or(iced::Color::from_rgb(0.9, 0.7, 0.2)),
            _ => t
                .color_var("text-faint")
                .unwrap_or(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.3)),
        }
    };

    let mut grid = TileGrid::new()
        .layout(state.tile_layout.clone())
        .spacing(d.space_150)
        .tile_padding(d.space_200)
        .selected(&state.tile_selected);

    for (i, dev) in devices.iter().enumerate() {
        let tile_content = t
            .column(&["cluster"])
            .push(t.text(dev.name, &["label-large"]))
            .push(t.text(dev.os, &["label-small", "text-soft"]))
            .push(
                row![
                    t.text("\u{25CF} ", &["label-small"])
                        .color(status_color(dev.status)),
                    t.text(dev.status, &["label-small"]),
                ]
                .spacing(d.space_25),
            )
            .push(t.text(
                format!("Latency: {}", dev.latency),
                &["label-micro", "text-faint"],
            ));
        grid = grid.push(tile_content, Msg::TilePressed(i));
    }

    let tile_view = grid.view(t);

    let layout_options = vec![
        "Flow".to_string(),
        "Horizontal".to_string(),
        "Vertical".to_string(),
    ];
    let current_layout = match state.tile_layout {
        TileLayout::Flow { .. } => "Flow".to_string(),
        TileLayout::Horizontal => "Horizontal".to_string(),
        TileLayout::Vertical => "Vertical".to_string(),
    };

    let toolbar = t
        .row(&["row"])
        .push(t.text("Layout:", &["label-small"]))
        .push(
            pick_list(layout_options, Some(current_layout), Msg::TileLayoutChanged)
                .text_size(d.font_label_small)
                .style(t.pick_list(&["select"]))
                .menu_style(t.menu(&["select-menu"])),
        )
        .push(t.text(
            format!("Selected: {:?}", state.tile_selected),
            &["label-micro", "text-faint"],
        ))
        .align_y(iced::Alignment::Center);

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Tile Grid", &["title-small"]))
            .push(toolbar)
            .push(tile_view),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn data_table_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;

    let status_color = |s: &str| -> iced::Color {
        match s {
            "Online" => t
                .color_var("surface-success-s0")
                .unwrap_or(iced::Color::from_rgb(0.2, 0.8, 0.4)),
            "Away" => t
                .color_var("surface-warning-s0")
                .unwrap_or(iced::Color::from_rgb(0.9, 0.7, 0.2)),
            _ => t
                .color_var("text-faint")
                .unwrap_or(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.3)),
        }
    };

    let columns = vec![
        DataColumn::new("Name", move |c: &Contact, _| {
            if c.detail.is_empty() {
                t.text(c.name.clone(), &["label-medium"]).into()
            } else {
                iced::widget::column![
                    t.text(c.name.clone(), &["label-medium"]),
                    t.text(c.detail.clone(), &["label-small", "text-soft"]),
                ]
                .spacing(2)
                .into()
            }
        })
        .sortable("name")
        .width(Length::Fill)
        .col_min_width(220.0),
        DataColumn::new("Email", move |c: &Contact, _| {
            t.text(c.email.clone(), &["label-medium"]).into()
        })
        .sortable("email")
        .width(Length::Fill)
        .col_min_width(200.0),
        DataColumn::new("Role", move |c: &Contact, _| {
            t.text(c.role.clone(), &["label-medium"]).into()
        })
        .sortable("role")
        .width(Length::Fill)
        .col_min_width(100.0),
        DataColumn::new("Status", move |c: &Contact, _| {
            let color = status_color(&c.status);
            t.text(c.status.clone(), &["label-medium"])
                .color(color)
                .into()
        })
        .sortable("status")
        .width(Length::Fill)
        .col_min_width(100.0),
        DataColumn::new("Location", move |c: &Contact, _| {
            t.text(c.location.clone(), &["label-medium"]).into()
        })
        .sortable("location")
        .width(Length::Fill)
        .col_min_width(140.0),
    ];

    let mut tbl = DataTable::new(columns, &state.dt_filtered)
        .selected(&state.dt_selected)
        .page(state.dt_page)
        .page_size(state.dt_page_size)
        .select_column(true)
        .search(&state.dt_search);

    if let Some(ref sort) = state.dt_sort {
        tbl = tbl.sort_state(sort);
    }

    let (table_header, table_body, table_footer) = tbl.view_split(
        t,
        Msg::DtRowPressed,
        Msg::DtSort,
        Msg::DtSelect,
        Msg::DtSelectAll,
        Msg::DtPageChanged,
        Msg::DtPageSizeChanged,
        Msg::DtSearchChanged,
    );

    let title = t
        .column(&["field-col"])
        .push(t.text("Data Table", &["title-small"]))
        .push(t.text(
            "Sortable columns, row selection, pagination, search",
            &["label-small", "text-soft"],
        ));

    let search_bar = container(
        t.input("Search...", &state.dt_search, &["input", "sz-md"])
            .on_input(Msg::DtSearchChanged),
    )
    .padding([0, 8]);

    let sticky_table = icss::widgets::StickySection::new(table_header, table_body);

    t.frame(
        t.column(&["subsection"])
            .push(title)
            .push(search_bar)
            .push(sticky_table)
            .push(table_footer),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn typography_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let d = &state.dims;
    let family = state.vars.font_family;

    let surfaces: Vec<(&str, &[&str])> = vec![
        ("Surface", &["typo-surface"]),
        ("Surface+2", &["typo-surface-raised"]),
        ("Tint", &["typo-tint"]),
        ("Dark tint", &["typo-dark-tint"]),
        ("Black", &["typo-black"]),
        ("Primary", &["typo-primary"]),
        ("Primary container", &["typo-primary-container"]),
    ];

    let row1_surfaces = &surfaces[..4];
    let row2_surfaces = &surfaces[4..];

    let row1 = row![].spacing(d.space_100);
    let row1 = row1_surfaces.iter().fold(row1, |r, (label, classes)| {
        r.push(typo_block(label, classes, t, d, family))
    });

    let row2 = row![].spacing(d.space_100);
    let row2 = row2_surfaces.iter().fold(row2, |r, (label, classes)| {
        r.push(typo_block(label, classes, t, d, family))
    });

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Typography on Surfaces", &["title-small"]))
            .push(row1)
            .push(row2),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}

fn typo_block<'a>(
    label: &'a str,
    classes: &'a [&str],
    t: &'a Theme,
    d: &'a icss::engine::dims::DimTokens,
    family: FontFamily,
) -> Element<'a, Msg> {
    use iced::font::Weight;
    let w = |weight: Weight| family.weighted(weight);

    let content = column![
        text(label).size(d.font_label_micro),
        // Headlines: 300 (light) for large, 400 for small
        text("Headline Lg")
            .size(d.font_headline_medium)
            .font(w(Weight::Light)),
        text("Headline Sm")
            .size(d.font_headline_small)
            .font(w(Weight::Normal)),
        // Titles: 600 (semibold)
        text("Title Medium")
            .size(d.font_title_medium)
            .font(w(Weight::Semibold)),
        text("Title Small")
            .size(d.font_title_small)
            .font(w(Weight::Semibold)),
        // Labels: 600 (semibold) — used in buttons
        text("Label Large")
            .size(d.font_label_large)
            .font(w(Weight::Semibold)),
        text("Label Medium")
            .size(d.font_label_medium)
            .font(w(Weight::Semibold)),
        // Body: 400 (normal)
        text("Body Medium")
            .size(d.font_label_medium)
            .font(w(Weight::Normal)),
        text("Body Small")
            .size(d.font_label_small)
            .font(w(Weight::Normal)),
        text("Micro").size(d.font_label_micro),
    ]
    .spacing(2);

    container(content)
        .padding(Padding::from([d.space_100, d.space_150]))
        .width(Length::Fill)
        .style(t.container(classes))
        .into()
}

fn color_wheel_section(state: &State) -> Element<'_, Msg> {
    let d = &state.dims;
    let sb = state
        .vars
        .font_family
        .weighted(iced::font::Weight::Semibold);

    let gamma = state.vars.gamma;
    let step: usize = 50;
    let n: usize = 20;
    let anchors = [
        state.vars.primary.as_str(),
        state.vars.secondary.as_str(),
        state.vars.tertiary.as_str(),
        state.vars.quaternary.as_str(),
    ];
    let wheel = icss::engine::tonal::color_wheel(&anchors, step, n, gamma);

    // Work out which wheel slot each base palette color lands in (nearest hue).
    let base_hues: [(&str, f32); 4] = [
        (
            "P",
            icss::engine::tonal::TonalPalette::hue_of(&state.vars.primary),
        ),
        (
            "S",
            icss::engine::tonal::TonalPalette::hue_of(&state.vars.secondary),
        ),
        (
            "T",
            icss::engine::tonal::TonalPalette::hue_of(&state.vars.tertiary),
        ),
        (
            "Q",
            icss::engine::tonal::TonalPalette::hue_of(&state.vars.quaternary),
        ),
    ];
    let slot_label = |i: usize| -> Option<&'static str> {
        let hue = (i as f32) * 360.0 / (n as f32);
        let mut best: Option<(&'static str, f32)> = None;
        for (tag, bh) in &base_hues {
            let mut d = (hue - *bh).abs();
            if d > 180.0 {
                d = 360.0 - d;
            }
            // Half-slot radius = 360/(2n) = 9° for n=20.
            let radius = 360.0 / (2.0 * n as f32);
            if d <= radius {
                match best {
                    Some((_, cur)) if cur <= d => {}
                    _ => best = Some((tag, d)),
                }
            }
        }
        best.map(|(t, _)| t)
    };

    let mut swatches: Vec<Element<'_, Msg>> = Vec::with_capacity(n);
    for (i, c) in wheel.iter().enumerate() {
        let color = iced::Color::from_rgb(c.red, c.green, c.blue);
        let hue_deg = (i as f32) * 360.0 / (n as f32);
        let tag = slot_label(i);
        let swatch = container("")
            .width(40)
            .height(40)
            .style(move |_theme: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(color)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        let mut col = column![swatch].spacing(2).align_x(iced::Alignment::Center);
        if let Some(tag) = tag {
            col = col.push(text(tag).size(d.font_label_small).font(sb));
        } else {
            col = col.push(text(" ").size(d.font_label_small));
        }
        col = col.push(text(format!("{hue_deg:.0}°")).size(d.font_label_micro));
        swatches.push(col.into());
    }

    column![
        text(format!("Color wheel (step {step}, {n} hues)"))
            .size(d.font_title_small)
            .font(sb),
        Row::with_children(swatches).spacing(d.space_50).wrap(),
    ]
    .spacing(d.space_50)
    .into()
}

fn closest_path_section(state: &State) -> Element<'_, Msg> {
    let d = &state.dims;
    let sb = state
        .vars
        .font_family
        .weighted(iced::font::Weight::Semibold);

    let gamma = state.vars.gamma;
    let step: usize = 50;
    let n: usize = 20;
    let anchors = [
        state.vars.primary.as_str(),
        state.vars.secondary.as_str(),
        state.vars.tertiary.as_str(),
        state.vars.quaternary.as_str(),
    ];
    let scale = icss::engine::tonal::closest_path_scale(&anchors, step, n, gamma);

    let mut swatches: Vec<Element<'_, Msg>> = Vec::with_capacity(scale.len());
    for (c, hue_deg, is_anchor) in &scale {
        let color = iced::Color::from_rgb(c.red, c.green, c.blue);
        let swatch = container("")
            .width(40)
            .height(40)
            .style(move |_theme: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(color)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        let marker = if *is_anchor { "•" } else { " " };
        let col = column![
            swatch,
            text(marker).size(d.font_label_small).font(sb),
            text(format!("{:.0}°", hue_deg)).size(d.font_label_micro),
        ]
        .spacing(2)
        .align_x(iced::Alignment::Center);
        swatches.push(col.into());
    }

    column![
        text(format!(
            "Closest-path scale (step {step}, {n} hues, longest arc dropped)"
        ))
        .size(d.font_title_small)
        .font(sb),
        Row::with_children(swatches).spacing(d.space_50).wrap(),
    ]
    .spacing(d.space_50)
    .into()
}

fn text_colors_section(state: &State) -> Element<'_, Msg> {
    let t = &state.theme;
    let d = &state.dims;

    // Each block: surface bg + text at 4 emphasis levels.
    // "Full contrast" inherits from the container's `color`.
    // Lower emphasis levels use alpha relative to the text direction.
    // Text direction is determined by surface lightness:
    //   surface_lightness < 50 → light text (white alpha)
    //   surface_lightness >= 50 → dark text (black alpha)
    // Each surface: bg class, on-surface CSS var prefix.
    let surfaces: Vec<(&str, &[&str], &str)> = vec![
        ("Surface", &["color-surface"], "on-surface"),
        ("Tint", &["color-tint"], "on-surface-tint"),
        ("Dark tint", &["color-dark-tint"], "on-surface-dark-tint"),
        ("Primary", &["color-primary"], "on-surface-primary"),
        ("Black", &["color-black"], "on-surface-black"),
    ];

    let blocks = t.row(&["row"]);
    let blocks = surfaces.iter().fold(blocks, |r, (label, classes, prefix)| {
        // Resolve actual on-surface text tokens from the theme.
        let c = |suffix: &str| -> iced::Color {
            let var = if suffix.is_empty() {
                prefix.to_string()
            } else {
                format!("{prefix}-{suffix}")
            };
            t.color_var(&var).unwrap_or(iced::Color::WHITE)
        };

        let block = column![
            t.text(*label, &["label-micro"]),
            t.text("Full contrast", &["label-medium"]).color(c("")),
            t.text("Default emphasis", &["label-medium"])
                .color(c("default")),
            t.text("Soft / secondary", &["label-medium"])
                .color(c("soft")),
            t.text("Disabled / muted", &["label-medium"])
                .color(c("muted")),
        ]
        .spacing(3);

        r.push(
            container(block)
                .padding(Padding::from([d.space_100, d.space_150]))
                .width(Length::Fill)
                .style(t.container(classes)),
        )
    });

    t.frame(
        t.column(&["subsection"])
            .push(t.text("Text Colors on Surfaces", &["title-small"]))
            .push(blocks),
        &["section", "section-body"],
    )
    .width(Length::Fill)
    .into()
}
