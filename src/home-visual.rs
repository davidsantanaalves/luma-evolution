use crate::home_evolution::{CinemaScale, DockHorizontal, DockScale, DockSize, GlobalPosition};
use crate::home_motion::{Motion, MotionCurve, MotionProfile, MotionSpeed};
use crate::i18n::Language;
use crate::model::Preferences;
use crate::state::Target;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub fn read_option<'de, D: serde::Deserializer<'de>, T: DeserializeOwned + Default>(
    d: D,
) -> Result<T, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    Ok(serde_json::from_value(value).unwrap_or_default())
}

macro_rules! visual_options {
    ($name:ident { $first:ident => $first_key:literal $(, $option:ident => $key:literal)+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            #[default]
            $first,
            $($option),+
        }

        impl $name {
            const ALL: &'static [Self] = &[Self::$first, $(Self::$option),+];

            pub fn key(self) -> &'static str {
                match self {
                    Self::$first => $first_key,
                    $(Self::$option => $key),+
                }
            }
        }
    };
}

visual_options!(DockTheme {
    GlassPremium => "evolutionGlassPremium",
    GlassDark => "evolutionGlassDark",
    Minimal => "evolutionMinimal",
});
visual_options!(DockPosition {
    Standard => "evolutionStandard",
    Raised => "evolutionRaised",
});
visual_options!(DockLabels {
    Focused => "evolutionFocusedOnly",
    Always => "evolutionAlways",
    Hidden => "stateOff",
});
visual_options!(WidgetsLayout {
    Split => "evolutionWidgetsSplit",
    Compact => "evolutionWidgetsCompact",
    Right => "evolutionWidgetsRight",
});
visual_options!(WeatherDetails {
    Balanced => "evolutionMotionBalanced",
    Minimal => "evolutionMinimal",
    Detailed => "evolutionDetailed",
});
visual_options!(CinemaLayout {
    Hybrid => "evolutionCinemaHybrid",
    Strip => "evolutionCinemaStrip",
    Spotlight => "evolutionCinemaSpotlight",
});
visual_options!(CinemaDetails {
    Balanced => "evolutionMotionBalanced",
    Minimal => "evolutionMinimal",
});

fn step<T: Copy + PartialEq>(options: &[T], saved: T, increase: bool) -> T {
    let index = options
        .iter()
        .position(|value| *value == saved)
        .unwrap_or(0);
    let next = if increase {
        (index + 1).min(options.len() - 1)
    } else {
        index.saturating_sub(1)
    };
    options[next]
}

pub const ROTATIONS: [Option<u64>; 6] = [None, Some(5), Some(10), Some(15), Some(20), Some(30)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Category {
    Dock,
    Cinema,
    Layout,
    Shortcuts,
    Animations,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Prompt {
    Restore,
    Unsaved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Field {
    DockScale,
    DockHorizontal,
    DockSize,
    DockTheme,
    DockPosition,
    DockLabels,
    CinemaScale,
    CinemaRotation,
    CinemaLayout,
    CinemaDetails,
    WidgetsVisible,
    CinemaVisible,
    WidgetsLayout,
    WeatherDetails,
    GlobalPosition,
    MotionProfile,
    MotionCurve,
    MotionSpeed,
}

impl Field {
    pub fn target(self) -> Target {
        match self {
            Self::DockScale => Target::DockScale,
            Self::DockHorizontal => Target::DockHorizontal,
            Self::CinemaScale => Target::CinemaScale,
            _ => Target::VisualField(self),
        }
    }

    pub fn step_target(self, increase: bool) -> Target {
        match self {
            Self::DockScale => Target::DockScaleStep(increase),
            Self::DockHorizontal => Target::DockHorizontalStep(increase),
            Self::CinemaScale => Target::CinemaScaleStep(increase),
            _ => Target::VisualStep(self, increase),
        }
    }

    pub fn from_target(target: &Target) -> Option<Self> {
        match *target {
            Target::DockScale | Target::DockScaleStep(_) => Some(Self::DockScale),
            Target::DockHorizontal | Target::DockHorizontalStep(_) => Some(Self::DockHorizontal),
            Target::CinemaScale | Target::CinemaScaleStep(_) => Some(Self::CinemaScale),
            Target::VisualField(field) | Target::VisualStep(field, _) => Some(field),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::DockScale => "evolutionDockScale",
            Self::DockHorizontal => "evolutionDockHorizontal",
            Self::DockSize => "evolutionDockCardSize",
            Self::DockTheme => "evolutionTheme",
            Self::DockPosition => "evolutionDockPosition",
            Self::DockLabels => "appNames",
            Self::CinemaScale => "evolutionCinemaScale",
            Self::CinemaRotation => "evolutionCinemaRotation",
            Self::CinemaLayout => "evolutionComposition",
            Self::CinemaDetails => "evolutionDetails",
            Self::WidgetsVisible => "evolutionWidgetsVisible",
            Self::CinemaVisible => "evolutionCinemaVisible",
            Self::WidgetsLayout => "evolutionWidgetsLayout",
            Self::WeatherDetails => "evolutionWeatherDetails",
            Self::GlobalPosition => "evolutionGlobalActions",
            Self::MotionProfile => "evolutionMotionProfile",
            Self::MotionCurve => "evolutionMotionCurve",
            Self::MotionSpeed => "evolutionMotionSpeed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Action {
    Apply,
    Restore,
    ConfirmRestore,
    Cancel,
    Discard,
    Continue,
}

impl Action {
    pub fn key(self, category: Category) -> &'static str {
        match self {
            Self::Apply => "evolutionApply",
            Self::Restore if category == Category::All => "evolutionRestoreAll",
            Self::Restore => "evolutionRestoreCategory",
            Self::ConfirmRestore => "evolutionRestoreDefaults",
            Self::Cancel => "cancel",
            Self::Discard => "evolutionDiscard",
            Self::Continue => "evolutionContinueEditing",
        }
    }
}

impl Category {
    pub fn fields(self) -> &'static [Field] {
        use Field::*;
        match self {
            Self::Dock => &[
                DockScale,
                DockHorizontal,
                DockSize,
                DockTheme,
                DockPosition,
                DockLabels,
            ],
            Self::Cinema => &[CinemaScale, CinemaRotation, CinemaLayout, CinemaDetails],
            Self::Layout => &[WidgetsVisible, CinemaVisible, WidgetsLayout, WeatherDetails],
            Self::Shortcuts => &[GlobalPosition],
            Self::Animations => &[MotionProfile, MotionCurve, MotionSpeed],
            Self::All => &[],
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Dock => "evolutionDock",
            Self::Cinema => "evolutionCinema",
            Self::Layout => "evolutionLayout",
            Self::Shortcuts => "evolutionGlobalActions",
            Self::Animations => "evolutionAnimations",
            Self::All => "evolutionRestoreAll",
        }
    }

    pub fn row(self) -> usize {
        use crate::state::*;
        match self {
            Self::Dock => DOCK_ROW,
            Self::Cinema => CINEMA_ROW,
            Self::Layout => LAYOUT_ROW,
            Self::Shortcuts => NAVIGATION_ROW,
            Self::Animations => ANIMATIONS_ROW,
            Self::All => RESTORE_VISUAL_ROW,
        }
    }

    pub fn action(self, action: Action) -> Target {
        use Action::*;
        match (self, action) {
            (Self::Dock, Apply) => Target::DockApply,
            (Self::Dock, Restore) => Target::DockRestore,
            (Self::Dock, ConfirmRestore) => Target::DockConfirmRestore,
            (Self::Dock, Cancel) => Target::DockCancel,
            (Self::Cinema, Apply) => Target::CinemaApply,
            (Self::Cinema, Restore) => Target::CinemaRestore,
            (Self::Cinema, ConfirmRestore) => Target::CinemaConfirmRestore,
            (Self::Cinema, Cancel) => Target::CinemaCancel,
            (Self::Cinema, Discard) => Target::CinemaDiscard,
            (Self::Cinema, Continue) => Target::CinemaContinue,
            _ => Target::VisualAction(action),
        }
    }

    pub fn restore_prompt(self) -> &'static str {
        match self {
            Self::Dock => "evolutionRestoreDock",
            Self::Cinema => "evolutionRestoreCinema",
            Self::Layout => "evolutionRestoreLayout",
            Self::Shortcuts => "evolutionRestoreShortcuts",
            Self::Animations => "evolutionRestoreAnimations",
            Self::All => "evolutionRestoreAllPrompt",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Edit {
    pub category: Category,
    pub draft: VisualPreferences,
    pub prompt: Option<Prompt>,
}

impl Edit {
    pub fn rows(self) -> Vec<(Target, &'static str)> {
        use Action::*;
        let actions: &[Action] = match (self.prompt, self.category) {
            (Some(Prompt::Restore), _) => &[ConfirmRestore, Cancel],
            (Some(Prompt::Unsaved), _) => &[Apply, Discard, Continue],
            (None, Category::All) => &[Apply, Cancel],
            _ => &[Apply, Restore, Cancel],
        };
        let fields = self
            .category
            .fields()
            .iter()
            .filter(|_| self.prompt.is_none());
        fields
            .map(|field| (field.target(), field.key()))
            .chain(
                actions
                    .iter()
                    .map(|&action| (self.category.action(action), action.key(self.category))),
            )
            .collect()
    }

    pub fn first(self) -> Target {
        self.category.fields().first().map_or_else(
            || self.category.action(Action::Apply),
            |field| field.target(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct VisualPreferences {
    dock_scale: DockScale,
    dock_horizontal: DockHorizontal,
    dock_size: DockSize,
    dock_theme: DockTheme,
    dock_position: DockPosition,
    dock_labels: DockLabels,
    legacy_home_names: bool,
    cinema_scale: CinemaScale,
    cinema_layout: CinemaLayout,
    cinema_details: CinemaDetails,
    rotation_enabled: bool,
    rotation_seconds: u64,
    widgets_visible: bool,
    cinema_visible: bool,
    widgets_layout: WidgetsLayout,
    weather_details: WeatherDetails,
    global_position: GlobalPosition,
    pub motion_profile: MotionProfile,
    motion_curve: MotionCurve,
    motion_speed: MotionSpeed,
    legacy_reduced_motion: bool,
}

impl VisualPreferences {
    pub fn saved(prefs: &Preferences) -> Self {
        let p = &prefs.evolution_home;
        Self {
            dock_scale: p.cinema_dock_scale,
            dock_horizontal: p.dock_horizontal,
            dock_size: p.dock_size,
            dock_theme: p.dock_theme,
            dock_position: p.dock_position,
            dock_labels: p.dock_labels,
            legacy_home_names: prefs.show_home_names,
            cinema_scale: p.compact_cinema_scale,
            cinema_layout: p.cinema_layout,
            cinema_details: p.cinema_details,
            rotation_enabled: p.rotation_enabled,
            rotation_seconds: p.rotation_seconds,
            widgets_visible: p.show_top,
            cinema_visible: p.show_center,
            widgets_layout: p.widgets_layout,
            weather_details: p.weather_details,
            global_position: p.global_position,
            motion_profile: p.motion_profile,
            motion_curve: p.motion_curve,
            motion_speed: p.motion_speed,
            legacy_reduced_motion: prefs.reduced_motion,
        }
    }

    pub fn apply(self, prefs: &mut Preferences) {
        let p = &mut prefs.evolution_home;
        p.cinema_dock_scale = self.dock_scale;
        p.dock_horizontal = self.dock_horizontal;
        p.dock_size = self.dock_size;
        p.dock_theme = self.dock_theme;
        p.dock_position = self.dock_position;
        p.dock_labels = self.dock_labels;
        prefs.show_home_names = self.legacy_home_names;
        p.compact_cinema_scale = self.cinema_scale;
        p.cinema_layout = self.cinema_layout;
        p.cinema_details = self.cinema_details;
        p.rotation_enabled = self.rotation_enabled;
        p.rotation_seconds = self.rotation_seconds;
        p.show_top = self.widgets_visible;
        p.show_center = self.cinema_visible;
        p.widgets_layout = self.widgets_layout;
        p.weather_details = self.weather_details;
        p.global_position = self.global_position;
        p.motion_profile = self.motion_profile;
        p.motion_curve = self.motion_curve;
        p.motion_speed = self.motion_speed;
        prefs.reduced_motion = self.legacy_reduced_motion;
    }

    pub fn restore(&mut self, category: Category) {
        let d = Self::saved(&Preferences::default());
        if category == Category::All {
            *self = d;
            return;
        }
        match category {
            Category::Dock => {
                self.dock_scale = d.dock_scale;
                self.dock_horizontal = d.dock_horizontal;
                self.dock_size = d.dock_size;
                self.dock_theme = d.dock_theme;
                self.dock_position = d.dock_position;
                self.dock_labels = d.dock_labels;
                self.legacy_home_names = d.legacy_home_names;
            }
            Category::Cinema => {
                self.cinema_scale = d.cinema_scale;
                self.cinema_layout = d.cinema_layout;
                self.cinema_details = d.cinema_details;
                self.rotation_enabled = d.rotation_enabled;
                self.rotation_seconds = d.rotation_seconds;
            }
            Category::Layout => {
                self.widgets_visible = d.widgets_visible;
                self.cinema_visible = d.cinema_visible;
                self.widgets_layout = d.widgets_layout;
                self.weather_details = d.weather_details;
            }
            Category::Shortcuts => self.global_position = d.global_position,
            Category::Animations => {
                self.motion_profile = d.motion_profile;
                self.motion_curve = d.motion_curve;
                self.motion_speed = d.motion_speed;
                self.legacy_reduced_motion = d.legacy_reduced_motion;
            }
            Category::All => unreachable!(),
        }
    }

    pub fn motion(self) -> Motion {
        Motion {
            profile: self.motion_profile.from_legacy(self.legacy_reduced_motion),
            curve: self.motion_curve,
            speed: self.motion_speed,
        }
    }

    fn rotation(self) -> Option<u64> {
        self.rotation_enabled.then_some(self.rotation_seconds)
    }

    pub fn set_global_position(&mut self, position: GlobalPosition) {
        self.global_position = position;
    }

    pub fn adjust(&mut self, field: Field, increase: bool) {
        use Field::*;
        match field {
            DockScale => self.dock_scale = self.dock_scale.stepped(increase),
            DockHorizontal => self.dock_horizontal = self.dock_horizontal.stepped(increase),
            DockSize => {
                self.dock_size = step(
                    &[
                        crate::home_evolution::DockSize::Compact,
                        crate::home_evolution::DockSize::Standard,
                        crate::home_evolution::DockSize::Large,
                    ],
                    self.dock_size,
                    increase,
                )
            }
            DockTheme => {
                self.dock_theme = step(
                    crate::home_visual::DockTheme::ALL,
                    self.dock_theme,
                    increase,
                )
            }
            DockPosition => {
                self.dock_position = step(
                    crate::home_visual::DockPosition::ALL,
                    self.dock_position,
                    increase,
                )
            }
            DockLabels => {
                let current = if self.dock_labels == crate::home_visual::DockLabels::Focused
                    && self.legacy_home_names
                {
                    crate::home_visual::DockLabels::Always
                } else {
                    self.dock_labels
                };
                self.dock_labels = step(crate::home_visual::DockLabels::ALL, current, increase);
                self.legacy_home_names = false;
            }
            CinemaScale => self.cinema_scale = self.cinema_scale.stepped(increase),
            CinemaLayout => {
                self.cinema_layout = step(
                    crate::home_visual::CinemaLayout::ALL,
                    self.cinema_layout,
                    increase,
                )
            }
            CinemaDetails => {
                self.cinema_details = step(
                    crate::home_visual::CinemaDetails::ALL,
                    self.cinema_details,
                    increase,
                )
            }
            CinemaRotation => {
                let current = self.rotation().map(|seconds| {
                    if ROTATIONS.contains(&Some(seconds)) {
                        seconds
                    } else {
                        10
                    }
                });
                let next = step(&ROTATIONS, current, increase);
                self.rotation_enabled = next.is_some();
                self.rotation_seconds = next.unwrap_or(10);
            }
            WidgetsVisible => self.widgets_visible = increase,
            CinemaVisible => self.cinema_visible = increase,
            WidgetsLayout => {
                self.widgets_layout = step(
                    crate::home_visual::WidgetsLayout::ALL,
                    self.widgets_layout,
                    increase,
                )
            }
            WeatherDetails => {
                self.weather_details = step(
                    crate::home_visual::WeatherDetails::ALL,
                    self.weather_details,
                    increase,
                )
            }
            GlobalPosition => {
                self.global_position = step(
                    &crate::home_evolution::GlobalPosition::ALL,
                    self.global_position,
                    increase,
                )
            }
            MotionProfile => {
                self.motion_profile = step(
                    &crate::home_motion::MotionProfile::ALL,
                    self.motion().profile,
                    increase,
                );
                self.legacy_reduced_motion = false;
            }
            MotionCurve => {
                self.motion_curve = if increase {
                    crate::home_motion::MotionCurve::Elastic
                } else {
                    crate::home_motion::MotionCurve::Standard
                }
            }
            MotionSpeed => self.motion_speed = self.motion_speed.stepped(increase),
        }
    }

    pub fn label(self, field: Field, language: Language) -> String {
        use Field::*;
        let key = match field {
            DockScale => return self.dock_scale.label(),
            DockHorizontal => self.dock_horizontal.label(),
            DockSize => match self.dock_size {
                crate::home_evolution::DockSize::Compact => "evolutionCompact",
                crate::home_evolution::DockSize::Standard => "evolutionStandard",
                crate::home_evolution::DockSize::Large => "evolutionLarge",
            },
            DockTheme => self.dock_theme.key(),
            DockPosition => self.dock_position.key(),
            DockLabels
                if self.dock_labels == crate::home_visual::DockLabels::Focused
                    && self.legacy_home_names =>
            {
                "evolutionAlways"
            }
            DockLabels => self.dock_labels.key(),
            CinemaScale => return self.cinema_scale.label(),
            CinemaLayout => self.cinema_layout.key(),
            CinemaDetails => self.cinema_details.key(),
            CinemaRotation => {
                return self.rotation().map_or_else(
                    || language.t("stateOff").to_owned(),
                    |seconds| format!("{seconds} s"),
                );
            }
            WidgetsVisible => {
                if self.widgets_visible {
                    "stateOn"
                } else {
                    "stateOff"
                }
            }
            CinemaVisible => {
                if self.cinema_visible {
                    "stateOn"
                } else {
                    "stateOff"
                }
            }
            WidgetsLayout => self.widgets_layout.key(),
            WeatherDetails => self.weather_details.key(),
            GlobalPosition => return self.global_position.label(language).to_owned(),
            MotionProfile => self.motion().profile.key(),
            MotionCurve => match self.motion_curve {
                crate::home_motion::MotionCurve::Standard => "evolutionStandard",
                crate::home_motion::MotionCurve::Elastic => "evolutionElastic",
            },
            MotionSpeed => return format!("{}%", self.motion_speed.percent()),
        };
        language.t(key).to_owned()
    }
}
