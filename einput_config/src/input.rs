use std::f32::consts::PI;

use einput_device::input::{
    DeviceInput,
    buttons::{Button, Buttons},
    stick::StickId,
    triggers::TriggerId,
};
use einput_util::axis::{Stick, Trigger};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DeviceInputConfig {
    pub buttons: [Button; Button::ALL.len()],
    pub sticks: [StickConfig; StickId::ALL.len()],
    pub triggers: [TriggerConfig; TriggerId::ALL.len()],
}

impl Default for DeviceInputConfig {
    fn default() -> Self {
        Self {
            buttons: Button::ALL,
            sticks: Default::default(),
            triggers: TriggerId::ALL.map(|id| TriggerConfig {
                remap_to: id,
                min: 0,
                max: 255,
            }),
        }
    }
}

impl DeviceInputConfig {
    pub(super) fn apply(&self, device: &mut DeviceInput) {
        if let Some(buttons) = device.buttons_mut() {
            let mut new_buttons = Buttons::default();

            for button in buttons.get_pressed() {
                let button = self.buttons[button as usize];
                new_buttons.set(button, true);
            }

            *buttons = new_buttons;
        }

        for id in StickId::ALL {
            let Some(stick) = device.stick_mut(id) else {
                continue;
            };

            *stick = self.sticks[id as usize].apply(*stick);
        }

        if let Some(triggers) = device.triggers_mut() {
            let mut new_triggers = triggers.clone();

            for id in TriggerId::ALL {
                let new_id = self.triggers[id as usize].remap_to;

                *new_triggers.get_mut(new_id) = self.triggers[id as usize].apply(*triggers.get(id));
            }

            *triggers = new_triggers;
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub remap_to: TriggerId,
    pub min: u8,
    pub max: u8,
}

impl TriggerConfig {
    fn apply(&self, value: Trigger) -> Trigger {
        use einput_util::axis::TriggerAxis;

        let value = value.0.to_f32();
        let min = self.min.to_f32();
        let max = self.max.to_f32();
        let range = max - min;

        let value = ((value - min).max(0.0) / range).min(1.0);

        u8::from_f32(value).into()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StickConfig {
    pub deadzone: f32,
    pub samples: Option<[f32; 32]>,
}

impl Default for StickConfig {
    fn default() -> Self {
        Self {
            deadzone: 0.0,
            samples: None,
        }
    }
}

impl StickConfig {
    fn apply(&self, value: Stick) -> Stick {
        if self.deadzone == 0.0 && self.samples.is_none() {
            return value;
        }

        let max = match &self.samples {
            Some(samples) => {
                let mut angle = f32::atan2(value.y, value.x);
                if angle < 0.0 {
                    angle = 2.0 * std::f32::consts::PI + angle;
                }
                Self::sample(samples, angle)
            }
            None => 1.0,
        };

        let range = max - self.deadzone;

        if range <= 0.0 {
            return Stick::default();
        }

        let new_length = ((value.length() - self.deadzone).max(0.0) / range).min(1.0);
        value.normalized() * new_length
    }

    fn sample(samples: &[f32; 32], angle: f32) -> f32 {
        let (mut i1, mut i2) = (0, 0);
        let mut influence = 0.0;

        for i in 0..32 {
            let min_angle = Self::index_to_angle(i);
            let max_angle = Self::index_to_angle(i + 1);
            if min_angle <= angle && angle < max_angle {
                i1 = i;
                i2 = (i + 1) % 32;
                influence = (angle - min_angle) / (max_angle - min_angle);
                break;
            }
        }

        let v1 = samples[i1] * (1.0 - influence);
        let v2 = samples[i2] * influence;

        v1 + v2
    }

    fn index_to_angle(index: usize) -> f32 {
        (index as f32) * (PI * 2.0 / 32.0)
    }
}

pub struct StickSampler {
    pub samples: [f32; 32],
}

impl StickSampler {
    pub fn new() -> Self {
        Self { samples: [0.0; 32] }
    }

    pub fn add(&mut self, stick: Stick) {
        let mut angle = f32::atan2(stick.y, stick.x);
        if angle < 0.0 {
            angle = 2.0 * PI + angle;
        }

        let (mut i1, mut i2) = (0, 0);
        let mut influence = 0.0;

        for i in 0..32 {
            let min_angle = StickConfig::index_to_angle(i);
            let max_angle = StickConfig::index_to_angle(i + 1);
            if min_angle <= angle && angle < max_angle {
                i1 = i;
                i2 = (i + 1) % 32;
                influence = (angle - min_angle) / (max_angle - min_angle);
                break;
            }
        }

        if influence <= 0.5 {
            self.samples[i1] = f32::max(self.samples[i1], stick.length());
        }

        if influence >= 0.5 {
            self.samples[i2] = f32::max(self.samples[i2], stick.length());
        }
    }
}
