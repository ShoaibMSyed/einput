use std::{
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::TrySendError,
        Arc, Mutex, MutexGuard,
    },
};

use bevy::ecs::{component::Component, entity::Entity};
use einput_device::{Device, DeviceConfig, DeviceInfo};
use index_map::IndexMap;

use crate::sync::EventHandle;

pub struct DeviceOwner(Arc<SharedDevice>);

impl DeviceOwner {
    pub fn id(&self) -> Entity {
        self.0.id
    }

    pub fn update<T, F>(&self, updater: F) -> T
    where
        F: FnOnce(&mut Device) -> T,
    {
        let mut raw = self.0.device_raw.lock().unwrap();
        let mut configured = self.0.device_configured.lock().unwrap();
        let config = self.0.config.lock().unwrap();

        let out = updater(&mut raw);
        Device::clone_from(&mut configured, &raw);

        drop(raw);

        config.apply(&mut configured);

        drop(config);
        drop(configured);

        self.0.channels.lock().unwrap().retain(|_, channel| {
            match channel.send.try_send(self.0.id) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) => true,
                Err(TrySendError::Disconnected(_)) => false,
            }
        });

        out
    }
}

impl Drop for DeviceOwner {
    fn drop(&mut self) {
        self.0.owned.store(false, Ordering::Release);
    }
}

pub struct DeviceHandle {
    device: Arc<SharedDevice>,
    channel: Option<usize>,
}

impl DeviceHandle {
    fn new(device: Arc<SharedDevice>) -> Self {
        DeviceHandle {
            device,
            channel: None,
        }
    }

    pub fn config(&self) -> ConfigHandle {
        ConfigHandle::new(&self.device.config)
    }

    pub fn device_raw(&self) -> DeviceView {
        DeviceView::new(&self.device.device_raw)
    }

    pub fn device(&self) -> DeviceView {
        DeviceView::new(&self.device.device_configured)
    }

    pub fn info(&self) -> &DeviceInfo {
        &self.device.info
    }

    pub fn register_events(&mut self, handle: EventHandle) {
        if let Some(channel) = self.channel.take() {
            self.device.channels.lock().unwrap().remove(channel);
        }

        let channel = self.device.channels.lock().unwrap().insert(handle);
        self.channel = Some(channel);
    }
}

impl Clone for DeviceHandle {
    fn clone(&self) -> Self {
        DeviceHandle::new(self.device.clone())
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        if let Some(channel) = self.channel.take() {
            self.device.channels.lock().unwrap().remove(channel);
        }
    }
}

#[derive(Component)]
pub struct EInputDevice(Arc<SharedDevice>);

impl EInputDevice {
    pub fn new(id: Entity, device: Device, info: DeviceInfo, config: DeviceConfig) -> Self {
        EInputDevice(Arc::new(SharedDevice::new(id, device, config, info)))
    }

    pub fn own(&self) -> Option<DeviceOwner> {
        let owned = self.0.owned.swap(true, Ordering::AcqRel);

        if owned {
            None
        } else {
            Some(DeviceOwner(self.0.clone()))
        }
    }

    pub fn handle(&self) -> DeviceHandle {
        DeviceHandle::new(self.0.clone())
    }

    pub fn config(&self) -> ConfigHandle {
        ConfigHandle::new(&self.0.config)
    }

    pub fn device_raw(&self) -> DeviceView {
        DeviceView::new(&self.0.device_raw)
    }

    pub fn device(&self) -> DeviceView {
        DeviceView::new(&self.0.device_configured)
    }

    pub fn info(&self) -> &DeviceInfo {
        &self.0.info
    }
}

struct SharedDevice {
    id: Entity,

    owned: AtomicBool,

    device_raw: Mutex<Device>,
    device_configured: Mutex<Device>,
    config: Mutex<DeviceConfig>,
    info: DeviceInfo,

    channels: Mutex<IndexMap<EventHandle>>,
}

impl SharedDevice {
    fn new(id: Entity, device: Device, config: DeviceConfig, info: DeviceInfo) -> Self {
        SharedDevice {
            id,

            owned: AtomicBool::new(false),

            device_raw: Mutex::new(device.clone()),
            device_configured: Mutex::new(device),
            config: Mutex::new(config),
            info,

            channels: Mutex::default(),
        }
    }
}

pub struct DeviceView<'a>(MutexGuard<'a, Device>);

impl<'a> DeviceView<'a> {
    fn new(mutex: &'a Mutex<Device>) -> Self {
        DeviceView(mutex.lock().unwrap())
    }
}

impl<'a> Deref for DeviceView<'a> {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub struct ConfigHandle<'a>(MutexGuard<'a, DeviceConfig>);

impl<'a> ConfigHandle<'a> {
    fn new(mutex: &'a Mutex<DeviceConfig>) -> Self {
        ConfigHandle(mutex.lock().unwrap())
    }
}

impl<'a> Deref for ConfigHandle<'a> {
    type Target = DeviceConfig;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a> DerefMut for ConfigHandle<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
