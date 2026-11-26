use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionProfile {
    #[default]
    Balanced,
    Agile,
    Cinematic,
    Reduced,
    None,
}

impl MotionProfile {
    pub const ALL: [Self; 5] = [
        Self::Balanced,
        Self::Agile,
        Self::Cinematic,
        Self::Reduced,
        Self::None,
    ];

    pub fn from_legacy(self, reduced: bool) -> Self {
        if reduced && self == Self::Balanced {
            Self::Reduced
        } else {
            self
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Balanced => "evolutionMotionBalanced",
            Self::Agile => "evolutionMotionAgile",
            Self::Cinematic => "evolutionMotionCinematic",
            Self::Reduced => "evolutionMotionReduced",
            Self::None => "evolutionMotionNone",
        }
    }

    pub fn geometric(self) -> bool {
        matches!(self, Self::Balanced | Self::Agile | Self::Cinematic)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionCurve {
    #[default]
    Standard,
    Elastic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct MotionSpeed(u8);

impl MotionSpeed {
    pub fn stepped(self, increase: bool) -> Self {
        Self(if increase {
            (self.0 + 10).min(150)
        } else {
            (self.0 - 10).max(50)
        })
    }

    pub fn percent(self) -> u8 {
        self.0
    }
}

impl Default for MotionSpeed {
    fn default() -> Self {
        Self(100)
    }
}

impl<'de> Deserialize<'de> for MotionSpeed {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        Ok(value
            .as_u64()
            .filter(|n| (50..=150).contains(n) && n.is_multiple_of(10))
            .map(|n| Self(n as u8))
            .unwrap_or_default())
    }
}

#[derive(Clone, Copy)]
pub enum MotionPart {
    Focus,
    Carousel,
    Expand,
    Collapse,
    Backdrop,
}

#[derive(Clone, Copy)]
pub struct Motion {
    pub profile: MotionProfile,
    pub curve: MotionCurve,
    pub speed: MotionSpeed,
}

impl Motion {
    pub const SAMPLE: Duration = Duration::from_millis(1500);

    pub fn saved(prefs: &crate::model::Preferences) -> Self {
        let home = &prefs.evolution_home;
        Self {
            profile: home.motion_profile.from_legacy(prefs.reduced_motion),
            curve: home.motion_curve,
            speed: home.motion_speed,
        }
    }

    pub fn duration(self, part: MotionPart) -> f32 {
        match self.profile {
            MotionProfile::None => 0.,
            MotionProfile::Reduced => match part {
                MotionPart::Backdrop => 0.120,
                _ => 0.,
            },
            profile => {
                let base = match part {
                    MotionPart::Focus => 180.,
                    MotionPart::Carousel => 200.,
                    MotionPart::Expand => 320.,
                    MotionPart::Collapse => 300.,
                    MotionPart::Backdrop => 350.,
                };
                // Desktop timing references for the two alternate geometric profiles.
                let factor = match profile {
                    MotionProfile::Agile => 0.75,
                    MotionProfile::Cinematic => 1.25,
                    _ => 1.,
                };
                base * factor / (10. * f32::from(self.speed.0))
            }
        }
    }

    pub fn progress(self, part: MotionPart, elapsed: f32) -> f32 {
        let duration = self.duration(part);
        let t = if duration == 0. {
            1.
        } else {
            (elapsed / duration).clamp(0., 1.)
        };
        if self.profile.geometric()
            && self.curve == MotionCurve::Elastic
            && !matches!(part, MotionPart::Backdrop)
        {
            return if t < 0.75 {
                1.03 * (1. - (1. - t / 0.75).powi(3))
            } else {
                1. + 0.03 * (1. - (t - 0.75) / 0.25).powi(3)
            };
        }
        match part {
            MotionPart::Focus | MotionPart::Carousel => 1. - (1. - t).powi(3),
            MotionPart::Backdrop => t * t * (3. - 2. * t),
            _ if t < 0.5 => 4. * t.powi(3),
            _ => 1. - (-2. * t + 2.).powi(3) / 2.,
        }
    }
}
