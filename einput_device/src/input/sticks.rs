use einput_util::axis::Stick;
use serde::{Deserialize, Serialize};

use crate::util::{DeviceIndex, DeviceIndexMut};

use super::DeviceInput;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sticks {
    pub left: Stick,
    pub right: Stick,
}

impl Sticks {
    pub fn get(&self, id: StickId) -> &Stick {
        match id {
            StickId::Left => &self.left,
            StickId::Right => &self.right,
        }
    }

    pub fn get_mut(&mut self, id: StickId) -> &mut Stick {
        match id {
            StickId::Left => &mut self.left,
            StickId::Right => &mut self.right,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StickId {
    Left,
    Right,
}

impl StickId {
    pub const ALL: [Self; 2] = [Self::Left, Self::Right];

    pub fn name(self) -> &'static str {
        match self {
            StickId::Left => "Left",
            StickId::Right => "Right",
        }
    }
}

impl DeviceIndex<DeviceInput> for StickId {
    type Output<'a> = Option<&'a Stick>;

    fn index<'a>(self, device: &'a DeviceInput) -> Self::Output<'a> {
        Some(device.sticks()?.get(self))
    }
}

impl DeviceIndexMut<DeviceInput> for StickId {
    type Output<'a> = Option<&'a mut Stick>;

    fn index_mut<'a>(self, device: &'a mut DeviceInput) -> Self::Output<'a> {
        Some(device.sticks_mut()?.get_mut(self))
    }
}