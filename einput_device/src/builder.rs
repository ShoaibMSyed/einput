use core::cmp::Reverse;

use crate::{
    info::{DeviceInfo, InternalInfo},
    raw_device::{ComponentCtor, RawDevice},
    Component, Device, RawComponentId,
};

pub struct DeviceBuilder {
    ctors: Vec<ComponentCtor>,
    info: InternalInfo,
}

impl DeviceBuilder {
    pub fn new(name: String, id: String) -> Self {
        DeviceBuilder {
            ctors: Vec::new(),
            info: InternalInfo::new(name, id),
        }
    }

    #[track_caller]
    pub fn add_components<T: Component>(mut self, info: impl IntoIterator<Item = T::Info>) -> Self {
        let amount = self.info.add_info::<T>(info);
        if amount > 255 {
            panic!(
                "added too many '{}' components",
                core::any::type_name::<T>()
            );
        }
        let amount = amount as u8;

        let data = self.find_or_create_ctor::<T>();

        data.count = match data.count.checked_add(amount) {
            Some(count) => count,
            None => {
                panic!(
                    "added too many '{}' components",
                    core::any::type_name::<T>()
                )
            }
        };

        if self.ctors.len() > 255 {
            panic!("added too many types of components");
        }

        self
    }

    pub fn build(self) -> (Device, DeviceInfo) {
        let mut ctors = self.ctors;
        ctors.sort_by_key(|ctor| Reverse(ctor.align()));

        (Device(RawDevice::new(&ctors)), DeviceInfo::new(self.info))
    }

    fn find_or_create_ctor<T: Component>(&mut self) -> &mut ComponentCtor {
        let id = RawComponentId::of::<T>();

        let index = self
            .ctors
            .iter()
            .enumerate()
            .find(|(_, ctor)| ctor.id() == id)
            .map(|(i, _)| i);

        let index = match index {
            Some(index) => index,
            None => {
                self.ctors.push(ComponentCtor::new::<T>());
                self.ctors.len() - 1
            }
        };

        &mut self.ctors[index]
    }
}
