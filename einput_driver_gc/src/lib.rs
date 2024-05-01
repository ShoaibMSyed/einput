use std::sync::{Arc, Mutex};

use einput_core::EInput;
use log::warn;
use rusb::{GlobalContext, Hotplug, HotplugBuilder};

use self::device::DeviceDriver;

mod device;

type UsbDevice = rusb::Device<GlobalContext>;
type UsbDeviceHandle = rusb::DeviceHandle<GlobalContext>;

const VENDOR_ID: u16 = 0x057E;
const PRODUCT_ID: u16 = 0x0337;

pub fn start(einput: EInput) {
    match HotplugBuilder::new()
        .enumerate(true)
        .register(GlobalContext {}, Box::new(Callback { einput: einput.clone(), numbers: Numbers::default() })) {
        Ok(ctx) => std::mem::forget(ctx),
        Err(e) => warn!("error creating registering hotplug callback: {e}"),
    }
}

struct Callback {
    einput: EInput,
    numbers: Numbers,
}

impl Hotplug<GlobalContext> for Callback {
    fn device_arrived(&mut self, device: UsbDevice) {
        scan(&self.einput, device, &self.numbers);
    }

    fn device_left(&mut self, _device: UsbDevice) {
        
    }
}

fn scan(einput: &EInput, device: UsbDevice, numbers: &Numbers) {
    let desc = match device.device_descriptor() {
        Ok(desc) => desc,
        Err(e) => {
            warn!("error getting device descriptor: {e}");
            return;
        }
    };

    if desc.vendor_id() != VENDOR_ID || desc.product_id() != PRODUCT_ID {
        return;
    }

    let interface = device
        .active_config_descriptor()
        .ok()
        .and_then(|desc| desc
            .interfaces()
            .flat_map(|interface| interface.descriptors())
            .find(|desc| desc.class_code() == 3 && desc.sub_class_code() == 0 && desc.protocol_code() == 0)
            .map(|interface| interface.interface_number())
        );
    
    let Some(interface) = interface
    else {
        warn!("interface not found");
        return
    };

    let einput = einput.clone();
    let number = numbers.get();

    std::thread::spawn(move || {
        let device = match device.open() {
            Ok(handle) => handle,
            Err(e) => {
                warn!("error opening device: {e}");
                return;
            }
        };

        match device.set_auto_detach_kernel_driver(true) {
            Ok(()) => {}
            Err(e) => {
                warn!("error with auto-detach kernel driver: {e}");
            }
        }

        match device.claim_interface(interface) {
            Ok(()) => {}
            Err(e) => {
                warn!("error claiming interface: {e}");
                return;
            }
        }

        let serial = device.read_serial_number_string_ascii(&desc).ok();

        let number = number;
        match DeviceDriver::new(einput, device, number.number, serial).run() {
            Ok(()) => {}
            Err(e) => {
                warn!("{e}");
            }
        }
    });
}

#[derive(Default)]
struct Numbers {
    inner: Arc<Mutex<NumbersInner>>,
}

impl Numbers {
    fn get(&self) -> Number {
        let number = self.inner.lock().expect("Numbers poisoned").get();

        Number {
            number,
            inner: self.inner.clone(),
        }
    }
}

struct Number {
    number: usize,
    inner: Arc<Mutex<NumbersInner>>,
}

impl Drop for Number {
    fn drop(&mut self) {
        let Ok(mut lock) = self.inner.lock()
        else { return };

        lock.free(self.number);
    }
}

#[derive(Default)]
struct NumbersInner {
    next: usize,
    freed: Vec<usize>,
}

impl NumbersInner {
    fn get(&mut self) -> usize {
        match &self.freed[..] {
            [a, ..] => *a,
            _ => {
                self.next += 1;
                self.next - 1
            }
        }
    }

    fn free(&mut self, number: usize) {
        self.freed.push(number);
        self.freed.sort_unstable();
    }
}