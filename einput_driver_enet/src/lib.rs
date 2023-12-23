mod driver;

use std::{
    thread::JoinHandle,
    time::{Duration, Instant},
};

use bevy::prelude::*;
use einput_core::{
    thread_command::{self, TCReceiver},
    DriverInfo,
};

use self::driver::Args;

pub struct EInputDriverENet;

impl Plugin for EInputDriverENet {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (receive_commands, restart));
    }
}

#[derive(Resource)]
struct DriverEntities {
    driver: Entity,
}

#[derive(Resource)]
struct DriverState {
    recv: TCReceiver<World>,
    handle: JoinHandle<anyhow::Result<()>>,
    last_restart: Instant,
}

fn setup(mut cmd: Commands) {
    let driver = cmd
        .spawn(DriverInfo {
            name: "enet".into(),
        })
        .id();

    let (send, recv) = thread_command::new();

    let handle = driver::start(Args { send });

    cmd.insert_resource(DriverEntities { driver });
    cmd.insert_resource(DriverState {
        recv,
        handle,
        last_restart: Instant::now(),
    });
}

fn restart(mut state: ResMut<DriverState>) {
    if state.handle.is_finished() && Instant::now() - state.last_restart > Duration::from_secs(5) {
        let (send, recv) = thread_command::new();

        let handle = driver::start(Args { send });

        let handle = std::mem::replace(&mut state.handle, handle);
        let _ = std::mem::replace(&mut state.recv, recv);

        state.last_restart = Instant::now();

        match handle.join() {
            Ok(r) => match r {
                Ok(()) => warn!("driver thread exited successfully..."),
                Err(e) => warn!("driver thread returned error: {e:?}"),
            },
            Err(_) => warn!("driver thread panicked"),
        }
    }
}

fn receive_commands(world: &mut World) {
    world.resource_scope::<DriverState, _>(|world, mut state| {
        state.recv.execute(world);
    });
}
