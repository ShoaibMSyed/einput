use bevy::prelude::*;
use einput_core::{
    device::{init_component_registry, ComponentRegistry},
    EInputPlugin,
};
use einput_driver_enet::EInputDriverENet;
use einput_driver_powera::EInputDriverPowerA;
use einput_driver_usb::EInputDriverUsb;
use einput_egui::EInputEgui;
use einput_output_enet::EInputOutputENet;
use einput_output_uinput::EInputOutputUInput;

fn main() {
    let mut registry = ComponentRegistry::new();
    einput_components::register(&mut registry);
    init_component_registry(registry);

    App::new()
        .add_plugins((
            DefaultPlugins,
            EInputPlugin,
            EInputDriverENet,
            EInputDriverPowerA,
            EInputDriverUsb,
            EInputOutputENet,
            EInputOutputUInput,
            EInputEgui,
        ))
        .run();
}
