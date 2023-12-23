#![cfg_attr(not(feature = "std"), no_std)]

use crate::gamepad::Gamepad;

pub mod gamepad;

#[cfg(feature = "device")]
pub fn register(registry: &mut einput_device::ComponentRegistry) {
    registry.register::<Gamepad>("Gamepad", 0);
}
