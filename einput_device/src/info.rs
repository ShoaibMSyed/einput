use alloc::sync::Arc;
use core::any::Any;

use hashbrown::HashMap;

use crate::{component_registry, Component, Device, RawComponentId};

#[derive(Clone)]
pub struct DeviceInfo(Arc<InternalInfo>);

impl DeviceInfo {
    pub(super) fn new(info: InternalInfo) -> Self {
        DeviceInfo(Arc::new(info))
    }

    pub fn default_from(device: &Device, name: String, id: String) -> Self {
        let registry = component_registry();

        let components = device
            .meta()
            .iter()
            .map(|meta| {
                let info_fn = registry.get_by_id(meta.id).unwrap().default_info;

                let mut vec = Vec::new();
                for _ in 0..meta.count {
                    vec.push(info_fn());
                }

                (meta.id, vec)
            })
            .collect();

        DeviceInfo::new(InternalInfo {
            name,
            id,
            components,
        })
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn id(&self) -> &str {
        &self.0.id
    }

    pub fn component<T: Component>(&self, index: usize) -> Option<&T::Info> {
        self.0
            .components
            .get(&RawComponentId::of::<T>())
            .and_then(|vec| vec.get(index))
            .and_then(|any| any.downcast_ref())
    }
}

pub(super) struct InternalInfo {
    name: String,
    id: String,
    components: HashMap<RawComponentId, Vec<Box<dyn Any + Send + Sync>>>,
}

impl InternalInfo {
    pub fn new(name: String, id: String) -> Self {
        InternalInfo {
            name,
            id,
            components: HashMap::new(),
        }
    }

    pub fn add_info<T: Component>(&mut self, info: impl IntoIterator<Item = T::Info>) -> usize {
        let info_vec = self
            .components
            .entry(RawComponentId::of::<T>())
            .or_default();

        let mut i = 0;
        for info in info {
            info_vec.push(Box::new(info) as _);
            i += 1;
        }
        i
    }
}
