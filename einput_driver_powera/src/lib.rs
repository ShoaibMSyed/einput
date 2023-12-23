mod driver;

use std::thread::JoinHandle;

use bevy::prelude::*;
use einput_components::gamepad::{Gamepad, GamepadInfo};
use einput_core::{device::Device, DeviceBundle, DriverInfo};
use einput_driver_usb::UsbDevice;

use self::driver::Args;

const VENDOR_ID: u16 = 0x20D6;
const PRODUCT_ID: u16 = 0xA713;

pub struct EInputDriverPowerA;

impl Plugin for EInputDriverPowerA {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (check_devices, remove_handles));
    }
}

#[derive(Resource)]
struct DriverState {
    count: usize,
    driver: Entity,
}

fn setup(mut cmd: Commands) {
    let driver = cmd
        .spawn(DriverInfo {
            name: "powera".into(),
        })
        .id();

    cmd.insert_resource(DriverState { count: 0, driver });
}

#[derive(Component)]
struct Handle(JoinHandle<anyhow::Result<()>>);

fn check_devices(
    mut cmd: Commands,
    mut devs: ResMut<DriverState>,
    q_devices: Query<&UsbDevice, Added<UsbDevice>>,
) {
    for udev in &q_devices {
        let Ok(desc) = udev.0.device_descriptor() else {
            continue;
        };

        if desc.vendor_id() != VENDOR_ID || desc.product_id() != PRODUCT_ID {
            continue;
        }

        match udev.0.open() {
            Ok(handle) => {
                devs.count += 1;
                let id = devs.count;

                let serial = match handle.read_serial_number_string_ascii(&desc) {
                    Ok(t) => t,
                    Err(e) => {
                        warn!("error reading serial number: {e}");
                        format!("powera_wired_pro{id}")
                    }
                };

                let name = match id {
                    1 => format!("PowerA Wired Pro Controller"),
                    _ => format!("PowerA Wired Pro Controller {id}"),
                };

                let (device, info) = Device::builder(name, serial)
                    .add_components::<Gamepad>([GamepadInfo::default()])
                    .build();

                let entity = cmd.spawn_empty().id();
                let (bundle, owner) = DeviceBundle::new(entity, device, info, devs.driver);

                let mut entity = cmd.entity(entity);
                entity.insert(bundle);

                let handle = driver::start(Args { handle, owner });

                entity.insert(Handle(handle));
            }
            Err(e) => {
                warn!("error opening usb device: {e}");
            }
        }
    }
}

fn remove_handles(mut cmd: Commands, q_handles: Query<(Entity, &Handle)>) {
    for (entity, handle) in &q_handles {
        if handle.0.is_finished() {
            // TODO: Log errors from thread
            cmd.entity(entity).despawn_recursive();
        }
    }
}
