use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use bevy::prelude::*;
use rusb::{GlobalContext, Hotplug, HotplugBuilder, Registration};

use self::driver::DriverAction;

mod driver;

pub type UsbDeviceHandle = rusb::DeviceHandle<GlobalContext>;

pub struct EInputDriverUsb;

impl Plugin for EInputDriverUsb {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init)
            .add_systems(PreUpdate, (add_devices, add_hotplugged_devices).chain());
    }
}

#[derive(Resource)]
struct DriverChannel(Sender<DriverAction>, Mutex<Receiver<UsbDevice>>);

enum DevicePlug {
    Arrived,
    Left,
}

#[derive(Resource)]
struct HotplugCtx {
    _registration: Mutex<Registration<GlobalContext>>,
    devices: Mutex<Receiver<(UsbDevice, DevicePlug)>>,
}

struct HotplugCallback {
    devices: Sender<(UsbDevice, DevicePlug)>,
}

impl Hotplug<GlobalContext> for HotplugCallback {
    fn device_arrived(&mut self, device: rusb::Device<GlobalContext>) {
        let _ = self.devices.send((UsbDevice(device), DevicePlug::Arrived));
    }

    fn device_left(&mut self, device: rusb::Device<GlobalContext>) {
        let _ = self.devices.send((UsbDevice(device), DevicePlug::Left));
    }
}

#[derive(Component)]
pub struct UsbDevice(pub rusb::Device<GlobalContext>);

fn init(world: &mut World) {
    let (send, recv) = std::sync::mpsc::channel();
    let (dsend, drecv) = std::sync::mpsc::channel();

    let channel = DriverChannel(send, Mutex::new(drecv));

    channel.0.send(Box::new(driver::refresh)).unwrap();

    world.insert_resource(channel);

    driver::init(driver::Thread {
        fn_channel: recv,
        device_channel: dsend,
    });

    if rusb::has_hotplug() {
        let (send, recv) = std::sync::mpsc::channel();

        let reg = HotplugBuilder::new().enumerate(false).register(
            GlobalContext {},
            Box::new(HotplugCallback { devices: send }),
        );

        match reg {
            Ok(reg) => {
                world.insert_resource(HotplugCtx {
                    devices: Mutex::new(recv),
                    _registration: Mutex::new(reg),
                });
            }
            Err(e) => {
                warn!("error registering hotplug callback: {e}");
            }
        }
    } else {
        warn!("libusb does not support hotplug");
    }
}

fn add_devices(mut cmd: Commands, channel: Res<DriverChannel>, q_devices: Query<&UsbDevice>) {
    let Ok(recv) = channel.1.lock() else { return };

    for device in recv.try_iter() {
        if q_devices.iter().any(|d| device_equals(&device, d)) {
            continue;
        }

        cmd.spawn(device);
    }
}

fn add_hotplugged_devices(
    mut cmd: Commands,
    hotplug: Res<HotplugCtx>,
    q_devices: Query<(Entity, &UsbDevice)>,
) {
    let Ok(recv) = hotplug.devices.lock() else {
        return;
    };

    let mut to_add = Vec::new();
    let mut to_remove = Vec::new();

    for (device, plug) in recv.try_iter() {
        match plug {
            DevicePlug::Arrived => {
                let mut removed = false;
                to_remove.retain(|d| {
                    if !device_equals(d, &device) {
                        removed = true;
                        true
                    } else {
                        false
                    }
                });

                if !removed {
                    to_add.push(device);
                }
            }
            DevicePlug::Left => {
                let mut removed = false;
                to_add.retain(|d| {
                    if !device_equals(d, &device) {
                        removed = true;
                        true
                    } else {
                        false
                    }
                });

                if !removed {
                    to_remove.push(device);
                }
            }
        }
    }

    for (entity, device) in &q_devices {
        if to_add
            .iter()
            .chain(&to_remove)
            .any(|d| device_equals(d, device))
        {
            cmd.entity(entity).despawn_recursive();
        }
    }

    for device in to_add {
        cmd.spawn(device);
    }
}

fn device_equals(a: &UsbDevice, b: &UsbDevice) -> bool {
    if a.0 == b.0 {
        return true;
    }

    let path_equals = a.0.bus_number() == b.0.bus_number()
        && a.0.address() == b.0.address()
        && a.0.port_number() == b.0.port_number();

    if !path_equals {
        return false;
    }

    if let (Ok(a), Ok(b)) = (a.0.device_descriptor(), b.0.device_descriptor()) {
        if a.product_id() != b.product_id() || a.vendor_id() != b.vendor_id() {
            return false;
        }
    }

    true
}
