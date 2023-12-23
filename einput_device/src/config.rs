use core::any::Any;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{device::Device, ComponentConfig, RawComponentId};

#[cfg_attr(feature = "typetag", typetag::serde(tag = "type"))]
pub trait ApplyConfig: Any + Send + Sync + 'static {
    fn apply_to(&self, device: &mut Device, index: u8);
    fn clone(&self) -> Box<dyn ApplyConfig>;

    #[doc(hidden)]
    fn __component_id(&self) -> RawComponentId;
}

#[cfg(feature = "config")]
#[macro_export]
macro_rules! impl_apply_config {
    ($typ:ty) => {
        #[typetag::serde]
        impl $crate::ApplyConfig for $typ {
            fn apply_to(&self, device: &mut $crate::Device, index: u8) {
                if let Some(component) =
                    device.get_mut::<<Self as $crate::ComponentConfig>::Component>(index)
                {
                    <Self as $crate::ComponentConfig>::apply(self, component);
                }
            }

            fn clone(&self) -> Box<dyn $crate::ApplyConfig> {
                Box::new(<Self as Clone>::clone(self))
            }

            fn __component_id(&self) -> $crate::RawComponentId {
                $crate::RawComponentId::of::<<Self as $crate::ComponentConfig>::Component>()
            }
        }
    };
}

#[cfg(not(feature = "config"))]
#[macro_export]
macro_rules! impl_apply_config {
    ($typ:ty) => {
        impl $crate::ApplyConfig for $typ {
            fn apply_to(&self, device: &mut $crate::Device, index: u8) {
                if let Some(component) =
                    device.get_mut::<<Self as $crate::ComponentConfig>::Component>(index)
                {
                    <Self as $crate::ComponentConfig>::apply(self, component);
                }
            }

            fn clone(&self) -> Box<dyn $crate::ApplyConfig> {
                Box::new(<Self as Clone>::clone(self))
            }

            fn __component_id(&self) -> $crate::RawComponentId {
                $crate::RawComponentId::of::<<Self as $crate::ComponentConfig>::Component>()
            }
        }
    };
}

#[cfg(feature = "config")]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(from = "SerdeConfig", into = "SerdeConfig")]
pub struct DeviceConfig {
    configs: HashMap<ConfigIndex, CloneableConfig>,
}

#[cfg(feature = "config")]
impl DeviceConfig {
    pub fn new() -> Self {
        DeviceConfig::default()
    }

    pub fn apply(&self, device: &mut Device) {
        for (index, config) in &self.configs {
            config.0.apply_to(device, index.index);
        }
    }

    pub fn insert<T: ComponentConfig>(&mut self, config: T, index: u8) -> Option<T> {
        let index = ConfigIndex::new::<T>(index);

        let prev = self
            .configs
            .insert(index, CloneableConfig(Box::new(config)));

        match prev {
            Some(prev) => {
                let config = prev.0;
                let typed = (config as Box<dyn Any>).downcast::<T>().ok()?;
                Some(*typed)
            }
            None => None,
        }
    }

    pub fn get<T: ComponentConfig>(&self, index: u8) -> Option<&T> {
        let index = ConfigIndex::new::<T>(index);

        self.configs
            .get(&index)
            .and_then(|config| (&config.0 as &dyn Any).downcast_ref())
    }

    pub fn get_mut<T: ComponentConfig>(&mut self, index: u8) -> Option<&mut T> {
        let index = ConfigIndex::new::<T>(index);

        self.configs
            .get_mut(&index)
            .and_then(|config| (&mut config.0 as &mut dyn Any).downcast_mut())
    }

    pub fn remove<T: ComponentConfig>(&mut self, index: u8) -> Option<T> {
        let index = ConfigIndex::new::<T>(index);

        let config = self.configs.remove(&index)?;
        let config = config.0;
        let typed = (config as Box<dyn Any>).downcast::<T>().ok()?;
        Some(*typed)
    }
}

#[cfg(feature = "config")]
impl From<SerdeConfig> for DeviceConfig {
    fn from(value: SerdeConfig) -> Self {
        DeviceConfig {
            configs: value
                .0
                .into_iter()
                .map(|(index, config)| {
                    let component = config.__component_id();
                    (ConfigIndex { component, index }, CloneableConfig(config))
                })
                .collect(),
        }
    }
}

#[cfg(feature = "config")]
impl From<DeviceConfig> for SerdeConfig {
    fn from(value: DeviceConfig) -> Self {
        SerdeConfig(
            value
                .configs
                .into_iter()
                .map(|(index, config)| (index.index, config.0))
                .collect(),
        )
    }
}

#[cfg(feature = "config")]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct ConfigIndex {
    component: RawComponentId,
    index: u8,
}

#[cfg(feature = "config")]
impl ConfigIndex {
    fn new<T: ComponentConfig>(index: u8) -> Self {
        ConfigIndex {
            component: RawComponentId::of::<T::Component>(),
            index,
        }
    }
}

#[cfg(feature = "config")]
struct CloneableConfig(Box<dyn ApplyConfig>);

#[cfg(feature = "config")]
impl Clone for CloneableConfig {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(feature = "config")]
#[derive(Serialize, Deserialize)]
struct SerdeConfig(Vec<(u8, Box<dyn ApplyConfig>)>);
