use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use einput_device::{input::DeviceInputConfig, DeviceId, DeviceInfo, DeviceKind};

use self::device::{Device, DeviceOwner};

pub mod device;
pub mod output;

#[allow(dead_code)]
#[derive(Clone)]
pub struct EInput(Arc<Mutex<Inner>>);

impl EInput {
    pub fn new() -> Self {
        EInput(Arc::new(Mutex::new(Inner::new())))
    }

    pub fn get_or_create(&self, id: DeviceId) -> Device {
        let mut lock = self.0.lock().unwrap();

        match lock.devices.get(&id) {
            Some(device) => device.clone(),
            None => {
                let input_config = lock.input_configs.get(&id).cloned().unwrap_or_default();
                let device = Device::new(DeviceInfo::new(
                    id.as_str().to_owned(),
                    id.as_str().to_owned(),
                    id.clone(),
                    DeviceKind::Unknown,
                ), input_config);
                lock.devices.insert(id, device.clone());
                device
            }
        }
    }

    pub fn create_device(&self, info: DeviceInfo) -> Option<DeviceOwner> {
        let mut lock = self.0.lock().unwrap();

        match lock.devices.get_mut(info.id()) {
            Some(device) => device.replace(info),
            None => {
                let id = info.id().clone();

                let input_config = lock.input_configs.get(&id).cloned().unwrap_or_default();

                let device = Device::new(info, input_config);
                let owner = device.create_owner();
                lock.devices.insert(id, device);
                owner
            }
        }
    }

    pub fn devices(&self) -> impl Iterator<Item = Device> {
        let devices: Vec<Device> = self.0.lock().unwrap().devices.values().cloned().collect();

        devices.into_iter()
    }

    pub fn get_input_config(&self, id: &DeviceId) -> Option<DeviceInputConfig> {
        self.0
            .lock()
            .unwrap()
            .input_configs
            .get(id)
            .cloned()
    }

    pub fn set_input_config(&self, id: DeviceId, config: DeviceInputConfig) {
        let mut lock = self.0.lock().unwrap();
        lock.input_configs.insert(id.clone(), config.clone());
        if let Some(dev) = lock.devices.get(&id) {
            *dev.input_config.lock().unwrap() = config;
        }
    }
}

struct Inner {
    devices: HashMap<DeviceId, Device>,
    input_configs: HashMap<DeviceId, DeviceInputConfig>,
}

impl Inner {
    fn new() -> Self {
        Inner {
            devices: HashMap::new(),
            input_configs: HashMap::new(),
        }
    }
}
