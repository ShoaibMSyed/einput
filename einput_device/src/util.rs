use std::{
    collections::HashSet,
    hash::Hash,
    marker::PhantomData,
    num::{NonZeroU16, NonZeroU8},
};

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct Offset<T>(pub NonZeroU16, pub PhantomData<T>);

impl<T> Clone for Offset<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T> Copy for Offset<T> {}

impl<T> Offset<T> {
    pub unsafe fn get<'a>(self, ptr: *const u8) -> &'a T {
        &*ptr.offset(self.0.get() as isize).cast()
    }

    pub unsafe fn get_mut<'a>(self, ptr: *mut u8) -> &'a mut T {
        &mut *ptr.offset(self.0.get() as isize).cast()
    }

    pub unsafe fn write(self, ptr: *mut u8)
    where
        T: Default,
    {
        ptr.offset(self.0.get() as isize)
            .cast::<T>()
            .write(T::default());
    }
}

#[derive(Debug)]
pub(crate) struct SliceOffset<T> {
    pub offset: NonZeroU16,
    pub len: u8,
    pub _ph: PhantomData<T>,
}

impl<T> Clone for SliceOffset<T> {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset.clone(),
            len: self.len.clone(),
            _ph: self._ph.clone(),
        }
    }
}

impl<T> Copy for SliceOffset<T> {}

impl<T> SliceOffset<T> {
    pub unsafe fn get<'a>(self, ptr: *const u8) -> &'a [T] {
        std::slice::from_raw_parts(
            ptr.offset(self.offset.get() as isize).cast(),
            self.len as usize,
        )
    }

    pub unsafe fn get_mut<'a>(self, ptr: *mut u8) -> &'a mut [T] {
        std::slice::from_raw_parts_mut(
            ptr.offset(self.offset.get() as isize).cast(),
            self.len as usize,
        )
    }

    pub unsafe fn write(self, ptr: *mut u8)
    where
        T: Default,
    {
        let ptr = ptr.offset(self.offset.get() as isize).cast::<T>();

        for i in 0..self.len {
            ptr.offset(i as isize).write(T::default());
        }
    }
}

#[derive(Debug)]
pub(crate) struct IdOffset<T, I: InputId>
where
    [u8; I::LEN as usize]:,
{
    pub count: NonZeroU8,
    pub offset: u16,
    pub indices: [u8; I::LEN as usize],
    pub _ph: PhantomData<(T, I)>,
}

impl<T, I: InputId> Clone for IdOffset<T, I>
where
    [u8; I::LEN as usize]:,
{
    fn clone(&self) -> Self {
        Self {
            count: self.count.clone(),
            offset: self.offset.clone(),
            indices: self.indices.clone(),
            _ph: self._ph.clone(),
        }
    }
}

impl<T, I: InputId> Copy for IdOffset<T, I> where [u8; I::LEN as usize]: {}

impl<T, I: InputId> IdOffset<T, I>
where
    [u8; I::LEN as usize]:,
{
    pub unsafe fn get<'a>(self, ptr: *const u8) -> MapRef<'a, I, T> {
        let slice = std::slice::from_raw_parts(
            ptr.offset(self.offset as isize).cast(),
            self.count.get() as usize,
        );

        MapRef::new(slice, self.indices)
    }

    pub unsafe fn get_mut<'a>(self, ptr: *mut u8) -> MapMut<'a, I, T> {
        let slice = std::slice::from_raw_parts_mut(
            ptr.offset(self.offset as isize).cast(),
            self.count.get() as usize,
        );
        MapMut::new(slice, self.indices)
    }

    pub unsafe fn get_one<'a>(self, ptr: *const u8, id: I) -> Option<&'a T> {
        let slice: &[T] = std::slice::from_raw_parts(
            ptr.offset(self.offset as isize).cast(),
            self.count.get() as usize,
        );

        let index = self.indices[id.id() as usize] as usize;
        slice.get(index)
    }

    pub unsafe fn get_one_mut<'a>(self, ptr: *mut u8, id: I) -> Option<&'a mut T> {
        let slice: &mut [T] = std::slice::from_raw_parts_mut(
            ptr.offset(self.offset as isize).cast(),
            self.count.get() as usize,
        );

        let index = self.indices[id.id() as usize] as usize;
        slice.get_mut(index)
    }

    pub unsafe fn write(self, ptr: *mut u8)
    where
        T: Default,
    {
        let ptr = ptr.offset(self.offset as isize).cast::<T>();

        for i in 0..self.count.get() {
            ptr.offset(i as isize).write(T::default());
        }
    }
}

pub struct MapRef<'a, I: InputId, T>
where
    [u8; I::LEN as usize]:,
{
    slice: &'a [T],
    indices: [u8; I::LEN as usize],
    _ph: PhantomData<I>,
}

impl<'a, I: InputId, T> MapRef<'a, I, T>
where
    [u8; I::LEN as usize]:,
{
    pub(crate) fn new(slice: &'a [T], indices: [u8; I::LEN as usize]) -> Self {
        Self {
            slice,
            indices,
            _ph: PhantomData,
        }
    }

    pub fn get(&self, id: I) -> Option<&T> {
        let index = self.indices[id.id() as usize] as usize;

        self.slice.get(index)
    }
}

pub struct MapMut<'a, I: InputId, T>
where
    [u8; I::LEN as usize]:,
{
    slice: &'a mut [T],
    indices: [u8; I::LEN as usize],
    _ph: PhantomData<I>,
}

impl<'a, I: InputId, T> MapMut<'a, I, T>
where
    [u8; I::LEN as usize]:,
{
    pub(crate) fn new(slice: &'a mut [T], indices: [u8; I::LEN as usize]) -> Self {
        Self {
            slice,
            indices,
            _ph: PhantomData,
        }
    }

    pub fn get(&self, id: I) -> Option<&T> {
        let index = self.indices[id.id() as usize] as usize;

        self.slice.get(index)
    }

    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        let index = self.indices[id.id() as usize] as usize;

        self.slice.get_mut(index)
    }
}

pub trait InputId: Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash {
    const LEN: u8;

    fn all() -> [Self; Self::LEN as usize];
    fn id(self) -> u8;
}

pub trait DeviceIndex<D> {
    type Output<'a>
    where
        D: 'a;

    fn index<'a>(self, device: &'a D) -> Self::Output<'a>;
}

pub trait DeviceIndexMut<D> {
    type Output<'a>
    where
        D: 'a;

    fn index_mut<'a>(self, device: &'a mut D) -> Self::Output<'a>;
}

#[derive(Default)]
pub(crate) struct StructBuilder {
    offset: u16,
}

impl StructBuilder {
    pub fn write<T>(&mut self) -> isize {
        self.align(std::mem::align_of::<T>());
        let out = self.offset as isize;
        self.offset = self
            .offset
            .checked_add(
                std::mem::size_of::<T>()
                    .try_into()
                    .expect("type is too large"),
            )
            .expect("struct too large");
        out
    }

    pub fn write_maybe<T>(&mut self, has: bool) -> Option<Offset<T>> {
        if !has {
            return None;
        }

        self.align(std::mem::align_of::<T>());
        let offset =
            NonZeroU16::new(self.offset).expect("component cannot be first element of struct");
        
        self.write::<T>();

        Some(Offset(offset, PhantomData))
    }

    pub fn write_slice<T>(&mut self, len: u8) -> Option<SliceOffset<T>> {
        if len == 0 {
            return None;
        }

        self.align(std::mem::align_of::<T>());
        let offset = NonZeroU16::new(self.offset).expect("slice cannot be first element of struct");

        for _ in 0..len {
            self.write::<T>();
        }

        Some(SliceOffset {
            len,
            offset,
            _ph: PhantomData,
        })
    }

    pub fn write_map<T, I: InputId>(&mut self, set: &HashSet<I>) -> Option<IdOffset<T, I>>
    where
        [u8; I::LEN as usize]:,
    {
        if I::LEN == u8::MAX {
            panic!(
                "InputId type {} has too many ids",
                std::any::type_name::<I>()
            );
        }

        let Some(count) = NonZeroU8::new(set.len() as u8) else {
            return None;
        };

        let mut i = 0;

        let indices = I::all().map(|id| {
            if set.contains(&id) {
                i += 1;
                i - 1
            } else {
                u8::MAX
            }
        });

        self.align(std::mem::align_of::<T>());
        let offset = self.offset;

        for _ in 0..set.len() {
            self.write::<T>();
        }

        Some(IdOffset {
            count,
            offset,
            indices,
            _ph: PhantomData,
        })
    }

    pub fn finish(self) -> usize {
        self.offset as usize
    }

    fn align(&mut self, align: usize) {
        let align: u16 = align.try_into().expect("align is too large");
        if self.offset % align == 0 {
            return;
        }
        self.offset = self
            .offset
            .checked_add(align - self.offset % align)
            .expect("struct too large");
    }
}
