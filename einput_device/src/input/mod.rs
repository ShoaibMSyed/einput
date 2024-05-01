use std::{alloc::Layout, fmt::Debug, ptr::NonNull};

use einput_util::axis::{Stick, Trigger};

use crate::{
    info::DeviceInputInfo,
    util::{DeviceIndex, DeviceIndexMut, IdOffset, Offset, StructBuilder},
};

pub use self::config::{DeviceInputConfig, StickConfig, TriggerConfig};
use self::{
    acceleration::Acceleration,
    buttons::Buttons,
    gyroscope::Gyroscope,
    stick::StickId,
    triggers::Triggers,
};

pub mod acceleration;
pub mod buttons;
pub mod config;
pub mod gyroscope;
pub mod stick;
pub mod triggers;

const ALIGNMENT: usize = {
    const fn a<T: Sized>() -> usize {
        std::mem::align_of::<T>()
    }

    let types = [
        a::<Header>(),
        a::<Acceleration>(),
        a::<Buttons>(),
        a::<Gyroscope>(),
        a::<Stick>(),
        a::<Trigger>(),
    ];

    let mut align = 8;

    let mut i = 0;
    while i < types.len() {
        let ty = types[i];
        if ty > align {
            align = ty;
        }
        i += 1;
    }

    align
};

#[derive(Clone, Copy, Debug)]
struct Header {
    acceleration: Option<Offset<Acceleration>>,
    buttons: Option<Offset<Buttons>>,
    gyroscope: Option<Offset<Gyroscope>>,
    sticks: Option<IdOffset<Stick, StickId>>,
    triggers: Option<Offset<Triggers>>,
}

pub struct DeviceInput {
    ptr: NonNull<u8>,
    size: usize,
}

impl Clone for DeviceInput {
    fn clone(&self) -> Self {
        let layout = Layout::from_size_align(self.size, ALIGNMENT).expect("invalid layout");
        let ptr = unsafe { std::alloc::alloc(layout) };

        let Some(ptr) = NonNull::new(ptr) else {
            std::alloc::handle_alloc_error(layout)
        };

        unsafe {
            ptr.as_ptr()
                .copy_from_nonoverlapping(self.ptr.as_ptr(), self.size)
        }

        DeviceInput {
            ptr,
            size: self.size,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        if self.size != source.size {
            *self = source.clone();
        } else {
            unsafe {
                self.ptr
                    .as_ptr()
                    .copy_from_nonoverlapping(source.ptr.as_ptr(), self.size)
            }
        }
    }
}

impl Drop for DeviceInput {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: This Layout is valid because it is the Layout used to allocate the Device
            let layout = Layout::from_size_align_unchecked(self.size, ALIGNMENT);

            std::alloc::dealloc(self.ptr.as_ptr(), layout);
        }
    }
}

impl Debug for DeviceInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Device");

        builder.field("size", &self.size);

        if let Some(accel) = self.acceleration() {
            builder.field("acceleration", accel);
        }

        if let Some(buttons) = self.buttons() {
            builder.field("buttons", buttons);
        }

        if let Some(gyro) = self.gyroscope() {
            builder.field("gyroscope", gyro);
        }

        for id in StickId::ALL {
            let Some(stick) = self.get(id) else { continue };

            builder.field(&format!("stick_{id:#?}"), stick);
        }

        if let Some(triggers) = self.triggers() {
            builder.field("triggers", triggers);
        }

        builder.finish()
    }
}

impl DeviceInput {
    pub fn new(info: &DeviceInputInfo) -> Self {
        let mut builder = StructBuilder::default();

        let header_offset = builder.write::<Header>();
        let header = Header {
            acceleration: builder.write_maybe(info.acceleration),
            buttons: builder.write_maybe(info.buttons.get_pressed().next().is_some()),
            gyroscope: builder.write_maybe(info.gyroscope),
            sticks: builder.write_map(&info.sticks),
            triggers: builder.write_maybe(info.triggers),
        };

        let size = builder.finish();

        unsafe {
            let layout = Layout::from_size_align(size, ALIGNMENT).expect("invalid layout");
            let ptr = std::alloc::alloc(layout);
            let Some(ptr) = NonNull::new(ptr) else {
                std::alloc::handle_alloc_error(layout)
            };

            ptr.as_ptr()
                .offset(header_offset)
                .cast::<Header>()
                .write(header);

            if let Some(acceleration) = header.acceleration {
                acceleration.write(ptr.as_ptr());
            }

            if let Some(buttons) = header.buttons {
                buttons.write(ptr.as_ptr());
            }

            if let Some(gyroscope) = header.gyroscope {
                gyroscope.write(ptr.as_ptr());
            }

            if let Some(sticks) = header.sticks {
                sticks.write(ptr.as_ptr());
            }

            if let Some(triggers) = header.triggers {
                triggers.write(ptr.as_ptr());
            }

            Self { ptr, size }
        }
    }

    fn header(&self) -> &Header {
        unsafe { &*self.ptr.as_ptr().cast() }
    }

    pub fn get<'a, I>(&'a self, index: I) -> I::Output<'a>
    where
        I: DeviceIndex<Self>,
    {
        index.index(self)
    }

    pub fn get_mut<'a, I>(&'a mut self, index: I) -> I::Output<'a>
    where
        I: DeviceIndexMut<Self>,
    {
        index.index_mut(self)
    }
}

super::component_accessors! {
    DeviceInput;
    single {
        acceleration: Acceleration;
        buttons: Buttons;
        gyroscope: Gyroscope;
        triggers: Triggers;
    }
    multi {

    }
    map {
        stick: Stick[StickId];
    }
}

unsafe impl Send for DeviceInput {}
unsafe impl Sync for DeviceInput {}