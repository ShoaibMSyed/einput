use std::{fmt::Debug, hash::Hasher, sync::Arc};

use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
    utils::HashMap,
};
use bevy_egui::egui::Ui;

#[derive(Clone)]
pub struct DynamicWidget<T>(Arc<dyn Fn(&mut World, &mut Ui, T, WidgetId) + Send + Sync>);

impl<T> DynamicWidget<T> {
    pub fn new<W: WidgetSystem<Input = T>>() -> Self {
        DynamicWidget(Arc::new(W::show))
    }

    pub fn show(&self, world: &mut World, ui: &mut Ui, input: T, id: WidgetId) {
        (self.0)(world, ui, input, id);
    }
}

pub trait WidgetSystem: Send + Sync + SystemParam + 'static {
    type Input;

    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut Ui,
        input: Self::Input,
        id: WidgetId,
    );

    fn show(world: &mut World, ui: &mut Ui, input: Self::Input, id: WidgetId) {
        widget::<Self>(world, ui, input, id)
    }
}

pub fn widget<S: 'static + WidgetSystem>(
    world: &mut World,
    ui: &mut Ui,
    input: S::Input,
    id: WidgetId,
) {
    // We need to cache `SystemState` to allow for a system's locally tracked state
    if !world.contains_resource::<StateInstances<S>>() {
        // Note, this message should only appear once! If you see it twice in the logs, the function
        // may have been called recursively, and will panic.
        debug!("Init system state {}", std::any::type_name::<S>());
        world.insert_resource(StateInstances::<S> {
            instances: HashMap::new(),
        });
    }

    world.resource_scope(|world, mut states: Mut<StateInstances<S>>| {
        if !states.instances.contains_key(&id) {
            debug!(
                "Registering system state for widget {id:?} of type {}",
                std::any::type_name::<S>()
            );
            states.instances.insert(id, SystemState::new(world));
        }
        let cached_state = states.instances.get_mut(&id).unwrap();
        S::system(world, cached_state, ui, input, id);
        cached_state.apply(world);
    });
}

/// A UI widget may have multiple instances. We need to ensure the local state of these instances is
/// not shared. This hashmap allows us to dynamically store instance states.
#[derive(Default, Resource)]
struct StateInstances<T: WidgetSystem> {
    instances: HashMap<WidgetId, SystemState<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub fn new(name: &str) -> Self {
        let bytes = name.as_bytes();
        let mut hasher = bevy::utils::AHasher::default();
        hasher.write(bytes);
        WidgetId(hasher.finish())
    }

    pub fn with<T: Debug>(&self, name: T) -> WidgetId {
        Self::new(&format!("{}{name:?}", self.0))
    }
}
