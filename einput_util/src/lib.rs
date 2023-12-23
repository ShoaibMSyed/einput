#![no_std]

mod axis;

use core::ops::{Deref, DerefMut};

use bytemuck::{Pod, Zeroable};

pub use self::axis::{StickAxis, TriggerAxis};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct Stick<T = u8>(pub [T; 2]);

impl<T: StickAxis> Default for Stick<T> {
    fn default() -> Self {
        Self([T::neutral(); 2])
    }
}

impl<T: StickAxis> Deref for Stick<T> {
    type Target = StickVec<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const Self).cast() }
    }
}

impl<T: StickAxis> DerefMut for Stick<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut Self).cast() }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StickVec<T> {
    pub x: T,
    pub y: T,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct Trigger<T = u8>(pub T);

impl<T: TriggerAxis> Trigger<T> {
    pub fn min() -> Self {
        Self(T::min())
    }

    pub fn max() -> Self {
        Self(T::max())
    }
}

impl<T: TriggerAxis> Default for Trigger<T> {
    fn default() -> Self {
        Trigger(T::min())
    }
}

impl<T: TriggerAxis> From<T> for Trigger<T> {
    fn from(value: T) -> Self {
        Trigger(value)
    }
}
