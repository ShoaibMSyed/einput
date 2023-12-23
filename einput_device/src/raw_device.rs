use core::{alloc::Layout, ptr::NonNull};

use bytemuck::{Pod, Zeroable};
use hashbrown::HashMap;

use crate::{component_registry, Component, ComponentId, Device, RawComponentId};

pub(crate) struct ComponentCtor {
    id: RawComponentId,
    pub count: u8,
    init: fn(&mut [u8]),

    size: usize,
    align: usize,
}

impl ComponentCtor {
    pub fn new<T: Component>() -> Self {
        ComponentCtor {
            id: RawComponentId::of::<T>(),
            count: 0,
            init: |bytes| {
                let t = T::default();
                let slice = bytemuck::bytes_of(&t);
                bytes[..slice.len()].copy_from_slice(slice);
            },

            size: core::mem::size_of::<T>(),
            align: core::mem::align_of::<T>(),
        }
    }

    pub fn id(&self) -> RawComponentId {
        self.id
    }

    pub fn align(&self) -> usize {
        self.align
    }
}

pub(crate) struct RawDevice {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl RawDevice {
    pub fn new(ctors: &[ComponentCtor]) -> Self {
        let offsets = Offsets::get(ctors);

        assert_eq!(offsets.header, 0);
        assert_eq!(offsets.meta, 2);

        let header = RawDeviceHeader {
            component_types: ctors.len().try_into().expect("too many component types"),
        };
        let metas: Vec<ComponentMeta> = ctors
            .iter()
            .map(|ctor| ComponentMeta {
                id: ctor.id,
                count: ctor.count,
                offset: *offsets.components.get(&ctor.id).unwrap(),
            })
            .collect();

        let align = component_registry().min_alignment();

        let layout = Layout::from_size_align(offsets.size, align).expect("invalid device layout");

        unsafe {
            let ptr = NonNull::new(alloc::alloc::alloc_zeroed(layout)).expect("allocation error");

            {
                let mut ptr = ptr.as_ptr();
                ptr.cast::<RawDeviceHeader>().write(header);
                ptr = ptr.offset(2);

                for meta in metas {
                    ptr.cast::<ComponentMeta>().write(meta);
                    ptr = ptr.offset(
                        core::mem::size_of::<ComponentMeta>()
                            .try_into()
                            .expect("component meta too large"),
                    );
                }
            }

            {
                for ctor in ctors {
                    let offset = *offsets.components.get(&ctor.id).unwrap();
                    let mut ptr = ptr
                        .as_ptr()
                        .offset(offset.try_into().expect("offset too large"));

                    for _ in 0..ctor.count {
                        let slice = core::slice::from_raw_parts_mut(ptr, ctor.size);
                        (ctor.init)(slice);
                        ptr = ptr.offset(ctor.size.try_into().expect("component too large"));
                    }
                }
            }

            RawDevice { ptr, layout }
        }
    }
}

impl Clone for RawDevice {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = alloc::alloc::alloc_zeroed(self.layout);
            let ptr = NonNull::new(ptr).expect("allocation failed");

            core::ptr::copy_nonoverlapping(self.ptr.as_ptr(), ptr.as_ptr(), self.layout.size());

            RawDevice {
                ptr,
                layout: self.layout,
            }
        }
    }

    fn clone_from(&mut self, source: &Self) {
        if self.layout == source.layout {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    source.ptr.as_ptr(),
                    self.ptr.as_ptr(),
                    self.layout.size(),
                );
            }
        } else {
            *self = source.clone();
        }
    }
}

impl Drop for RawDevice {
    fn drop(&mut self) {
        let ptr = core::mem::replace(&mut self.ptr, NonNull::dangling());
        let layout = core::mem::replace(&mut self.layout, Layout::for_value::<[u8]>(&[]));

        unsafe {
            alloc::alloc::dealloc(ptr.as_ptr(), layout);
        }
    }
}

unsafe impl Send for RawDevice {}
unsafe impl Sync for RawDevice {}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
struct RawDeviceHeader {
    component_types: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, PartialEq, Eq, Zeroable)]
pub(crate) struct ComponentMeta {
    pub(crate) id: RawComponentId,
    pub(crate) count: u8,
    pub(crate) offset: u16,
}

struct Offsets {
    header: u16,
    meta: u16,
    components: HashMap<RawComponentId, u16>,
    size: usize,
}

impl Offsets {
    fn get(ctors: &[ComponentCtor]) -> Self {
        let mut offset = Offset(0);

        let header: u16 = offset.0.try_into().expect("offset too large");
        offset.write::<RawDeviceHeader>(1);
        let meta: u16 = offset.0.try_into().expect("offset too large");

        let mut components = HashMap::new();

        for ctor in ctors {
            offset.align(ctor.align);

            components.insert(ctor.id, offset.0.try_into().expect("offset too large"));

            offset.write_raw(ctor.align, ctor.size, ctor.count.into());
        }

        let size = offset.0;

        Offsets {
            header,
            meta,
            components,
            size,
        }
    }
}

struct Offset(usize);

impl Offset {
    fn write<T>(&mut self, count: usize) {
        self.write_raw(core::mem::align_of::<T>(), core::mem::size_of::<T>(), count)
    }

    fn write_raw(&mut self, align: usize, size: usize, count: usize) {
        self.align(align);
        self.offset(size.checked_mul(count).expect("offset was too large"));
    }

    fn align(&mut self, align: usize) {
        let cur_align = self.0 % align;
        if cur_align != 0 {
            self.0 += align - cur_align;
        }
    }

    fn offset(&mut self, amount: usize) {
        self.0 = self.0.checked_add(amount).expect("offset was too large");
    }
}

#[repr(transparent)]
pub struct DevicePtr([u8]);

impl DevicePtr {
    #[inline]
    pub(crate) fn new(device: &RawDevice) -> &Self {
        unsafe {
            let slice = core::slice::from_raw_parts(device.ptr.as_ptr(), device.layout.size());
            core::mem::transmute(slice)
        }
    }

    #[inline]
    pub(crate) fn new_mut(device: &mut RawDevice) -> &mut Self {
        unsafe {
            let slice = core::slice::from_raw_parts_mut(device.ptr.as_ptr(), device.layout.size());
            core::mem::transmute(slice)
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ()> {
        let registry = component_registry();

        if bytes.as_ptr() as *const u8 as usize % registry.min_alignment() != 0 {
            return Err(());
        }

        let header: RawDeviceHeader = bytemuck::cast(read_bytes::<2>(bytes)?);

        for i in 0..header.component_types {
            let meta: ComponentMeta =
                bytemuck::cast(read_bytes::<4>(&bytes[(2 + i as usize * 4)..])?);
            let ty = registry.get_by_id(meta.id).ok_or(())?;
            if meta.offset as usize % ty.align != 0 {
                return Err(());
            }

            if meta.offset as usize + meta.count as usize * ty.size > bytes.len() {
                return Err(());
            }
        }

        Ok(unsafe { core::mem::transmute(bytes) })
    }

    fn header(&self) -> &RawDeviceHeader {
        let ptr = self.0.as_ptr();

        unsafe { &*ptr.cast::<RawDeviceHeader>() }
    }

    pub(crate) fn meta(&self) -> &[ComponentMeta] {
        unsafe {
            let header = *self.header();
            let ptr = self.0.as_ptr();
            let slice =
                core::slice::from_raw_parts(ptr.offset(2).cast(), header.component_types.into());
            slice
        }
    }

    fn meta_for(&self, id: RawComponentId) -> Option<ComponentMeta> {
        self.meta().iter().find(|meta| meta.id == id).copied()
    }

    #[inline]
    pub fn component_ids(&self) -> impl Iterator<Item = RawComponentId> + '_ {
        self.meta().iter().map(|meta| meta.id)
    }

    #[inline]
    pub fn all<T: Component>(&self) -> Option<&[T]> {
        self.all_by_id(ComponentId::new())
    }

    #[inline]
    pub fn all_by_id<T: Component>(&self, id: ComponentId<T>) -> Option<&[T]> {
        let meta = self.meta_for(id.raw())?;
        unsafe {
            Some(core::slice::from_raw_parts(
                self.0.as_ptr().offset(meta.offset as _).cast(),
                meta.count.into(),
            ))
        }
    }

    #[inline]
    pub fn all_mut<T: Component>(&mut self) -> Option<&mut [T]> {
        self.all_by_id_mut(ComponentId::new())
    }

    #[inline]
    pub fn all_by_id_mut<T: Component>(&mut self, id: ComponentId<T>) -> Option<&mut [T]> {
        let meta = self.meta_for(id.raw())?;
        unsafe {
            Some(core::slice::from_raw_parts_mut(
                self.0.as_mut_ptr().offset(meta.offset as _).cast(),
                meta.count.into(),
            ))
        }
    }

    #[inline]
    pub fn get<T: Component>(&self, index: u8) -> Option<&T> {
        self.get_by_id(ComponentId::new(), index)
    }

    #[inline]
    pub fn get_mut<T: Component>(&mut self, index: u8) -> Option<&mut T> {
        self.get_by_id_mut(ComponentId::new(), index)
    }

    #[inline]
    pub fn get_by_id<T: Component>(&self, id: ComponentId<T>, index: u8) -> Option<&T> {
        self.all_by_id(id)
            .and_then(|slice| slice.get(index as usize))
    }

    #[inline]
    pub fn get_by_id_mut<T: Component>(&mut self, id: ComponentId<T>, index: u8) -> Option<&mut T> {
        self.all_by_id_mut(id)
            .and_then(|slice| slice.get_mut(index as usize))
    }

    pub(crate) fn update(&mut self, from: &DevicePtr) -> Result<(), ()> {
        if self.header() != from.header()
            || self.meta() != from.meta()
            || self.0.len() != from.0.len()
        {
            Err(())
        } else {
            self.0.copy_from_slice(&from.0);

            Ok(())
        }
    }
}

impl ToOwned for DevicePtr {
    type Owned = Device;

    fn to_owned(&self) -> Self::Owned {
        let size = self.0.len();
        let align = component_registry().min_alignment();
        let layout = Layout::from_size_align(size, align).expect("invalid layout");

        let ptr =
            NonNull::new(unsafe { alloc::alloc::alloc_zeroed(layout) }).expect("allocation error");

        unsafe {
            core::ptr::copy_nonoverlapping(self.0.as_ptr(), ptr.as_ptr(), size);
        }

        Device(RawDevice { layout, ptr })
    }
}

fn read_bytes<const N: usize>(bytes: &[u8]) -> Result<[u8; N], ()> {
    if bytes.len() < N {
        return Err(());
    }

    Ok(bytes[..N].try_into().unwrap())
}
