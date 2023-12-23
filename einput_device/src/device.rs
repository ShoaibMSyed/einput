use core::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

use crate::{
    builder::DeviceBuilder,
    raw_device::{DevicePtr, RawDevice},
};

pub struct Device(pub(crate) RawDevice);

impl Device {
    pub fn builder(name: String, id: String) -> DeviceBuilder {
        DeviceBuilder::new(name, id)
    }

    pub fn update(&mut self, ptr: &DevicePtr) -> Result<(), ()> {
        DevicePtr::update(self, ptr)
    }
}

impl Clone for Device {
    fn clone(&self) -> Self {
        Device(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}

impl Borrow<DevicePtr> for Device {
    fn borrow(&self) -> &DevicePtr {
        self.deref()
    }
}

impl BorrowMut<DevicePtr> for Device {
    fn borrow_mut(&mut self) -> &mut DevicePtr {
        self.deref_mut()
    }
}

impl Deref for Device {
    type Target = DevicePtr;

    fn deref(&self) -> &Self::Target {
        DevicePtr::new(&self.0)
    }
}

impl DerefMut for Device {
    fn deref_mut(&mut self) -> &mut Self::Target {
        DevicePtr::new_mut(&mut self.0)
    }
}
