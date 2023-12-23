mod driver;

use std::{sync::mpsc::Sender, thread::JoinHandle};

use anyhow::Result;
use bevy::{
    ecs::system::{RunSystemOnce, SystemParam, SystemState},
    prelude::*,
};
use einput_core::core_device::{DeviceHandle, EInputDevice};
use einput_egui::{
    egui::{ComboBox, Id, RichText, Ui},
    widget::{DynamicWidget, WidgetId, WidgetSystem},
    OutputWidget,
};

use self::driver::{Args, DriverCommand, NetDevice};

pub struct EInputOutputENet;

impl Plugin for EInputOutputENet {
    fn build(&self, app: &mut App) {
        app.init_resource::<Config>()
            .init_resource::<State>()
            .add_systems(Startup, init)
            .add_systems(PreUpdate, check_state);
    }
}

#[derive(Resource)]
struct Config {
    ip: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ip: String::from("127.0.0.1:29810"),
        }
    }
}

#[derive(Resource, Default)]
enum State {
    Init {
        handle: JoinHandle<Result<()>>,
        commands: Sender<DriverCommand>,
        current_ip: String,

        devices: Vec<Entity>,
    },
    #[default]
    Uninit,
}

fn init(mut cmd: Commands) {
    cmd.spawn(OutputWidget(DynamicWidget::new::<OutputPanel>()));
}

fn start(mut cmd: Commands, config: Res<Config>) {
    let (send, recv) = std::sync::mpsc::channel();

    let handle = driver::start(Args {
        commands: recv,
        ip: config.ip.clone(),
    });

    cmd.insert_resource(State::Init {
        handle,
        commands: send,
        current_ip: config.ip.clone(),

        devices: Vec::new(),
    });
}

fn check_state(mut state: ResMut<State>) {
    if let State::Init { handle, .. } = &*state {
        if handle.is_finished() {
            let State::Init { handle, .. } = std::mem::replace(&mut *state, State::Uninit) else {
                unreachable!()
            };

            match handle.join() {
                Ok(r) => match r {
                    Ok(()) => warn!("driver thread exited successfully..."),
                    Err(e) => warn!("driver thread returned error: {e:?}"),
                },
                Err(_) => warn!("driver thread panicked"),
            }
        }
    }
}

#[derive(SystemParam)]
struct OutputPanel<'w, 's> {
    cmd: Commands<'w, 's>,
    config: ResMut<'w, Config>,
    state: ResMut<'w, State>,
    devices: Query<'w, 's, (Entity, &'static EInputDevice)>,
}

impl WidgetSystem for OutputPanel<'static, 'static> {
    type Input = ();

    fn system(world: &mut World, state: &mut SystemState<Self>, ui: &mut Ui, _: (), _: WidgetId) {
        let mut params = state.get_mut(world);

        ui.label(RichText::new("enet").strong());

        let (commands, devices) = match &mut *params.state {
            State::Uninit => {
                ui.text_edit_singleline(&mut params.config.ip);

                if ui.button("Start").clicked() {
                    params
                        .cmd
                        .add(|world: &mut World| world.run_system_once(start));
                }

                return;
            }
            State::Init {
                commands,
                current_ip,
                devices,
                ..
            } => {
                ui.label(format!("IP: {}", current_ip));

                (commands, devices)
            }
        };

        let mut selected = None;

        let mut i = 0;
        for &entity in devices.iter() {
            let Ok((_, device)) = params.devices.get(entity) else {
                continue;
            };

            ComboBox::new(
                Id::new("output/enet/device").with(i),
                format!("Device {}", i + 1),
            )
            .selected_text(device.info().name())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, Some((i, None)), "[None]");

                for (new_entity, new_device) in params.devices.iter() {
                    ui.selectable_value(
                        &mut selected,
                        Some((i, Some(new_entity))),
                        new_device.info().name(),
                    );
                }
            });

            i += 1;
        }

        if let Some((i, entity)) = selected.take() {
            let remove = devices[i];

            let _ = commands.send(cmd_remove_device(remove));

            if let Some(entity) = entity {
                if let Ok((_, device)) = params.devices.get(entity) {
                    let handle = device.handle();

                    let _ = commands.send(cmd_add_device(entity, handle));
                }
            }

            match entity {
                Some(entity) => devices[i] = entity,
                None => {
                    devices.remove(i);
                }
            }
        }

        let mut selected = None;

        ComboBox::new("output/enet/device_new", format!("Device {}", i + 1))
            .selected_text("[None]")
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, None, "[None]");

                for (new_entity, new_device) in params.devices.iter() {
                    ui.selectable_value(&mut selected, Some(new_entity), new_device.info().name());
                }
            });

        if let Some(entity) = selected {
            devices.push(entity);

            if let Ok((_, device)) = params.devices.get(entity) {
                let handle = device.handle();

                let _ = commands.send(cmd_add_device(entity, handle));
            }
        }
    }
}

fn cmd_add_device(entity: Entity, mut handle: DeviceHandle) -> DriverCommand {
    Box::new(move |driver| {
        handle.register_events(driver.listener.handle());
        let device = NetDevice::new(handle, entity.to_bits());
        driver.devices.entry(entity).or_default().push(device);
        Ok(())
    })
}

fn cmd_remove_device(entity: Entity) -> DriverCommand {
    Box::new(move |driver| {
        driver.devices.entry(entity).or_default().pop();
        Ok(())
    })
}
