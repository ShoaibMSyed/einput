pub mod core_device;
pub mod sync;
pub mod thread_command;

use std::sync::Arc;

use bevy::prelude::*;
use device::{Device, DeviceInfo};
use einput_device::DeviceConfig;

use self::core_device::{DeviceOwner, EInputDevice};

pub use einput_device as device;

pub struct EInputPlugin;

impl Plugin for EInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_config);
    }
}

/// Updates the device's config when changed
#[derive(Component, Default, Deref, DerefMut)]
pub struct EInputDeviceConfig(pub DeviceConfig);

#[derive(Bundle)]
pub struct DeviceBundle {
    pub device: EInputDevice,
    pub config: EInputDeviceConfig,
    pub driver: DeviceDriver,
}

impl DeviceBundle {
    pub fn new(
        id: Entity,
        device: Device,
        info: DeviceInfo,
        driver: Entity,
    ) -> (Self, DeviceOwner) {
        let device = EInputDevice::new(id, device, info, default());
        let owner = device.own().unwrap();
        (
            DeviceBundle {
                device,
                config: default(),
                driver: DeviceDriver(driver),
            },
            owner,
        )
    }
}

#[derive(Component)]
pub struct DeviceDriver(pub Entity);

#[derive(Component)]
pub struct DriverInfo {
    pub name: Arc<str>,
}

fn update_config(query: Query<(&EInputDevice, &EInputDeviceConfig), Changed<EInputDeviceConfig>>) {
    for (device, config) in &query {
        device.config().clone_from(&config);
    }
}
