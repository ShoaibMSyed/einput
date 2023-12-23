use std::sync::mpsc::{Receiver, Sender};

use crate::UsbDevice;

pub type DriverAction = Box<dyn FnOnce(&mut Driver) -> Result<(), rusb::Error> + Send>;

pub struct Thread {
    pub fn_channel: Receiver<DriverAction>,
    pub device_channel: Sender<UsbDevice>,
}

pub struct Driver {
    device_channel: Sender<UsbDevice>,
}

pub fn init(thread: Thread) {
    std::thread::spawn(move || {
        driver(thread);
    });
}

fn driver(thread: Thread) {
    let mut driver = Driver {
        device_channel: thread.device_channel,
    };

    while let Ok(func) = thread.fn_channel.recv() {
        match func(&mut driver) {
            Ok(()) => {}
            Err(e) => {
                bevy::prelude::warn!("{e}");
            }
        }
    }
}

pub fn refresh(driver: &mut Driver) -> Result<(), rusb::Error> {
    for device in rusb::devices()?.iter() {
        // ignore errors, since if the main thread is closed, the app should be shutting down
        let _ = driver.device_channel.send(UsbDevice(device));
    }

    Ok(())
}
