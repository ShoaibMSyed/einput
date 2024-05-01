use einput_util::axis::Trigger;
use serde::{Deserialize, Serialize};

use crate::{
    util::{DeviceIndex, DeviceIndexMut},
    DeviceInput,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Triggers {
    pub l1: Trigger,
    pub r1: Trigger,
    pub l2: Trigger,
    pub r2: Trigger,
}

impl Triggers {
    pub fn get(&self, id: TriggerId) -> &Trigger {
        match id {
            TriggerId::L1 => &self.l1,
            TriggerId::R1 => &self.r1,
            TriggerId::L2 => &self.l2,
            TriggerId::R2 => &self.r2,
        }
    }

    pub fn get_mut(&mut self, id: TriggerId) -> &mut Trigger {
        match id {
            TriggerId::L1 => &mut self.l1,
            TriggerId::R1 => &mut self.r1,
            TriggerId::L2 => &mut self.l2,
            TriggerId::R2 => &mut self.r2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TriggerId {
    L1,
    R1,
    L2,
    R2,
}

impl TriggerId {
    pub const ALL: [Self; 4] = [Self::L1, Self::R1, Self::L2, Self::R2];

    pub fn name(self) -> &'static str {
        match self {
            TriggerId::L1 => "L1",
            TriggerId::R1 => "R1",
            TriggerId::L2 => "L2",
            TriggerId::R2 => "R2",
        }
    }
}

impl DeviceIndex<DeviceInput> for TriggerId {
    type Output<'a> = Option<&'a Trigger>;

    fn index<'a>(self, device: &'a DeviceInput) -> Self::Output<'a> {
        Some(device.triggers()?.get(self))
    }
}

impl DeviceIndexMut<DeviceInput> for TriggerId {
    type Output<'a> = Option<&'a mut Trigger>;

    fn index_mut<'a>(self, device: &'a mut DeviceInput) -> Self::Output<'a> {
        Some(device.triggers_mut()?.get_mut(self))
    }
}
