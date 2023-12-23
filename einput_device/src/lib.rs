#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_type_id)]

extern crate alloc;

mod builder;
mod component;
mod config;
mod device;
mod info;
mod lazy;
mod raw_device;

pub use self::builder::DeviceBuilder;
pub use self::component::{
    component_registry, init_component_registry, Component, ComponentConfig, ComponentId,
    ComponentRegistry, ComponentType, RawComponentId,
};
pub use self::config::ApplyConfig;
#[cfg(feature = "config")]
pub use self::config::DeviceConfig;
pub use self::device::Device;
pub use self::info::DeviceInfo;
pub use self::raw_device::DevicePtr;
