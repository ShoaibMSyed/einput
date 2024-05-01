use std::{sync::{Arc, Mutex}, time::Duration};

use einput_core::{device::Device, output::Output};
use log::info;

mod output;

type Devices = Arc<Mutex<DeviceList>>;

#[derive(Default)]
struct DeviceList {
    list: Vec<Device>,
    changed: bool,
}

pub struct XboxOutput {
    devices: Devices,
}

impl XboxOutput {
    pub fn new() -> Self {
        let devices = Devices::default();
        start(devices.clone());

        Self {
            devices,
        }
    }
}

impl Output for XboxOutput {
    fn name(&self) -> &str {
        "Xbox"
    }

    fn max_devices(&self) -> usize {
        4
    }

    fn update(&mut self, devices: &[Device]) {
        let mut lock = self.devices.lock().unwrap();
        lock.list.clear();
        lock.list.extend_from_slice(devices);
        lock.changed = true;
    }
}

fn start(devices: Devices) {
    std::thread::spawn(move || run(devices));
}

fn run(devices: Devices) {
    loop {
        devices.lock().unwrap().changed = true;
        
        let devices = devices.clone();

        info!("starting vigem client");
        let result = output::run(devices);
        
        match result {
            Ok(()) => info!("vigem client exited"),
            Err(e) => info!("vigem client error: {e}, restarting..."),
        }

        std::thread::sleep(Duration::from_secs(3));
    }
}