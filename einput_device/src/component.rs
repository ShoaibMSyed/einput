use core::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use bytemuck::{Pod, Zeroable};
use hashbrown::HashMap;

use crate::{config::ApplyConfig, lazy::LazySpin, Device};

pub trait Component: Copy + Clone + Default + Send + Sync + Pod + Zeroable + 'static {
    #[cfg(feature = "info")]
    type Info: ComponentInfo;
    #[cfg(feature = "config")]
    type Config: ComponentConfig<Component = Self>;
}

pub trait ComponentInfo: Any + Clone + Default + Send + Sync + 'static {}

impl<T> ComponentInfo for T where T: Any + Clone + Default + Send + Sync + 'static {}

pub trait ComponentConfig: Clone + ApplyConfig + 'static {
    type Component: Component;

    fn apply(&self, to: &mut Self::Component);
}

#[derive(Copy, Clone)]
pub struct ComponentType {
    pub type_id: TypeId,
    pub size: usize,
    pub align: usize,
    pub name: &'static str,
    pub id: RawComponentId,
    pub(crate) default_info: fn() -> Box<dyn Any + Send + Sync>,
}

impl ComponentType {
    fn new<T: Component>(name: &'static str, id: RawComponentId) -> Self {
        ComponentType {
            type_id: TypeId::of::<T>(),
            size: core::mem::size_of::<T>(),
            align: core::mem::align_of::<T>(),
            name,
            id,
            default_info: || Box::new(<T::Info>::default()),
        }
    }
}

static COMPONENT_REGISTRY: LazySpin<ComponentRegistry> = LazySpin::new();

pub fn init_component_registry(registry: ComponentRegistry) {
    COMPONENT_REGISTRY.init(registry);
}

pub fn component_registry<'a>() -> &'a ComponentRegistry {
    COMPONENT_REGISTRY.get()
}

#[derive(Default)]
pub struct ComponentRegistry {
    components: HashMap<TypeId, ComponentType>,
    names: HashMap<&'static str, TypeId>,
    ids: HashMap<RawComponentId, TypeId>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    #[track_caller]
    pub fn register<T: Component>(&mut self, name: &'static str, id: u8) {
        let id = RawComponentId(id);

        let ty = ComponentType::new::<T>(name, id);

        if self.names.contains_key(name) {
            panic!("component already registered with name '{name}'");
        }

        if self.ids.contains_key(&id) {
            panic!("component already registered with id '{}'", id.0);
        }

        self.components.insert(ty.type_id, ty);
        self.names.insert(name, ty.type_id);
        self.ids.insert(id, ty.type_id);
    }

    pub fn min_alignment(&self) -> usize {
        self.all().map(|ty| ty.align).max().unwrap_or(2)
    }

    pub fn all(&self) -> impl Iterator<Item = &ComponentType> {
        self.components.values()
    }

    #[track_caller]
    pub fn get<T: Component>(&self) -> (ComponentType, ComponentId<T>) {
        match self.components.get(&TypeId::of::<T>()) {
            Some(ty) => (*ty, ComponentId(ty.id.0, PhantomData)),
            None => panic!(
                "unregistered component type '{}'",
                core::any::type_name::<T>()
            ),
        }
    }

    #[track_caller]
    pub fn get_by_name(&self, name: &str) -> Option<ComponentType> {
        let type_id = self.names.get(name)?;

        match self.components.get(type_id) {
            Some(ty) => Some(*ty),
            None => panic!("unregistered component with name '{name}'"),
        }
    }

    #[track_caller]
    pub fn get_by_id(&self, id: RawComponentId) -> Option<ComponentType> {
        let type_id = self.ids.get(&id)?;

        match self.components.get(type_id) {
            Some(ty) => Some(*ty),
            None => panic!("unregistered component with id '{}'", id.0),
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct RawComponentId(pub u8);

impl RawComponentId {
    pub fn of<T: Component>() -> Self {
        ComponentId::<T>::new().raw()
    }
}

#[derive(Copy, Clone)]
pub struct ComponentId<T>(u8, PhantomData<T>);

impl<T: Component> ComponentId<T> {
    #[inline]
    pub fn new() -> Self {
        component_registry().get::<T>().1
    }

    #[inline]
    pub fn raw(self) -> RawComponentId {
        RawComponentId(self.0)
    }

    #[inline]
    pub fn get(self, device: &Device, index: u8) -> Option<&T> {
        device.get_by_id(self, index)
    }

    #[inline]
    pub fn get_mut(self, device: &mut Device, index: u8) -> Option<&mut T> {
        device.get_by_id_mut(self, index)
    }

    #[inline]
    pub fn all(self, device: &Device) -> Option<&[T]> {
        device.all_by_id(self)
    }

    #[inline]
    pub fn all_mut(self, device: &mut Device) -> Option<&mut [T]> {
        device.all_by_id_mut(self)
    }
}

impl<T: Component> Default for ComponentId<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
