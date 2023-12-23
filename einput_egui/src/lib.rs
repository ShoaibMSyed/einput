pub mod widget;

use bevy::{
    ecs::system::{RunSystemOnce, SystemParam, SystemState},
    prelude::*,
};
use bevy_egui::{egui::Ui, EguiContexts, EguiPlugin};
use einput_components::gamepad::{Button, Gamepad};
use einput_core::{core_device::EInputDevice, DeviceDriver, DriverInfo};

use self::widget::{DynamicWidget, WidgetId, WidgetSystem};

pub use bevy_egui;
pub use bevy_egui::egui;

pub struct EInputEgui;

impl Plugin for EInputEgui {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (menu, output_panel, main_panel).chain());
    }
}

#[derive(Component, Clone)]
pub struct OutputWidget(pub DynamicWidget<()>);

fn setup(mut cmd: Commands) {
    cmd.spawn(Camera2dBundle::default());
}

fn menu(mut ectx: EguiContexts) {
    egui::TopBottomPanel::top("menu_bar_panel").show(ectx.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |_ui| {});
        });
    });
}

fn get_context(mut ectx: EguiContexts) -> egui::Context {
    ectx.ctx_mut().clone()
}

fn output_panel(world: &mut World) {
    let ctx = world.run_system_once(get_context);

    let width = ctx.screen_rect().width() / 3.0;

    egui::SidePanel::right("output_panel")
        .exact_width(width)
        .resizable(false)
        .show(&ctx, |ui| {
            OutputPanel::show(world, ui, (), WidgetId::new("output_panel"));
        });
}

fn main_panel(world: &mut World) {
    let ctx = world.run_system_once(get_context);

    egui::CentralPanel::default().show(&ctx, |ui| {
        MainPanel::show(world, ui, (), WidgetId::new("main_panel"));
    });
}

#[derive(SystemParam)]
struct OutputPanel<'w, 's> {
    outputs: Query<'w, 's, (Entity, &'static OutputWidget)>,
}

impl WidgetSystem for OutputPanel<'static, 'static> {
    type Input = ();

    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut Ui,
        input: Self::Input,
        id: WidgetId,
    ) {
        let widgets: Vec<_> = state
            .get(world)
            .outputs
            .iter()
            .map(|(entity, output)| (entity, output.0.clone()))
            .collect();

        for (entity, widget) in widgets {
            ui.add_space(4.0);

            egui::Frame::none()
                .fill(ui.style().visuals.panel_fill)
                .shadow(ui.style().visuals.popup_shadow)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    widget.show(world, ui, input, id.with(entity));
                });
        }
    }
}

#[derive(SystemParam)]
struct MainPanel<'w, 's> {
    devices: Query<'w, 's, Entity, With<EInputDevice>>,
}

impl WidgetSystem for MainPanel<'static, 'static> {
    type Input = ();

    fn system(world: &mut World, state: &mut SystemState<Self>, ui: &mut Ui, _: (), id: WidgetId) {
        let params = state.get_mut(world);
        let devices: Vec<Entity> = params.devices.iter().collect();

        for device in devices {
            ControllerWidget::show(world, ui, device, id.with(device));
            ui.add_space(5.0);
        }
    }
}

#[derive(SystemParam)]
struct ControllerWidget<'w, 's> {
    devices: Query<'w, 's, (&'static EInputDevice, &'static DeviceDriver)>,
    drivers: Query<'w, 's, &'static DriverInfo>,
}

impl WidgetSystem for ControllerWidget<'static, 'static> {
    type Input = Entity;

    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut Ui,
        input: Entity,
        _id: WidgetId,
    ) {
        let params = state.get(world);

        let Ok((device, driver_entity)) = params.devices.get(input) else {
            return;
        };

        egui::Frame::none()
            .fill(ui.style().visuals.panel_fill)
            .shadow(ui.style().visuals.popup_shadow)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(device.info().name()).strong());

                    ui.label(egui::RichText::new(device.info().id()).weak());

                    ui.label(egui::RichText::new(format!("({:?})", input)).weak());

                    if let Ok(info) = params.drivers.get(driver_entity.0) {
                        ui.label(egui::RichText::new(info.name.to_string()).weak());
                    }
                });

                let view = device.device();
                if let Some(gamepad) = view.get::<Gamepad>(0) {
                    ui.horizontal(|ui| {
                        for button in Button::ALL {
                            if gamepad.buttons.pressed(button) {
                                ui.label(format!("{}", button));
                            }
                        }
                    });
                }
            });
    }
}
