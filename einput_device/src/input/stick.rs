use einput_util::axis::Stick;

use crate::{
    util::{DeviceIndex, DeviceIndexMut, InputId},
    DeviceInput,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StickId {
    Left,
    Right,
}

impl StickId {
    pub const ALL: [Self; 2] = [Self::Left, Self::Right];
}

impl InputId for StickId {
    const LEN: u8 = 2;

    fn all() -> [Self; Self::LEN as usize] {
        Self::ALL
    }

    fn id(self) -> u8 {
        self as u8
    }
}

impl DeviceIndex<DeviceInput> for StickId {
    type Output<'a> = Option<&'a Stick>;

    fn index<'a>(self, device: &'a DeviceInput) -> Self::Output<'a> {
        device.stick(self)
    }
}

impl DeviceIndexMut<DeviceInput> for StickId {
    type Output<'a> = Option<&'a mut Stick>;

    fn index_mut<'a>(self, device: &'a mut DeviceInput) -> Self::Output<'a> {
        device.stick_mut(self)
    }
}
