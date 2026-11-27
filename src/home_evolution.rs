//! Native Evolution Home. Preferences persist; navigation and hover never do.
//! Used by the webOS runtime and the desktop Home preview.
pub use crate::home_motion::MotionProfile;
use crate::home_motion::{Motion, MotionCurve, MotionPart, MotionSpeed};
use crate::home_visual::{
    CinemaDetails, CinemaLayout, DockLabels, DockPosition, DockTheme, WeatherDetails, WidgetsLayout,
};
use crate::state::{Direction, Page, Rect, State, Target};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
#[path = "home-evolution-widgets.rs"]
mod widgets;
pub use widgets::{HeaderGeometry, calendar_labels, header_geometry};

pub const CONTENTS: [(&str, &str, &str); 8] = [
    ("dunes", "Além das dunas", "Filme · aventura"),
    ("san-francisco", "Horizonte azul", "Série · natureza"),
    ("forest", "Entre as árvores", "Filme · documentário"),
    ("summit", "Altitude", "Série · aventura"),
    ("northern-lights", "Mar aberto", "Filme · natureza"),
    ("dunes", "Luz do deserto", "Série · documentário"),
    ("forest", "Caminhos", "Filme · aventura"),
    ("yosemite", "Nosso lugar", "Série · natureza"),
];
pub const VISIBLE: usize = 5;
pub const INPUTS_PER_WINDOW: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlobalAction {
    Apps,
    Settings,
}

impl GlobalAction {
    pub const ALL: [Self; 2] = [Self::Apps, Self::Settings];

    pub fn index(self) -> usize {
        match self {
            Self::Apps => 0,
            Self::Settings => 1,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Apps => "grid",
            Self::Settings => "evolution-settings",
        }
    }

    pub fn label(self, language: crate::i18n::Language) -> &'static str {
        language.t(match self {
            Self::Apps => "evolutionAllApps",
            Self::Settings => "evolutionSettings",
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlobalPosition {
    #[default]
    Top,
    Dock,
}

impl GlobalPosition {
    pub const ALL: [Self; 2] = [Self::Top, Self::Dock];

    pub fn index(self) -> usize {
        usize::from(self == Self::Dock)
    }

    pub fn label(self, language: crate::i18n::Language) -> &'static str {
        language.t(match self {
            Self::Top => "evolutionTop",
            Self::Dock => "evolutionDock",
        })
    }
}

fn global_position<'de, D: serde::Deserializer<'de>>(d: D) -> Result<GlobalPosition, D::Error> {
    // A malformed field must not invalidate the surrounding saved preferences.
    let value = serde_json::Value::deserialize(d)?;
    Ok(serde_json::from_value(value).unwrap_or_default())
}

fn global_slots<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[usize; 2], D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    Ok(serde_json::from_value(value).unwrap_or([0, 1]))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Preferences {
    pub show_top: bool,
    pub show_center: bool,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub dock_size: DockSize,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub dock_theme: DockTheme,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub dock_position: DockPosition,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub dock_labels: DockLabels,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub widgets_layout: WidgetsLayout,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub weather_details: WeatherDetails,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub cinema_layout: CinemaLayout,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub cinema_details: CinemaDetails,
    pub cinema_dock_scale: DockScale,
    #[serde(deserialize_with = "deserialize_dock_horizontal")]
    pub dock_horizontal: DockHorizontal,
    pub compact_cinema_scale: CinemaScale,
    pub wheel_steps: WheelSteps,
    pub panel_focus: crate::state::PanelFocusMode,
    #[serde(deserialize_with = "crate::magic_remote::read_hold")]
    pub hold_ms: u64,
    #[serde(deserialize_with = "crate::magic_remote::read_hover")]
    pub hover_ms: u64,
    #[serde(deserialize_with = "crate::magic_remote::read_leave")]
    pub leave_ms: u64,
    pub pointer_expands: bool,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub motion_profile: MotionProfile,
    #[serde(deserialize_with = "crate::home_visual::read_option")]
    pub motion_curve: MotionCurve,
    pub motion_speed: MotionSpeed,
    pub rotation_enabled: bool,
    #[serde(deserialize_with = "rotation_interval")]
    pub rotation_seconds: u64,
    #[serde(deserialize_with = "global_position")]
    pub global_position: GlobalPosition,
    /// Final mixed-Dock indices for Apps and Settings, kept independently of favorites.
    #[serde(deserialize_with = "global_slots")]
    pub global_slots: [usize; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DockSize {
    Compact,
    #[default]
    Standard,
    Large,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DockHorizontal {
    Stop,
    #[default]
    Sequential,
    WrapRow,
}

fn deserialize_dock_horizontal<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<DockHorizontal, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).unwrap_or_default())
}

impl DockHorizontal {
    pub fn label(self) -> &'static str {
        match self {
            Self::Stop => "evolutionDockHorizontalStop",
            Self::Sequential => "evolutionDockHorizontalSequential",
            Self::WrapRow => "evolutionDockHorizontalWrapRow",
        }
    }

    pub fn stepped(self, increase: bool) -> Self {
        match (self, increase) {
            (Self::Stop | Self::Sequential, false) => Self::Stop,
            (Self::Stop, true) | (Self::WrapRow, false) => Self::Sequential,
            (Self::Sequential | Self::WrapRow, true) => Self::WrapRow,
        }
    }

    fn destination(
        self,
        slot: usize,
        row: std::ops::Range<usize>,
        count: usize,
        right: bool,
    ) -> usize {
        match (self, right) {
            (Self::Sequential, false) => slot.saturating_sub(1),
            (Self::Sequential, true) => (slot + 1).min(count - 1),
            (Self::WrapRow, false) if slot == row.start => row.end - 1,
            (Self::WrapRow, true) if slot + 1 == row.end => row.start,
            (_, false) => slot.saturating_sub(1).max(row.start),
            (_, true) => (slot + 1).min(row.end - 1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DockScale(u8);

impl DockScale {
    const MIN: u8 = 40;
    const MAX: u8 = 100;
    const STEP: u8 = 5;

    pub fn stepped(self, increase: bool) -> Self {
        Self(if increase {
            (self.0 + Self::STEP).min(Self::MAX)
        } else {
            (self.0 - Self::STEP).max(Self::MIN)
        })
    }

    pub fn label(self) -> String {
        format!("{}%", self.0)
    }

    fn contraction(self) -> f32 {
        f32::from(100 - self.0) / 100.
    }
}

impl Default for DockScale {
    fn default() -> Self {
        Self(Self::MIN)
    }
}

impl<'de> Deserialize<'de> for DockScale {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        Ok(value
            .as_u64()
            .and_then(|v| u8::try_from(v).ok())
            .filter(|v| (Self::MIN..=Self::MAX).contains(v) && v.is_multiple_of(Self::STEP))
            .map(Self)
            .unwrap_or_default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CinemaScale(u8);

impl CinemaScale {
    const MIN: u8 = 40;
    const DEFAULT: u8 = 100;
    const STEP: u8 = 10;

    pub fn from_percent(percent: u8) -> Option<Self> {
        ((Self::MIN..=Self::DEFAULT).contains(&percent) && percent.is_multiple_of(Self::STEP))
            .then_some(Self(percent))
    }

    pub fn percent(self) -> u8 {
        self.0
    }

    pub fn stepped(self, increase: bool) -> Self {
        Self(if increase {
            (self.0 + Self::STEP).min(Self::DEFAULT)
        } else {
            (self.0 - Self::STEP).max(Self::MIN)
        })
    }

    pub fn label(self) -> String {
        format!("{}%", self.0)
    }
}

impl Default for CinemaScale {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

impl<'de> Deserialize<'de> for CinemaScale {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        Ok(value
            .as_u64()
            .and_then(|v| u8::try_from(v).ok())
            .and_then(Self::from_percent)
            .unwrap_or_default())
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            show_top: true,
            show_center: true,
            dock_size: DockSize::Standard,
            dock_theme: DockTheme::default(),
            dock_position: DockPosition::default(),
            dock_labels: DockLabels::default(),
            widgets_layout: WidgetsLayout::default(),
            weather_details: WeatherDetails::default(),
            cinema_layout: CinemaLayout::default(),
            cinema_details: CinemaDetails::default(),
            cinema_dock_scale: DockScale::default(),
            dock_horizontal: DockHorizontal::default(),
            compact_cinema_scale: CinemaScale::default(),
            wheel_steps: WheelSteps::default(),
            panel_focus: crate::state::PanelFocusMode::default(),
            hold_ms: crate::magic_remote::Timing::Hold.default_ms(),
            hover_ms: crate::magic_remote::Timing::Hover.default_ms(),
            leave_ms: crate::magic_remote::Timing::Leave.default_ms(),
            pointer_expands: true,
            motion_profile: MotionProfile::Balanced,
            motion_curve: MotionCurve::default(),
            motion_speed: MotionSpeed::default(),
            rotation_enabled: true,
            rotation_seconds: 10,
            global_position: GlobalPosition::Top,
            global_slots: [0, 1],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct WheelSteps(u8);

impl WheelSteps {
    pub fn value(self) -> u8 {
        self.0
    }

    pub fn stepped(self, increase: bool) -> Self {
        Self(if increase {
            (self.0 + 1).min(3)
        } else {
            (self.0 - 1).max(1)
        })
    }
}

impl Default for WheelSteps {
    fn default() -> Self {
        Self(1)
    }
}

impl<'de> Deserialize<'de> for WheelSteps {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        Ok(match value.as_u64() {
            Some(2) => Self(2),
            Some(3) => Self(3),
            _ => Self::default(),
        })
    }
}

fn rotation_interval<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    Ok(value
        .as_u64()
        .filter(|v| (1..=300).contains(v))
        .unwrap_or(10))
}

#[derive(Default)]
pub struct Rotation {
    pub started: Option<Instant>,
    pub elapsed: Duration,
    pub paused: bool,
    interval: Duration,
    selected: i64,
    pub advances: u64,
}

impl Rotation {
    pub fn reset(&mut self) {
        self.started = None;
        self.elapsed = Duration::ZERO;
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed.as_secs_f32() / self.interval.as_secs_f32().max(0.001)).clamp(0., 1.)
    }

    pub fn tick(&mut self, now: Instant, eligible: bool, seconds: u64, selected: i64) -> bool {
        let interval = Duration::from_secs(seconds.clamp(1, 300));
        if self.interval != interval || self.selected != selected {
            self.reset();
        }
        self.interval = interval;
        self.selected = selected;
        if !eligible {
            self.started = None;
            self.paused = true;
            return false;
        }
        if self.paused {
            self.reset();
        }
        self.paused = false;
        let start = *self.started.get_or_insert(now);
        self.elapsed = now.saturating_duration_since(start);
        if self.elapsed >= interval {
            self.reset();
            self.started = Some(now);
            self.selected = selected + 1;
            self.advances += 1;
            return true;
        }
        false
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeatherFixture {
    #[default]
    None,
    Ready,
    Loading,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Area {
    Actions,
    Widgets,
    Cinema,
    Dock,
    Inputs,
}

const WHEEL_IDLE: Duration = Duration::from_millis(800);

struct WheelPartial {
    up: bool,
    steps: u8,
    last_step: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetApp {
    Calendar,
    Weather,
}

#[derive(Default, Serialize)]
pub struct WidgetAppFixture {
    pub calendar: bool,
    pub weather: bool,
}

impl Preferences {
    pub fn card(&self) -> (f32, f32) {
        let width = match self.dock_size {
            DockSize::Compact => 192.,
            DockSize::Standard => 224.,
            DockSize::Large => 256.,
        };
        // Legacy square_icons is deliberately ignored by serde and never written again.
        (width, width / 1.6)
    }

    pub fn columns(&self) -> usize {
        ((1760. + 20.) / (self.card().0 + 20.)).floor() as usize
    }
}

/// An interruptible cubic tween; retarget from the displayed value, never queue animations.
#[derive(Default)]
pub struct Tween {
    pub value: f32,
    from: f32,
    target: f32,
    elapsed: f32,
}

impl Tween {
    #[cfg(test)]
    fn advance(&mut self, target: f32, dt: f32, duration: f32, reduced: bool) -> bool {
        self.advance_eased(target, dt, if reduced { 0. } else { duration }, |elapsed| {
            let t = elapsed / duration;
            if t < 0.5 {
                4. * t.powi(3)
            } else {
                1. - (-2. * t + 2.).powi(3) / 2.
            }
        })
    }

    fn advance_eased(
        &mut self,
        target: f32,
        dt: f32,
        duration: f32,
        ease: impl FnOnce(f32) -> f32,
    ) -> bool {
        let before = self.value;
        if target != self.target {
            self.from = self.value;
            self.target = target;
            self.elapsed = 0.;
        }
        self.elapsed = (self.elapsed + dt.max(0.)).min(duration);
        self.value = if self.elapsed >= duration {
            target
        } else {
            self.from + (target - self.from) * ease(self.elapsed)
        };
        before != self.value
    }

    fn animate(&mut self, target: f32, dt: f32, motion: Motion, part: MotionPart) -> bool {
        self.advance_eased(target, dt, motion.duration(part), |elapsed| {
            motion.progress(part, elapsed)
        })
    }

    pub fn moving(&self) -> bool {
        self.value != self.target
    }
}

#[derive(Default)]
pub struct Home {
    /// Unbounded logical positions preserve direction across the circular boundary.
    pub selected: i64,
    pub first: i64,
    pub expansion: Tween,
    pub carousel: Tween,
    pub backdrop: Tween,
    pub backdrop_previous: Option<usize>,
    backdrop_selected: Option<usize>,
    pub cinema_opacity: Tween,
    pub widgets_opacity: Tween,
    pub focused_card: Option<Target>,
    pub card_focus: Tween,
    pub expanded: bool,
    pub last_dock: Option<Target>,
    favorite_order: Vec<String>,
    /// Last remote Dock viewport and its row's position before scrolling.
    dock_scroll: DockScroll,
    pub dock_in_view: bool,
    pub input_anchor: usize,
    /// Revealed only by vertical navigation, never by pointer hover or cinema expansion.
    pub inputs_revealed: bool,
    pub return_action: Option<GlobalAction>,
    pub moving_dock: Option<Vec<Target>>,
    pub hover: Option<(String, Instant)>,
    pub leave: Option<Instant>,
    pub top_expansion: Tween,
    pub button_focus: [Tween; 3],
    pub top_label: Tween,
    pub top_open: bool,
    pub weather_fixture: WeatherFixture,
    pub widget_apps: WidgetAppFixture,
    pub simulated_widget_open: Option<WidgetApp>,
    pub cinema_scale_override: Option<CinemaScale>,
    pub rotation: Rotation,
    pub rotation_trace_start: u64,
    pub rotation_trace_selected: i64,
    pub pointer_in_cinema: bool,
    pub pointer_target: Option<Target>,
    wheel_partial: Option<WheelPartial>,
    // Area focus is also used outside wheel navigation, e.g. after a widget disappears.
    wheel_entry: bool,
    wheel_viewport: Option<(f32, Tween)>,
    pub suspended: bool,
    /// The SDL loop advances the rotation only after processing pending input.
    pub defer_rotation: bool,
    /// Number of fictional titles for desktop preview. Normal runtime has no Cinema source yet.
    pub preview_catalog_len: Option<usize>,
}

pub fn compact_header(prefs: &crate::model::Preferences, now: time::OffsetDateTime) -> String {
    let language = prefs.language;
    let clock = language.clock(now, prefs.hour_24, prefs.show_seconds);
    let mut label = if prefs.show_greeting {
        let greeting = language.greeting(now.hour(), &prefs.greeting_name);
        format!(
            "{}, {clock}",
            greeting.trim_end_matches(['.', '!', '。', '！'])
        )
    } else {
        clock
    };
    if prefs.show_date {
        label.push_str(" - ");
        label.push_str(&prefs.date_format.format(language, now));
    }
    label
}

#[cfg(test)]
pub fn content(position: i64) -> usize {
    position.rem_euclid(CONTENTS.len() as i64) as usize
}

impl Home {
    pub fn preview() -> Self {
        Self {
            preview_catalog_len: Some(CONTENTS.len()),
            ..Self::default()
        }
    }

    pub fn for_launch(webos: bool, desktop_preview: bool) -> Option<Self> {
        if webos {
            Some(Self::default())
        } else {
            desktop_preview.then(Self::preview)
        }
    }

    pub fn is_preview(&self) -> bool {
        self.preview_catalog_len.is_some()
    }

    pub fn wheel_viewport_moving(&self) -> bool {
        self.wheel_viewport.is_some()
    }

    fn wheel_viewport_offset(&self, target: f32) -> f32 {
        self.wheel_viewport
            .as_ref()
            .map_or(target, |(from, tween)| from + (target - from) * tween.value)
    }

    fn expire_wheel(&mut self, now: Instant) {
        if self
            .wheel_partial
            .as_ref()
            .is_some_and(|partial| now.duration_since(partial.last_step) >= WHEEL_IDLE)
        {
            self.wheel_partial = None;
        }
    }

    pub fn dock_caption_transition(&self) -> f32 {
        self.expansion.value.max(self.top_expansion.value * 0.7)
    }

    pub fn dock_scale(&self, cinema_scale: DockScale) -> f32 {
        let cinema = self.expansion.value * cinema_scale.contraction();
        // Keep the existing contraction while Calendar and Weather occupy Area 2.
        let widgets = self.top_expansion.value * 0.7 * DockScale::default().contraction();
        1. - cinema.max(widgets)
    }

    pub fn set_widgets_open(&mut self, open: bool) {
        self.top_open = open;
        if open {
            self.cancel_hover();
            self.expanded = false;
            self.inputs_revealed = false;
        }
    }

    pub fn title_count(&self) -> usize {
        self.preview_catalog_len.unwrap_or(0).min(CONTENTS.len())
    }

    pub fn title_index(&self, position: i64) -> usize {
        position.rem_euclid(self.title_count().max(1) as i64) as usize
    }

    pub fn step(&mut self, delta: i64) {
        let count = self.title_count();
        if count < 2 {
            return;
        }
        let visible = VISIBLE.min(count) as i64;
        self.selected += delta;
        if self.selected < self.first {
            self.first = self.selected;
        } else if self.selected >= self.first + visible {
            self.first = self.selected - visible + 1;
        }
    }

    pub fn cancel_hover(&mut self) {
        self.hover = None;
        self.leave = None;
    }
}

pub struct Geometry {
    pub viewport: Rect,
    pub offset: f32,
    pub rows: usize,
    pub visible_rows: usize,
    pub empty_hint: Option<Rect>,
    pub center: Rect,
    pub cinema_card_height: f32,
    pub dock: Vec<Rect>,
    pub dock_start: usize,
    pub dock_bounds: Rect,
    pub inputs: Vec<Rect>,
    pub input_start: usize,
    pub scroll: ScrollHints,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollHints {
    pub up: bool,
    pub down: bool,
}

impl ScrollHints {
    pub const UP: Rect = Rect::new(946., 28., 28., 28.);
    pub const DOWN: Rect = Rect::new(946., 1036., 28., 28.);
}

impl Geometry {
    pub fn diagnostic(&self) -> serde_json::Value {
        let r = |r: Rect| [r.x, r.y, r.w, r.h];
        serde_json::json!({ "pane": r(self.dock_bounds), "cards": self.dock.iter().map(|v| r(*v)).collect::<Vec<_>>(), "first": self.dock_start, "center": r(self.center), "cinema_card_height": self.cinema_card_height,
            "viewport": r(self.viewport), "offset": self.offset, "rows": self.rows, "visible_rows": self.visible_rows, "empty_hint": self.empty_hint.map(r),
            "scroll": {"up": self.scroll.up.then(|| r(ScrollHints::UP)), "down": self.scroll.down.then(|| r(ScrollHints::DOWN))} })
    }
}

const DOCK_REST_BOTTOM: f32 = 1024.;
const INPUT_FOCUS_MARGIN: f32 = 8.;
const SCROLL_BOTTOM: f32 = DOCK_REST_BOTTOM + INPUT_FOCUS_MARGIN;
const DOCK_EMPTY_HEIGHT: f32 = 34.;
const DOCK_INPUT_GAP: f32 = 46.;
const INPUT_HEIGHT: f32 = 54.;
pub(crate) const CARD_FOCUS_GROWTH: f32 = 0.05;
pub(crate) const CARD_SHADOW_BLUR: f32 = 24.;
pub(crate) const CARD_SHADOW_DROP: f32 = 6.;
pub(crate) const DOCK_CAPTION_TOP: f32 = 15.;
pub(crate) const DOCK_CAPTION_HEIGHT: f32 = 26.;

#[derive(Default)]
struct DockScroll {
    offset: f32,
    /// Pair the row with a logical ID to rebase it when the layout or order changes.
    anchor: Option<(Target, f32)>,
}

struct DockFlow {
    viewport: Rect,
    pane: Rect,
    count: usize,
    columns: usize,
    rows: usize,
    first_y: f32,
    pitch: f32,
    w: f32,
    h: f32,
    gap: f32,
    focus_above: f32,
    focus_below: f32,
    empty_hint: Option<Rect>,
    input_y: f32,
}

impl DockFlow {
    fn new(
        prefs: &Preferences,
        count: usize,
        empty: bool,
        scale: f32,
        actions_bottom: f32,
        upper_bottom: f32,
        caption: bool,
    ) -> Self {
        let top = actions_bottom + 4.;
        let viewport = Rect::new(0., top, 1920., SCROLL_BOTTOM - top);
        let columns = prefs.columns();
        let rows = count.div_ceil(columns);
        let (w, h) = prefs.card();
        let (w, h, gap) = (w * scale, h * scale, 20. * scale);
        let pitch = h + 48. * scale;
        let grid_height = if rows == 0 {
            0.
        } else {
            (rows - 1) as f32 * pitch + h
        };
        let grow = h * CARD_FOCUS_GROWTH;
        let focus_above = grow + CARD_SHADOW_BLUR - CARD_SHADOW_DROP;
        let shadow_below = grow + CARD_SHADOW_BLUR + CARD_SHADOW_DROP;
        let caption_below = DOCK_CAPTION_TOP + DOCK_CAPTION_HEIGHT;
        let focus_below = if caption {
            shadow_below.max(caption_below)
        } else {
            shadow_below
        };
        let padding = if rows == 0 && empty {
            20.
        } else {
            (42. * scale).max(shadow_below.max(caption_below))
        };
        let hint_height = if empty { DOCK_EMPTY_HEIGHT } else { 0. };
        let insets = 2. * padding + hint_height;
        let dock_bottom = match prefs.dock_position {
            DockPosition::Standard => DOCK_REST_BOTTOM,
            DockPosition::Raised => 1080. - 120.,
        };
        let bottom_aligned = dock_bottom - grid_height - insets;
        let single_row_top = dock_bottom - grid_height.min(h) - insets;
        let pane_top = bottom_aligned.max(upper_bottom.min(single_row_top));
        let first_y = pane_top + padding + hint_height;
        let grid_width = if rows == 0 {
            0.
        } else {
            count.min(columns) as f32 * (w + gap) - gap
        };
        let width = (grid_width + 2. * padding).max(if empty || rows == 0 { 680. } else { 0. });
        let pane = Rect::new((1920. - width) / 2., pane_top, width, grid_height + insets);
        let empty_hint = empty.then(|| {
            Rect::new(
                80.,
                pane_top + if rows == 0 { padding } else { 9. },
                1760.,
                DOCK_EMPTY_HEIGHT,
            )
        });
        Self {
            viewport,
            pane,
            count,
            columns,
            rows,
            first_y,
            pitch,
            w,
            h,
            gap,
            focus_above,
            focus_below,
            empty_hint,
            input_y: (pane.y + pane.h).max(dock_bottom) + DOCK_INPUT_GAP,
        }
    }

    fn row_y(&self, row: usize) -> f32 {
        self.first_y + row as f32 * self.pitch
    }

    fn row_slots(&self, row: usize) -> std::ops::Range<usize> {
        let first = row * self.columns;
        first..(first + self.columns).min(self.count)
    }

    fn card_rect(&self, slot: usize, offset: f32) -> Rect {
        let row = slot / self.columns;
        let slots = self.row_slots(row);
        let width = slots.len() as f32 * (self.w + self.gap) - self.gap;
        Rect::new(
            (1920. - width) / 2. + (slot - slots.start) as f32 * (self.w + self.gap),
            self.row_y(row) - offset,
            self.w,
            self.h,
        )
    }

    fn destination(
        &self,
        slot: usize,
        direction: Direction,
        policy: DockHorizontal,
    ) -> Option<usize> {
        let row = slot / self.columns;
        let adjacent = match direction {
            Direction::Left | Direction::Right => {
                return Some(policy.destination(
                    slot,
                    self.row_slots(row),
                    self.count,
                    matches!(direction, Direction::Right),
                ));
            }
            Direction::Up => row.checked_sub(1)?,
            Direction::Down if row + 1 < self.rows => row + 1,
            Direction::Down => return None,
        };
        let source = self.card_rect(slot, 0.).center().0;
        closest_dock_slot(
            source,
            self.row_slots(adjacent)
                .map(|candidate| (candidate, self.card_rect(candidate, 0.).center().0)),
        )
    }

    fn focus_range(&self, row: usize) -> (f32, f32) {
        let y = self.row_y(row);
        (y - self.focus_above, y + self.h + self.focus_below)
    }

    fn dock_scroll_limit(&self) -> f32 {
        self.rows.checked_sub(1).map_or(0., |last| {
            (self.focus_range(last).1 - SCROLL_BOTTOM).max(0.)
        })
    }

    fn input_scroll_limit(&self) -> f32 {
        self.input_y + INPUT_HEIGHT + INPUT_FOCUS_MARGIN - SCROLL_BOTTOM
    }

    fn reveal(&self, slot: usize, offset: f32) -> f32 {
        let (top, bottom) = self.focus_range(slot / self.columns);
        let last = (top - self.viewport.y).clamp(0., self.dock_scroll_limit());
        // The legal interval contains every offset at which the entire focus fits.
        // If it cannot fit, keep its top at the fixed header rather than resizing it.
        let first = (bottom - SCROLL_BOTTOM).clamp(0., last);
        offset.clamp(first, last)
    }

    fn restored_offset(&self, items: &[Target], scroll: &DockScroll) -> f32 {
        let rebase = scroll.anchor.as_ref().and_then(|(anchor, old_y)| {
            let index = items.iter().position(|item| item == anchor)?;
            Some(self.row_y(index / self.columns) - old_y)
        });
        (scroll.offset + rebase.unwrap_or(0.)).clamp(0., self.dock_scroll_limit())
    }

    fn layout(&self, offset: f32, input_count: usize, input_anchor: usize) -> Geometry {
        let start = self.viewport.y + offset;
        let end = SCROLL_BOTTOM + offset;
        let first_row = (((start - self.first_y - self.h - self.focus_below) / self.pitch).floor()
            as usize)
            .min(self.rows);
        let end_row =
            (((end - self.first_y + self.focus_above) / self.pitch).ceil() as usize).min(self.rows);
        let mut dock = Vec::new();
        let mut visible_rows = 0;
        let mut dock_start = first_row * self.columns;
        for row in first_row..end_row {
            let (top, bottom) = self.focus_range(row);
            if bottom <= start || top >= end {
                continue;
            }
            if dock.is_empty() {
                dock_start = row * self.columns;
            }
            visible_rows += usize::from(top >= start && bottom <= end);
            dock.extend(self.row_slots(row).map(|slot| self.card_rect(slot, offset)));
        }
        let visible = if self.input_y < end {
            input_count.min(INPUTS_PER_WINDOW)
        } else {
            0
        };
        let input_start = (input_anchor / INPUTS_PER_WINDOW * INPUTS_PER_WINDOW)
            .min(input_count.saturating_sub(visible));
        let width = visible as f32 * 250. - 14. + 146.;
        let inputs = (0..visible)
            .map(|i| {
                Rect::new(
                    (1920. - width) / 2. + 146. + i as f32 * 250.,
                    self.input_y - offset,
                    236.,
                    INPUT_HEIGHT,
                )
            })
            .collect();
        let shifted = |r: Rect| Rect::new(r.x, r.y - offset, r.w, r.h);
        let hidden_above = self.rows > 0 && self.focus_range(0).0 < start
            || self.empty_hint.is_some_and(|hint| hint.y < start);
        let hidden_below = self.rows > 0 && self.focus_range(self.rows - 1).1 > end
            || self.empty_hint.is_some_and(|hint| hint.y + hint.h > end)
            || input_count > 0 && self.input_y + INPUT_HEIGHT > end;
        Geometry {
            viewport: self.viewport,
            offset,
            rows: self.rows,
            visible_rows,
            center: Rect::default(),
            cinema_card_height: 0.,
            dock,
            dock_start,
            dock_bounds: shifted(self.pane),
            empty_hint: self.empty_hint.map(shifted),
            inputs,
            input_start,
            scroll: ScrollHints {
                up: hidden_above,
                down: hidden_below,
            },
        }
    }
}

fn closest_dock_slot(source: f32, candidates: impl Iterator<Item = (usize, f32)>) -> Option<usize> {
    candidates
        .min_by(|(_, x), (_, y)| {
            let distance = |center: f32| (f64::from(center) - f64::from(source)).abs();
            distance(*x)
                .total_cmp(&distance(*y))
                .then_with(|| x.total_cmp(y))
        })
        .map(|(slot, _)| slot)
}

fn eligible_dock_favorites(
    prefs: &crate::model::Preferences,
    apps: &[crate::model::App],
) -> Vec<String> {
    let available: std::collections::HashSet<_> = apps.iter().map(|app| app.id.as_str()).collect();
    prefs
        .favorites(apps)
        .into_iter()
        .filter(|id| available.contains(id.as_str()) && !prefs.hidden.contains(id))
        .collect()
}

impl State {
    pub fn evolution_active(&self) -> bool {
        self.evolution.is_some() && self.page == Page::Home && self.overlay.is_none()
    }

    pub fn evolution_dock_items(&self) -> Vec<Target> {
        if let Some(draft) = self
            .evolution
            .as_ref()
            .and_then(|home| home.moving_dock.as_ref())
        {
            return draft.clone();
        }
        let mut items: Vec<_> = eligible_dock_favorites(&self.preferences, &self.inventory.apps)
            .into_iter()
            .map(Target::App)
            .collect();
        let prefs = &self.preferences.evolution_home;
        if prefs.global_position == GlobalPosition::Dock {
            let mut actions = GlobalAction::ALL;
            actions.sort_by_key(|action| prefs.global_slots[action.index()]);
            let first = prefs.global_slots[actions[0].index()].min(items.len());
            let second = prefs.global_slots[actions[1].index()].clamp(first + 1, items.len() + 1);
            items.insert(first, Target::EvolutionGlobal(actions[0]));
            items.insert(second, Target::EvolutionGlobal(actions[1]));
        }
        items
    }

    pub(crate) fn evolution_snapshot_dock_order(&mut self) {
        if let Some(home) = &mut self.evolution {
            home.favorite_order = self.preferences.favorites(&self.inventory.apps);
        }
    }

    pub(crate) fn evolution_reconcile_dock(&mut self) {
        if self.evolution.is_none() || self.evolution_moving() {
            return;
        }
        let eligible = eligible_dock_favorites(&self.preferences, &self.inventory.apps);
        let active = self.evolution_active();
        let focused = match &self.focus {
            Target::App(id) if active => Some(id),
            _ => None,
        };
        let home = self.evolution.as_mut().unwrap();
        let remembered = match &home.last_dock {
            Some(Target::App(id)) => Some(id),
            _ => None,
        };
        let lost_focus = focused.is_some_and(|id| !eligible.contains(id));
        if let Some(id) = focused.or(remembered) {
            let next = if eligible.contains(id) {
                Some(id)
            } else {
                let available: std::collections::HashSet<_> = eligible.iter().collect();
                home.favorite_order
                    .iter()
                    .position(|old| old == id)
                    .and_then(|position| {
                        let (earlier, later) = home.favorite_order.split_at(position);
                        later
                            .iter()
                            .skip(1)
                            .chain(earlier.iter().rev())
                            .find(|old| available.contains(old))
                    })
                    .or_else(|| remembered.filter(|old| available.contains(old)))
                    .or_else(|| eligible.first())
            };
            home.last_dock = next.cloned().map(Target::App);
        }
        home.favorite_order = self.preferences.favorites(&self.inventory.apps);
        if lost_focus {
            home.pointer_target = None;
            self.pressed = None;
            self.evolution_focus_dock(self.evolution_dock());
        }
    }

    pub fn evolution_inputs(&self) -> Vec<&crate::model::Input> {
        self.inventory
            .inputs
            .iter()
            .filter(|input| self.preferences.input_visible(input))
            .collect()
    }

    pub fn evolution_top_targets(&self) -> Vec<Target> {
        let mut targets = Vec::new();
        if self.evolution_has_today() {
            targets.push(Target::TodayCard);
        }
        if self.preferences.evolution_home.global_position == GlobalPosition::Top {
            targets.extend(GlobalAction::ALL.map(Target::EvolutionGlobal));
        }
        targets
    }

    /// Legacy widgets remain available to the original desktop harness.
    pub fn legacy_widgets_enabled(&self) -> bool {
        self.evolution.is_none()
    }

    pub fn widget_center(&self) -> Option<crate::model::Center> {
        use crate::model::Center;
        if self.legacy_widgets_enabled() {
            self.preferences.center()
        } else {
            self.preferences
                .center_among(&[Center::Shows, Center::Apps])
        }
    }

    pub fn crypto_in_today(&self) -> bool {
        self.legacy_widgets_enabled() && self.preferences.crypto_in_today()
    }

    pub fn evolution_widget_in_today(&self, widget: crate::model::Center) -> bool {
        use crate::model::Center;
        let prefs = &self.preferences;
        if self.legacy_widgets_enabled() {
            prefs.in_today(widget)
        } else {
            widget != Center::Cs2 && prefs.shown(widget)
        }
    }

    pub fn evolution_has_today(&self) -> bool {
        use crate::model::Center;
        [Center::Shows, Center::Apps]
            .into_iter()
            .any(|widget| self.evolution_widget_in_today(widget))
    }

    pub fn evolution_dock(&self) -> Target {
        let items = self.evolution_dock_items();
        self.evolution
            .as_ref()
            .and_then(|home| home.last_dock.as_ref())
            .filter(|target| items.contains(target))
            .or_else(|| items.iter().find(|target| matches!(target, Target::App(_))))
            .or_else(|| items.first())
            .cloned()
            .unwrap_or(if self.error.is_some() {
                Target::Retry
            } else {
                Target::EvolutionGlobal(GlobalAction::Apps)
            })
    }

    pub fn evolution_focus_dock(&mut self, target: Target) {
        let items = self.evolution_dock_items();
        let index = items.iter().position(|item| *item == target);
        if let Some(index) = index {
            self.evolution_remember_dock(&items, index);
        }
        if !self.evolution_moving() {
            self.evolution_snapshot_dock_order();
        }
        if let Some(home) = self.evolution.as_mut() {
            home.wheel_entry = false;
            home.wheel_viewport = None;
            home.inputs_revealed = false;
            home.top_open = false;
            home.dock_in_view = index.is_some();
            if index.is_some() {
                home.last_dock = Some(target.clone());
            } else if items.is_empty() {
                home.dock_scroll = DockScroll::default();
            }
        }
        self.focus = target;
    }

    pub fn evolution_moving(&self) -> bool {
        self.evolution
            .as_ref()
            .is_some_and(|home| home.moving_dock.is_some())
    }

    pub fn evolution_start_move(&mut self, target: Target) {
        let items = self.evolution_dock_items();
        if items.contains(&target) {
            self.evolution_discard_wheel();
            self.evolution_focus_dock(target);
            let home = self.evolution.as_mut().unwrap();
            home.moving_dock = Some(items);
            home.expanded = false;
            home.cancel_hover();
            self.pointer = false;
            self.dirty = true;
        }
    }

    pub fn evolution_finish_move(&mut self, save: bool) -> Vec<crate::state::Effect> {
        let Some(items) = self
            .evolution
            .as_mut()
            .and_then(|home| home.moving_dock.take())
        else {
            return vec![];
        };
        if save {
            for (index, item) in items.iter().enumerate() {
                if let Target::EvolutionGlobal(action) = item {
                    self.preferences.evolution_home.global_slots[action.index()] = index;
                }
            }
            if matches!(self.focus, Target::App(_)) {
                let mut favorites = self.preferences.favorites(&self.inventory.apps);
                let visible = self.apps();
                let mut reordered = items.iter().filter_map(|item| match item {
                    Target::App(id) => Some(id),
                    _ => None,
                });
                for id in &mut favorites {
                    if visible.iter().any(|app| app.id == *id)
                        && let Some(next) = reordered.next()
                    {
                        id.clone_from(next);
                    }
                }
                self.preferences.favorites = Some(favorites);
            }
        }
        self.evolution_focus_dock(self.focus.clone());
        self.dirty = true;
        if save {
            vec![crate::state::Effect::Save]
        } else {
            vec![]
        }
    }

    fn evolution_center_target(&self) -> Target {
        let home = self.evolution.as_ref().unwrap();
        let i = home.title_index(home.selected);
        Target::EvolutionContent(format!("{}:{i}", CONTENTS[i].0))
    }

    pub(crate) fn evolution_set_catalog_len(&mut self, count: usize) {
        let home = self.evolution.as_mut().unwrap();
        let previous_count = home.title_count();
        let selected = home.title_index(home.selected);
        home.preview_catalog_len = Some(count);
        let count = home.title_count();
        if previous_count == count {
            return;
        }

        let slot =
            (home.selected - home.first).clamp(0, VISIBLE.min(count).saturating_sub(1) as i64);
        // A confirmed removal in the preview removes a suffix of the ordered fixture. The next
        // valid title on the circular row is the first one when the previous ID was in that suffix.
        home.selected = if selected < count { selected as i64 } else { 0 };
        home.first = home.selected - slot;
        home.carousel = Tween {
            value: home.first as f32,
            target: home.first as f32,
            ..Tween::default()
        };
        home.backdrop_selected = (count > 0).then_some(home.selected as usize);
        home.backdrop_previous = None;
        home.rotation.reset();
        home.cancel_hover();
        let removed = |target: &Target| match target {
            Target::EvolutionContent(id) => {
                Self::evolution_content_index(id).is_none_or(|index| index >= count)
            }
            Target::EvolutionArea(Area::Cinema) => count == 0,
            _ => false,
        };
        if home.pointer_target.as_ref().is_some_and(removed) || count == 0 {
            home.pointer_target = None;
            home.pointer_in_cinema = false;
        }
        if count == 0 {
            home.expanded = false;
        }
        if self.pressed.as_ref().is_some_and(removed) {
            self.pressed = None;
        }
        if self.evolution_active() && removed(&self.focus) {
            if count == 0 {
                self.evolution_focus_dock(self.evolution_dock());
            } else {
                self.focus = self.evolution_center_target();
            }
        }
        self.dirty = true;
    }

    pub(crate) fn evolution_has_cinema(&self) -> bool {
        self.preferences.evolution_home.show_center
            && self.evolution.as_ref().unwrap().title_count() > 0
    }

    pub(crate) fn evolution_content_index(id: &str) -> Option<usize> {
        let (key, index) = id.rsplit_once(':')?;
        let index = index.parse::<usize>().ok()?;
        (CONTENTS.get(index)?.0 == key).then_some(index)
    }

    pub fn evolution_widget_eligible(&self, app: WidgetApp) -> bool {
        let home = self.evolution.as_ref().unwrap();
        let prefs = &self.preferences;
        prefs.evolution_home.show_top
            && home.widget_apps.weather
            && match app {
                WidgetApp::Calendar => home.widget_apps.calendar,
                WidgetApp::Weather => prefs.show_weather,
            }
    }

    fn evolution_first_widget(&self) -> Option<Target> {
        [WidgetApp::Calendar, WidgetApp::Weather]
            .into_iter()
            .find(|app| self.evolution_widget_eligible(*app))
            .map(Target::EvolutionWidget)
    }

    pub fn evolution_reconcile_widgets(&mut self) {
        if let Target::EvolutionWidget(app) = self.focus
            && !self.evolution_widget_eligible(app)
        {
            // With two blocks, the sole remaining block is the successor or predecessor.
            self.focus = self
                .evolution_first_widget()
                .unwrap_or(Target::EvolutionArea(Area::Widgets));
            let home = self.evolution.as_mut().unwrap();
            home.pointer_target = None;
            self.pressed = None;
            self.dirty = true;
        }
    }

    pub fn evolution_activate_widget(&mut self, app: WidgetApp) {
        if !self.evolution_widget_eligible(app) {
            self.evolution_reconcile_widgets();
            return;
        }
        self.evolution.as_mut().unwrap().simulated_widget_open = Some(app);
        let name = match app {
            WidgetApp::Calendar => "Calendário",
            WidgetApp::Weather => "Clima",
        };
        self.notify(format!("Preview · abertura simulada: {name}"));
    }

    pub fn evolution_discard_wheel(&mut self) {
        if let Some(home) = &mut self.evolution {
            home.wheel_partial = None;
        }
    }

    pub fn evolution_pointer_motion(&mut self, xrel: i32, yrel: i32) -> bool {
        if !self.evolution_active() {
            return true;
        }
        if xrel == 0 && yrel == 0 {
            return false;
        }
        self.evolution_discard_wheel();
        true
    }

    pub fn evolution_wheel(&mut self, y: i32, now: Instant) -> bool {
        if !self.evolution_active() || self.evolution_moving() || self.moving.is_some() {
            self.evolution_discard_wheel();
            return false;
        }
        if y == 0 {
            return false;
        }
        let top = self.evolution_top_targets();
        let mut current = match &self.focus {
            Target::EvolutionArea(area) => *area,
            Target::EvolutionWidget(_) => Area::Widgets,
            Target::EvolutionContent(_) => Area::Cinema,
            Target::Input(_) => Area::Inputs,
            target if top.contains(target) => Area::Actions,
            _ => Area::Dock,
        };
        let areas = [
            (Area::Actions, !top.is_empty()),
            (Area::Widgets, self.preferences.evolution_home.show_top),
            (Area::Cinema, self.evolution_has_cinema()),
            (Area::Dock, true),
            (Area::Inputs, !self.evolution_inputs().is_empty()),
        ];
        let up = y > 0;
        let threshold = self.preferences.evolution_home.wheel_steps.value();
        let animate = Motion::saved(&self.preferences).profile.geometric();
        let from = animate.then(|| self.evolution_geometry().offset);
        let mut changed_area = false;
        let home = self.evolution.as_mut().unwrap();
        home.expire_wheel(now);
        let mut steps = y.unsigned_abs();
        while steps != 0 {
            let next = if up {
                areas
                    .iter()
                    .rfind(|(area, enabled)| *enabled && (*area as u8) < current as u8)
            } else {
                areas
                    .iter()
                    .find(|(area, enabled)| *enabled && (*area as u8) > current as u8)
            };
            let Some(&(area, _)) = next else {
                home.wheel_partial = None;
                break;
            };
            let previous = home
                .wheel_partial
                .take()
                .filter(|partial| partial.up == up && partial.steps < threshold)
                .map_or(0, |partial| partial.steps);
            let needed = u32::from(threshold - previous);
            if steps < needed {
                home.wheel_partial = Some(WheelPartial {
                    up,
                    steps: previous + steps as u8,
                    last_step: now,
                });
                break;
            }
            steps -= needed;
            current = area;
            self.focus = Target::EvolutionArea(area);
            self.pressed = None;
            home.wheel_entry = true;
            home.cancel_hover();
            home.pointer_target = None;
            home.pointer_in_cinema = false;
            home.set_widgets_open(area == Area::Widgets);
            home.expanded = area == Area::Cinema;
            home.inputs_revealed = area == Area::Inputs;
            home.dock_in_view = area == Area::Dock;
            changed_area = true;
        }
        if changed_area {
            home.wheel_viewport = None;
            if let Some(from) = from
                && from != self.evolution_geometry().offset
            {
                let mut progress = Tween::default();
                progress.animate(
                    1.,
                    0.,
                    Motion::saved(&self.preferences),
                    MotionPart::Collapse,
                );
                self.evolution.as_mut().unwrap().wheel_viewport = Some((from, progress));
            }
            self.dirty = true;
        }
        true
    }

    pub fn evolution_direction(&mut self, direction: Direction) {
        let home = self.evolution.as_mut().unwrap();
        let wheel_entry = std::mem::take(&mut home.wheel_entry);
        home.pointer_in_cinema = false;
        home.pointer_target = None;
        self.evolution_reconcile_widgets();
        self.evolution.as_mut().unwrap().cancel_hover();
        let prefs = &self.preferences.evolution_home;
        let dock = self.evolution_dock_items();
        let columns = prefs.columns();
        let top = self.evolution_top_targets();
        let dock_focus = self.evolution_dock();
        let inputs: Vec<_> = self
            .evolution_inputs()
            .iter()
            .map(|input| Target::Input(input.id.clone()))
            .collect();
        let top_entry = top.first().cloned();
        let widgets = self.evolution_first_widget();
        let center = self
            .evolution_has_cinema()
            .then(|| self.evolution_center_target());
        let from_dock_up = || center.clone().or(widgets.clone()).or(top_entry.clone());
        let from_widgets_down = || {
            center.clone().or_else(|| {
                if dock.is_empty() && self.error.is_none() {
                    inputs.first().cloned().or(top_entry.clone())
                } else {
                    Some(dock_focus.clone())
                }
            })
        };

        let next = if let Some(i) = dock.iter().position(|target| *target == self.focus) {
            if self.evolution_moving() {
                let index = match direction {
                    Direction::Left => Some(i.saturating_sub(1)),
                    Direction::Right => Some((i + 1).min(dock.len() - 1)),
                    Direction::Up => i.checked_sub(columns),
                    Direction::Down if (i / columns + 1) * columns < dock.len() => {
                        Some((i + columns).min(dock.len() - 1))
                    }
                    Direction::Down => None,
                };
                if let Some(index) = index {
                    let draft = self
                        .evolution
                        .as_mut()
                        .unwrap()
                        .moving_dock
                        .as_mut()
                        .unwrap();
                    let item = draft.remove(i);
                    draft.insert(index, item);
                    self.evolution_focus_dock(self.focus.clone());
                }
                return;
            }
            self.evolution_dock_destination(&dock, i, direction)
                .map(|index| dock[index].clone())
                .or_else(|| match direction {
                    Direction::Up => from_dock_up(),
                    Direction::Down => inputs
                        .get(
                            self.evolution
                                .as_ref()
                                .unwrap()
                                .input_anchor
                                .min(inputs.len().saturating_sub(1)),
                        )
                        .cloned(),
                    _ => None,
                })
        } else if let Some(i) = top.iter().position(|target| *target == self.focus) {
            match direction {
                Direction::Left => top.get(i.saturating_sub(1)).cloned(),
                Direction::Right => top.get(i + 1).cloned(),
                Direction::Down => widgets.clone().or(center.clone()).or_else(|| {
                    if dock.is_empty() && self.error.is_none() {
                        inputs.first().cloned()
                    } else {
                        Some(dock_focus)
                    }
                }),
                Direction::Up => None,
            }
        } else if let Some(i) = inputs.iter().position(|target| *target == self.focus) {
            match direction {
                Direction::Left => inputs.get(i.saturating_sub(1)).cloned(),
                Direction::Right => inputs.get(i + 1).cloned(),
                Direction::Up => Some(dock_focus),
                Direction::Down => None,
            }
        } else {
            match (&self.focus, direction) {
                (Target::EvolutionContent(_), Direction::Left | Direction::Right) => {
                    self.evolution.as_mut().unwrap().step(
                        if matches!(direction, Direction::Left) {
                            -1
                        } else {
                            1
                        },
                    );
                    Some(self.evolution_center_target())
                }
                (Target::EvolutionContent(_), Direction::Up) => widgets.or(top_entry),
                (Target::EvolutionWidget(WidgetApp::Calendar), Direction::Right)
                    if self.evolution_widget_eligible(WidgetApp::Weather) =>
                {
                    Some(Target::EvolutionWidget(WidgetApp::Weather))
                }
                (Target::EvolutionWidget(WidgetApp::Weather), Direction::Left)
                    if self.evolution_widget_eligible(WidgetApp::Calendar) =>
                {
                    widgets
                }
                (Target::EvolutionWidget(WidgetApp::Calendar), Direction::Up) => top_entry,
                (Target::EvolutionWidget(WidgetApp::Weather), Direction::Up) => top.last().cloned(),
                (Target::EvolutionWidget(_), Direction::Down) => from_widgets_down(),
                (Target::EvolutionWidget(_), _) => None,
                (Target::EvolutionArea(Area::Actions), _) => top_entry,
                (Target::EvolutionArea(Area::Widgets), _) if wheel_entry || widgets.is_some() => {
                    widgets
                }
                (Target::EvolutionArea(Area::Widgets), Direction::Up) => top_entry,
                (Target::EvolutionArea(Area::Widgets), Direction::Down) => from_widgets_down(),
                (Target::EvolutionArea(Area::Widgets), _) => None,
                (Target::EvolutionArea(Area::Cinema), _) if center.is_some() => {
                    let home = self.evolution.as_mut().unwrap();
                    home.selected = home.first;
                    Some(self.evolution_center_target())
                }
                (Target::EvolutionArea(Area::Cinema), _) => None,
                (Target::EvolutionArea(Area::Dock), _) if wheel_entry => dock
                    .iter()
                    .find(|target| matches!(target, Target::App(_)))
                    .cloned(),
                (Target::EvolutionArea(Area::Dock), _) => Some(dock_focus),
                (Target::EvolutionArea(Area::Inputs), _) => inputs.first().cloned(),
                (Target::EvolutionContent(_), Direction::Down)
                    if dock.is_empty() && self.error.is_none() =>
                {
                    inputs.first().cloned().or(top_entry)
                }
                (Target::Browse | Target::Retry, Direction::Down) => inputs.first().cloned(),
                (_, Direction::Down) => Some(dock_focus),
                (_, Direction::Up) => from_dock_up(),
                _ => None,
            }
        };
        if let Some(next) = next {
            let input = inputs.iter().position(|target| *target == next);
            if dock.contains(&next) {
                self.evolution_focus_dock(next);
            } else {
                self.focus = next;
                self.evolution.as_mut().unwrap().dock_in_view = false;
            }
            let home = self.evolution.as_mut().unwrap();
            home.wheel_viewport = None;
            home.expanded = matches!(self.focus, Target::EvolutionContent(_));
            if home.expanded {
                home.rotation.paused = true;
                home.rotation.started = None;
            }
            home.set_widgets_open(matches!(self.focus, Target::EvolutionWidget(_)));
            home.inputs_revealed = input.is_some();
            if let Some(input) = input {
                home.input_anchor = input;
            }
        }
    }
    pub fn evolution_hover_gap(&mut self, position: Option<(f32, f32)>, now: Instant) {
        if !self.evolution_active() || !self.pointer {
            return;
        }
        if self.evolution.as_ref().unwrap().wheel_entry {
            return;
        }
        if self.evolution.as_ref().unwrap().top_open
            && position.is_some_and(|(x, y)| {
                let header = header_geometry(self);
                x >= header.clock.x
                    && x <= header.weather.x + header.weather.w
                    && y >= header.clock.y
                    && y < header.bottom
            })
        {
            if matches!(self.focus, Target::EvolutionWidget(_)) {
                self.focus = Target::EvolutionArea(Area::Widgets);
                self.evolution.as_mut().unwrap().pointer_target = None;
                self.dirty = true;
            }
            return;
        }
        if matches!(
            self.focus,
            Target::EvolutionWidget(_) | Target::EvolutionArea(Area::Widgets)
        ) {
            self.focus = Target::EvolutionArea(Area::Dock);
            self.dirty = true;
        }
        self.evolution_pointer(None, now);
    }

    pub fn evolution_pointer(&mut self, target: Option<&Target>, now: Instant) {
        if !self.evolution_active() {
            return;
        }
        if !self.pointer && target.is_none() {
            // Merely leaving a region cannot collapse an active directional selection.
            return;
        }
        let home = self.evolution.as_mut().unwrap();
        if target.is_some() {
            home.wheel_entry = false;
        }
        let open = matches!(target, Some(Target::EvolutionWidget(_)));
        self.dirty |= open != home.top_open;
        home.set_widgets_open(open);
        home.pointer_in_cinema = matches!(target, Some(Target::EvolutionContent(_)));
        let changed_target = home.pointer_target.as_ref() != target;
        home.pointer_target = target.cloned();
        if home.pointer_in_cinema {
            home.rotation.paused = true;
            home.rotation.started = None;
        }
        match target {
            Some(Target::EvolutionContent(id)) => {
                home.leave = None;
                if changed_target {
                    home.hover = Some((id.clone(), now));
                }
            }
            _ => {
                home.hover = None;
                if home.expanded {
                    home.leave.get_or_insert(now);
                }
            }
        }
    }

    pub fn evolution_pointer_hidden(&mut self, now: Instant) {
        if self.pointer {
            self.evolution_hover_gap(None, now);
        }
        if let Some(home) = &mut self.evolution {
            home.hover = None;
            home.pointer_target = None;
            home.pointer_in_cinema = false;
        }
    }

    pub fn evolution_tick(&mut self, dt: f32) {
        self.evolution_tick_at(dt, Instant::now());
    }

    pub(crate) fn evolution_tick_at(&mut self, dt: f32, now: Instant) {
        use crate::magic_remote::Timing;
        if !self.evolution_active() {
            if let Some(home) = self.evolution.as_mut() {
                home.wheel_partial = None;
                home.rotation.paused = true;
                home.rotation.started = None;
                home.cancel_hover();
                home.pointer_target = None;
                home.pointer_in_cinema = false;
            }
            return;
        }
        self.evolution_reconcile_widgets();
        let prefs = &self.preferences.evolution_home;
        let focused = if self.pointer {
            self.evolution.as_ref().unwrap().pointer_target.as_ref()
        } else {
            Some(&self.focus)
        };
        let selected_button = self
            .evolution_top_targets()
            .iter()
            .position(|t| focused == Some(t));
        let focused_card = focused
            .filter(|target| {
                matches!(
                    target,
                    Target::App(_) | Target::EvolutionContent(_) | Target::EvolutionGlobal(_)
                )
            })
            .cloned();
        let home = self.evolution.as_mut().unwrap();
        home.expire_wheel(now);
        let eligible = prefs.rotation_enabled
            && !home.suspended
            && prefs.show_center
            && home.title_count() > 1
            && !home.top_open
            && self.pressed.is_none()
            && !home.pointer_in_cinema
            && !matches!(self.focus, Target::EvolutionArea(Area::Cinema))
            && (self.pointer || !matches!(self.focus, Target::EvolutionContent(_)));
        let before = home.rotation.progress();
        let advanced = !home.defer_rotation
            && home
                .rotation
                .tick(now, eligible, prefs.rotation_seconds, home.selected);
        if advanced {
            home.step(1);
        }
        self.dirty |= advanced || (home.expansion.value > 0. && home.rotation.progress() != before);
        // No resize while a pointer press is captured, including between down and release.
        if self.pressed.is_some() && self.pointer {
            return;
        }
        if self.pointer {
            if let Some((id, since)) = &home.hover {
                let index =
                    Self::evolution_content_index(id).filter(|index| *index < home.title_count());
                let same_card = matches!(&home.pointer_target,
                    Some(Target::EvolutionContent(target)) if target == id);
                if !prefs.show_center || home.title_count() == 0 || !same_card {
                    home.hover = None;
                } else if let Some(index) = index {
                    if now.duration_since(*since)
                        >= Duration::from_millis(Timing::Hover.normalize(prefs.hover_ms))
                    {
                        let selected = home.first
                            + (index as i64 - home.first).rem_euclid(home.title_count() as i64);
                        if home.selected != selected {
                            home.rotation.reset();
                            self.dirty = true;
                        }
                        home.selected = selected;
                        home.expanded |= prefs.pointer_expands;
                        home.hover = None;
                    }
                } else {
                    home.hover = None;
                }
            }
            if home.leave.is_some_and(|since| {
                now.duration_since(since)
                    >= Duration::from_millis(Timing::Leave.normalize(prefs.leave_ms))
            }) {
                home.expanded = false;
                home.leave = None;
            }
        }
        let motion = Motion::saved(&self.preferences);
        if home.focused_card != focused_card {
            home.focused_card = focused_card;
            home.card_focus = Tween::default();
        }
        let focus_part = if motion.profile == MotionProfile::Reduced {
            MotionPart::Backdrop
        } else {
            MotionPart::Focus
        };
        self.dirty |= home.card_focus.animate(1., dt, motion, focus_part);
        let part = |expanded| {
            if expanded {
                MotionPart::Expand
            } else {
                MotionPart::Collapse
            }
        };
        if let Some((_, progress)) = &mut home.wheel_viewport {
            self.dirty |= progress.animate(1., dt, motion, MotionPart::Collapse);
            if !progress.moving() {
                home.wheel_viewport = None;
            }
        }
        self.dirty |=
            home.top_expansion
                .animate(f32::from(home.top_open), dt, motion, part(home.top_open));
        self.dirty |= home.top_label.animate(
            f32::from(selected_button.is_some()),
            dt,
            motion,
            MotionPart::Focus,
        );
        for (index, tween) in home.button_focus.iter_mut().enumerate() {
            self.dirty |= tween.animate(
                f32::from(selected_button == Some(index)),
                dt,
                motion,
                focus_part,
            );
        }
        self.dirty |=
            home.expansion
                .animate(f32::from(home.expanded), dt, motion, part(home.expanded));
        self.dirty |=
            home.cinema_opacity
                .animate(f32::from(home.expanded), dt, motion, MotionPart::Backdrop);
        self.dirty |= home.widgets_opacity.animate(
            f32::from(home.top_open),
            dt,
            motion,
            MotionPart::Backdrop,
        );
        let selected = home.title_index(home.selected);
        if home.backdrop_selected != Some(selected) {
            home.backdrop_previous = home.backdrop_selected.replace(selected);
            home.backdrop = Tween::default();
        }
        self.dirty |= home.backdrop.animate(1., dt, motion, MotionPart::Backdrop);
        if !home.backdrop.moving() {
            home.backdrop_previous = None;
        }
        // Pointer input stops the row at its displayed position, keeping click targets stable.
        if !self.pointer || eligible || !motion.profile.geometric() {
            self.dirty |=
                home.carousel
                    .animate(home.first as f32, dt, motion, MotionPart::Carousel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p02_dock_exact_tie_uses_left_and_preserves_subpixel_differences() {
        assert_eq!(
            closest_dock_slot(960., [(6, 1000.), (1, 920.)].into_iter()),
            Some(1)
        );
        let closer = f32::from_bits(1000_f32.to_bits() - 1);
        assert_eq!(
            closest_dock_slot(960., [(1, 920.), (6, closer)].into_iter()),
            Some(6)
        );
        // f32 subtraction can round the two distances to the same value here.
        assert_eq!(
            closest_dock_slot(
                960.,
                [(0, f32::from_bits(80_f32.to_bits() - 1)), (1, 1840.)].into_iter()
            ),
            Some(1)
        );
    }

    #[test]
    fn p02_dock_navigation_uses_both_rows_at_the_current_scale() {
        for size in [DockSize::Compact, DockSize::Standard, DockSize::Large] {
            let prefs = Preferences {
                dock_size: size,
                ..Preferences::default()
            };
            let columns = prefs.columns();
            for count in [1, 2, 3, columns] {
                for scale in [0.4, 0.721, 1.] {
                    let flow =
                        DockFlow::new(&prefs, columns + count, false, scale, 88., 830., true);
                    for (direction, from, to) in [
                        (Direction::Down, 0..columns, columns..columns + count),
                        (Direction::Up, columns..columns + count, 0..columns),
                    ] {
                        for slot in from {
                            let chosen = flow
                                .destination(slot, direction, DockHorizontal::Sequential)
                                .unwrap();
                            assert!(to.contains(&chosen));
                            let x = |slot| f64::from(flow.card_rect(slot, 0.).center().0);
                            let distance = (x(chosen) - x(slot)).abs();
                            for candidate in to.clone() {
                                let other = (x(candidate) - x(slot)).abs();
                                assert!(
                                    distance < other
                                        || (distance == other && x(chosen) <= x(candidate))
                                );
                            }
                        }
                    }
                    assert_eq!(
                        flow.destination(0, Direction::Up, DockHorizontal::Sequential),
                        None
                    );
                    assert_eq!(
                        flow.destination(columns, Direction::Down, DockHorizontal::Sequential),
                        None
                    );
                }
            }
        }
    }

    #[test]
    fn p02_dock_horizontal_edges_use_only_the_selected_policy() {
        let prefs = Preferences::default();
        let columns = prefs.columns();
        let flow = DockFlow::new(&prefs, columns + 2, false, 1., 88., 300., true);
        for (policy, expected) in [
            (
                DockHorizontal::Sequential,
                [0, columns, columns - 1, columns + 1],
            ),
            (DockHorizontal::Stop, [0, columns - 1, columns, columns + 1]),
            (
                DockHorizontal::WrapRow,
                [columns - 1, 0, columns + 1, columns],
            ),
        ] {
            for ((slot, direction), target) in [
                (0, Direction::Left),
                (columns - 1, Direction::Right),
                (columns, Direction::Left),
                (columns + 1, Direction::Right),
            ]
            .into_iter()
            .zip(expected)
            {
                assert_eq!(flow.destination(slot, direction, policy), Some(target));
            }
        }
    }

    #[test]
    fn carousel_moves_one_item_and_only_reveals_at_edges() {
        let mut home = Home::preview();
        home.step(-1);
        assert_eq!(
            (home.selected, home.first, home.title_index(home.selected)),
            (-1, -1, 7)
        );
        for _ in 0..4 {
            home.step(1);
        }
        assert_eq!(home.first, -1);
        home.step(1);
        assert_eq!(home.first, 0);
    }

    #[test]
    fn dock_capacity_and_order_survive_contextual_scaling() {
        for size in [DockSize::Compact, DockSize::Standard, DockSize::Large] {
            let prefs = Preferences {
                dock_size: size,
                ..Preferences::default()
            };
            for scale in [0.4, 0.58, 0.85, 1.] {
                let flow = DockFlow::new(&prefs, 30, false, scale, 88., 440., scale == 1.);
                assert_eq!(flow.columns, prefs.columns());
                assert_eq!(flow.rows, 30_usize.div_ceil(prefs.columns()));
                assert!((flow.w / prefs.card().0 - scale).abs() < 0.001);
                for selected in [0, 7, 29] {
                    let offset = flow.reveal(selected, 0.);
                    let layout = flow.layout(offset, 2, 0);
                    assert!(
                        selected >= layout.dock_start
                            && selected < layout.dock_start + layout.dock.len()
                    );
                    assert!(layout.inputs.is_empty());
                    for rect in layout.dock {
                        assert!((rect.w / rect.h - 1.6).abs() < 0.001);
                        assert!(rect.x >= 60. && rect.x + rect.w <= 1860.);
                    }
                }
            }
        }
    }

    #[test]
    fn scroll_hints_follow_the_geometry_of_each_window_and_eligible_input_row() {
        for dock_size in [DockSize::Compact, DockSize::Standard, DockSize::Large] {
            let prefs = Preferences {
                dock_size,
                ..Preferences::default()
            };
            let columns = prefs.columns();
            for scale in [0.4, 0.58, 0.85, 1.] {
                for count in [0, 1, 2 * columns] {
                    let flow =
                        DockFlow::new(&prefs, count, count == 0, scale, 88., 300., scale == 1.);
                    assert_eq!(flow.layout(0., 0, 0).scroll, ScrollHints::default());
                    assert!(flow.layout(0., 8, 0).scroll.down);
                    let end = flow.layout(flow.input_scroll_limit(), 8, 0);
                    assert_eq!(end.scroll, ScrollHints::default());
                    assert!(end.dock_bounds.y + end.dock_bounds.h < end.inputs[0].y);
                }
                let count = columns * 20 + 1;
                let flow = DockFlow::new(&prefs, count, false, scale, 88., 300., scale == 1.);
                assert_eq!(
                    flow.layout(0., 0, 0).scroll,
                    ScrollHints {
                        up: false,
                        down: true
                    }
                );
                let limit = flow.input_scroll_limit();
                assert_eq!(
                    flow.layout(limit * 0.5, 2, 0).scroll,
                    ScrollHints {
                        up: true,
                        down: true
                    }
                );
                let last = flow.reveal(count - 1, 0.);
                assert_eq!(
                    flow.layout(last, 2, 0).scroll,
                    ScrollHints {
                        up: true,
                        down: true
                    }
                );
                assert_eq!(
                    flow.layout(last, 0, 0).scroll,
                    ScrollHints {
                        up: true,
                        down: false
                    }
                );
                assert_eq!(
                    flow.layout(limit, 2, 0).scroll,
                    ScrollHints {
                        up: true,
                        down: false
                    }
                );
            }
        }
    }

    #[test]
    fn dock_uses_all_rows_that_fit_between_the_fixed_area_and_the_bottom() {
        let prefs = Preferences {
            dock_size: DockSize::Compact,
            ..Preferences::default()
        };
        let columns = prefs.columns();
        for rows in 1..=5 {
            let count = (rows - 1) * columns + 1;
            let flow = DockFlow::new(&prefs, count, false, 1., 88., 92., true);
            let layout = flow.layout(0., 0, 0);
            assert_eq!((layout.viewport.y, layout.viewport.h), (92., 940.));
            assert_eq!(
                (layout.rows, layout.visible_rows, layout.dock.len()),
                (rows, rows, count)
            );
        }
        let crowded = DockFlow::new(&prefs, 100, false, 1., 120., 652., true);
        assert_eq!(crowded.layout(0., 0, 0).visible_rows, 2);
    }

    #[test]
    fn dock_reveal_preserves_visible_rows_and_moves_only_as_far_as_the_clipped_edge() {
        let prefs = Preferences::default();
        let flow = DockFlow::new(&prefs, 100, false, 1., 120., 470., true);
        let row = 5;
        let slot = row * flow.columns;
        let limit = flow.dock_scroll_limit();
        let (top, bottom) = flow.focus_range(row);
        let at_bottom = bottom - SCROLL_BOTTOM;
        let at_top = top - flow.viewport.y;
        for offset in [at_bottom, at_bottom + 20., at_top] {
            assert_eq!(flow.reveal(slot, offset), offset);
        }
        for offset in [0., at_bottom - 11.] {
            assert_eq!(flow.reveal(slot, offset), at_bottom);
        }
        for offset in [at_top + 11., limit] {
            assert_eq!(flow.reveal(slot, offset), at_top);
        }
    }

    #[test]
    fn interrupted_expansion_reverses_and_reduced_motion_snaps() {
        let mut tween = Tween::default();
        tween.advance(1., 0.16, 0.32, false);
        let shown = tween.value;
        tween.advance(0., 0., 0.30, false);
        assert_eq!(tween.value, shown);
        tween.advance(0., 0.05, 0.30, false);
        assert!(tween.value < shown);
        tween.advance(1., 0., 0.32, true);
        assert_eq!(tween.value, 1.);
    }
}
