use crate::home_evolution::Preferences;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Timing {
    Hold,
    Hover,
    Leave,
}

impl Timing {
    const fn limits(self) -> (u64, u64, u64, u64) {
        match self {
            Self::Hold => (500, 2000, 100, 1000),
            Self::Hover => (100, 600, 50, 250),
            Self::Leave => (100, 800, 50, 350),
        }
    }

    pub const fn default_ms(self) -> u64 {
        self.limits().3
    }

    pub fn normalize(self, milliseconds: u64) -> u64 {
        let (min, max, step, _) = self.limits();
        let offset = milliseconds.clamp(min, max) - min;
        min + (offset + step / 2) / step * step
    }

    pub fn step(self, value: u64, increase: bool) -> u64 {
        let value = self.normalize(value);
        let step = self.limits().2;
        self.normalize(if increase {
            value + step
        } else {
            value.saturating_sub(step)
        })
    }
}

fn read<'de, D: Deserializer<'de>>(deserializer: D, timing: Timing) -> Result<u64, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(timing.normalize(value.as_u64().unwrap_or(timing.default_ms())))
}

pub fn read_hold<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    read(d, Timing::Hold)
}
pub fn read_hover<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    read(d, Timing::Hover)
}
pub fn read_leave<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    read(d, Timing::Leave)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Settings {
    pub hold_ms: u64,
    pub hover_ms: u64,
    pub leave_ms: u64,
    pub pointer_expands: bool,
}

impl Settings {
    pub fn saved(prefs: &Preferences) -> Self {
        Self {
            hold_ms: Timing::Hold.normalize(prefs.hold_ms),
            hover_ms: Timing::Hover.normalize(prefs.hover_ms),
            leave_ms: Timing::Leave.normalize(prefs.leave_ms),
            pointer_expands: prefs.pointer_expands,
        }
    }

    pub fn apply(self, prefs: &mut Preferences) {
        prefs.hold_ms = self.hold_ms;
        prefs.hover_ms = self.hover_ms;
        prefs.leave_ms = self.leave_ms;
        prefs.pointer_expands = self.pointer_expands;
    }

    pub fn step(&mut self, timing: Timing, increase: bool) {
        let value = match timing {
            Timing::Hold => &mut self.hold_ms,
            Timing::Hover => &mut self.hover_ms,
            Timing::Leave => &mut self.leave_ms,
        };
        *value = timing.step(*value, increase);
    }
}
