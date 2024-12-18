use std::{alloc::Layout, fmt::Debug, ptr::NonNull};

use crate::{
    info::DeviceOutputInfo,
    output::rumble::Rumble,
    util::{DeviceIndex, DeviceIndexMut, SliceOffset, StructBuilder},
};

pub mod rumble;

const ALIGNMENT: usize = {
    const fn a<T: Sized>() -> usize {
        std::mem::align_of::<T>()
    }

    let types = [a::<Header>(), a::<Rumble>()];

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

#[derive(Clone, Copy)]
struct Header {
    rumble: Option<SliceOffset<Rumble>>,
}

pub struct DeviceOutput {
    ptr: NonNull<u8>,
    size: usize,
}

impl Clone for DeviceOutput {
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

        DeviceOutput {
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

impl Drop for DeviceOutput {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: This Layout is valid because it is the Layout used to allocate the Device
            let layout = Layout::from_size_align_unchecked(self.size, ALIGNMENT);

            std::alloc::dealloc(self.ptr.as_ptr(), layout);
        }
    }
}

impl Debug for DeviceOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Device");

        builder.field("size", &self.size);

        builder.field("rumbles", &self.rumbles());

        builder.finish()
    }
}

impl DeviceOutput {
    pub fn new(info: &DeviceOutputInfo) -> Self {
        let mut builder = StructBuilder::default();

        let header_offset = builder.write::<Header>();
        let header = Header {
            rumble: builder.write_slice(info.rumble_motors),
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

            if let Some(rumble) = header.rumble {
                rumble.write(ptr.as_ptr());
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
    DeviceOutput;
    single {

    }
    multi {
        rumble: Rumble;
    }
}
