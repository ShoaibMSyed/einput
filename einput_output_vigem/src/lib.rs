use std::{sync::{Arc, Mutex}, time::Duration};

use einput_core::{device::Device, output::Output, EInput};
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
    pub fn new(einput: EInput) -> Self {
        let devices = Devices::default();
        start(einput, devices.clone());

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

fn start(einput: EInput, devices: Devices) {
    std::thread::spawn(move || run(einput, devices));
}

fn run(einput: EInput, devices: Devices) {
    loop {
        devices.lock().unwrap().changed = true;
        
        let einput = einput.clone();
        let devices = devices.clone();

        info!("starting vigem thread");
        let handle = std::thread::spawn(move || output::run(einput, devices));
        
        match handle.join() {
            Ok(Ok(())) => info!("vigem thread exited"),
            Ok(Err(e)) => info!("vigem thread error: {e}, restarting..."),
            Err(_) => info!("vigem thread crashed, restarting..."),
        }

        std::thread::sleep(Duration::from_secs(3));
    }
}