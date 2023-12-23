mod driver;

use std::{
    sync::mpsc::Sender,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::Result;
use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use einput_core::core_device::{DeviceHandle, EInputDevice};
use einput_egui::{
    egui::{ComboBox, Id, RichText, Ui},
    widget::{DynamicWidget, WidgetId, WidgetSystem},
    OutputWidget,
};

use self::driver::{Args, DriverCommand, UDevice};

pub struct EInputOutputUInput;

impl Plugin for EInputOutputUInput {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init)
            .add_systems(PreUpdate, check_state);
    }
}

#[derive(Resource)]
struct State {
    handle: JoinHandle<Result<()>>,
    commands: Sender<DriverCommand>,
    last_init: Instant,

    devices: Vec<Entity>,
}

impl State {
    fn new(handle: JoinHandle<Result<()>>, commands: Sender<DriverCommand>) -> Self {
        State {
            handle,
            commands,
            last_init: Instant::now(),
            devices: Vec::new(),
        }
    }
}

fn init(mut cmd: Commands) {
    cmd.spawn(OutputWidget(DynamicWidget::new::<OutputPanel>()));

    let (send, recv) = std::sync::mpsc::channel();

    let handle = driver::start(Args { commands: recv });

    cmd.insert_resource(State::new(handle, send));
}

fn check_state(mut cmd: Commands, state: Res<State>) {
    if Instant::now() - state.last_init >= Duration::from_secs(3) {
        if state.handle.is_finished() {
            cmd.add(restart);
        }
    }
}

fn restart(world: &mut World) {
    let Some(state) = world.remove_resource::<State>() else {
        return;
    };

    if !state.handle.is_finished() {
        return;
    }

    match state.handle.join() {
        Ok(r) => match r {
            Ok(()) => warn!("driver thread exited successfully..."),
            Err(e) => warn!("driver thread returned error: {e:?}"),
        },
        Err(_) => warn!("driver thread panicked"),
    }

    info!("restarting driver");

    let (send, recv) = std::sync::mpsc::channel();

    let handle = driver::start(Args { commands: recv });

    world.insert_resource(State::new(handle, send));
}

#[derive(SystemParam)]
struct OutputPanel<'w, 's> {
    state: ResMut<'w, State>,
    devices: Query<'w, 's, (Entity, &'static EInputDevice)>,
}

impl WidgetSystem for OutputPanel<'static, 'static> {
    type Input = ();

    fn system(world: &mut World, state: &mut SystemState<Self>, ui: &mut Ui, _: (), _: WidgetId) {
        let mut params = state.get_mut(world);

        ui.label(RichText::new("uinput").strong());

        let mut selected = None;

        let mut i = 0;
        for &entity in &params.state.devices {
            let Ok((_, device)) = params.devices.get(entity) else {
                continue;
            };

            ComboBox::new(Id::new("eou/device").with(i), format!("Device {}", i + 1))
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
            let remove = params.state.devices[i];

            let _ = params.state.commands.send(cmd_remove_device(remove));

            if let Some(entity) = entity {
                if let Ok((_, device)) = params.devices.get(entity) {
                    let handle = device.handle();

                    let _ = params.state.commands.send(cmd_add_device(entity, handle));
                }
            }

            match entity {
                Some(entity) => params.state.devices[i] = entity,
                None => {
                    params.state.devices.remove(i);
                }
            }
        }

        let mut selected = None;

        ComboBox::new("eou/device_new", format!("Device {}", i + 1))
            .selected_text("[None]")
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, None, "[None]");

                for (new_entity, new_device) in params.devices.iter() {
                    ui.selectable_value(&mut selected, Some(new_entity), new_device.info().name());
                }
            });

        if let Some(entity) = selected {
            params.state.devices.push(entity);

            if let Ok((_, device)) = params.devices.get(entity) {
                let handle = device.handle();

                let _ = params.state.commands.send(cmd_add_device(entity, handle));
            }
        }
    }
}

fn cmd_add_device(entity: Entity, mut handle: DeviceHandle) -> DriverCommand {
    Box::new(move |driver| {
        handle.register_events(driver.listener.handle());
        let device = UDevice::new(handle, driver.uinput())?;
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
