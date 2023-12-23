#![no_std]

extern crate alloc;

use core::{alloc::Layout, ptr::NonNull};

use alloc::vec::Vec;
use einput_device::{component_registry, Device, DevicePtr};

// Packet format (LITTLE ENDIAN):
// Device Id (u64)
// Alignment (u64)
// Padding bytes to alignment (start of packet buffer should be aligned to alignment)
// Device Bytes

pub struct PacketWriter {
    buf: Vec<u8>,
    device_start: usize,
}

impl PacketWriter {
    pub fn new(id: u64, device: &Device) -> Self {
        let mut buf = Vec::new();

        let align = component_registry().min_alignment();

        buf.extend_from_slice(&id.to_le_bytes());

        buf.extend_from_slice(&(align as u64).to_le_bytes());

        let offset = buf.len();
        let cur_align = offset % align;
        let offset = if cur_align == 0 {
            offset
        } else {
            offset + (align - cur_align)
        };
        let padding = offset - buf.len();

        buf.extend(core::iter::repeat(0).take(padding));

        let device_start = buf.len();

        buf.extend_from_slice(device.bytes());

        PacketWriter { buf, device_start }
    }

    pub fn update(&mut self, device: &Device) {
        let bytes = self.device_bytes();

        if bytes.len() != device.bytes().len() {
            log::warn!("tried to update packet with wrong device");
            return;
        }

        bytes.copy_from_slice(device.bytes());
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    fn device_bytes(&mut self) -> &mut [u8] {
        let start = self.device_start;
        let len = self.device_length();

        &mut self.buf[start..][..len]
    }

    fn device_length(&self) -> usize {
        self.buf.len() - self.device_start
    }
}

pub struct PacketBuf {
    buf: NonNull<u8>,
    layout: Layout,
    len: usize,
}

impl PacketBuf {
    pub fn new(size: usize) -> Self {
        if size < 16 {
            panic!("packet must have at least 16 bytes");
        }

        let align = component_registry().min_alignment().max(8);
        let layout = Layout::from_size_align(size, align).expect("invalid layout");

        let buf =
            NonNull::new(unsafe { alloc::alloc::alloc_zeroed(layout) }).expect("allocation error");

        PacketBuf {
            buf,
            layout,
            len: 0,
        }
    }

    pub fn write<E>(
        &mut self,
        writer: impl FnOnce(&mut [u8]) -> Result<usize, E>,
    ) -> Result<(), E> {
        let slice =
            unsafe { core::slice::from_raw_parts_mut(self.buf.as_ptr(), self.layout.size()) };
        let written = writer(slice)?;
        self.len = written;
        Ok(())
    }

    pub fn id(&self) -> u64 {
        unsafe { *self.buf.as_ptr().cast::<u64>() }
    }

    fn align(&self) -> u64 {
        unsafe { *self.buf.as_ptr().offset(8).cast::<u64>() }
    }

    pub fn device(&self) -> Result<&DevicePtr, ()> {
        if self.len < 16 {
            return Err(());
        }

        let align = self.align();

        if align != component_registry().min_alignment() as u64 {
            return Err(());
        }

        let offset = 16;
        let cur_align = offset % align;
        let offset = if cur_align == 0 {
            offset
        } else {
            offset + (align - cur_align)
        };

        let bytes = unsafe {
            core::slice::from_raw_parts(self.buf.as_ptr().offset(offset as _), self.len - 16)
        };

        DevicePtr::from_bytes(bytes)
    }
}
