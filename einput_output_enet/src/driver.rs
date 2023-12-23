use std::{net::UdpSocket, sync::mpsc::Receiver, thread::JoinHandle, time::Duration};

use anyhow::{Context, Result};
use bevy::{ecs::entity::Entity, log::info, utils::HashMap};
use einput_core::{core_device::DeviceHandle, sync::EventListener};
use einput_enet::PacketWriter;

pub struct Args {
    pub commands: Receiver<DriverCommand>,
    pub ip: String,
}

pub struct Driver {
    pub listener: EventListener,
    pub devices: HashMap<Entity, Vec<NetDevice>>,
    pub socket: UdpSocket,
}

impl Driver {}

pub type DriverCommand = Box<dyn FnOnce(&mut Driver) -> Result<()> + Send>;

pub struct NetDevice {
    handle: DeviceHandle,
    packet: PacketWriter,
}

impl NetDevice {
    pub fn new(handle: DeviceHandle, id: u64) -> NetDevice {
        let packet = PacketWriter::new(id, &handle.device());
        NetDevice { handle, packet }
    }

    fn update(&mut self, socket: &UdpSocket) -> Result<()> {
        self.packet.update(&self.handle.device());

        let _ = socket.send(self.packet.bytes());

        Ok(())
    }
}

pub fn start(args: Args) -> JoinHandle<Result<()>> {
    std::thread::spawn(move || driver(args))
}

fn driver(args: Args) -> Result<()> {
    let mut driver = Driver {
        listener: EventListener::new(),
        devices: HashMap::new(),
        socket: UdpSocket::bind("0.0.0.0:0").context("error binding socket")?,
    };

    info!("driver started");

    driver
        .socket
        .connect(&args.ip)
        .context("error connecting socket")?;

    loop {
        for entity in driver.listener.listen(Duration::from_millis(200)) {
            let Some(devices) = driver.devices.get_mut(&entity) else {
                continue;
            };

            for dev in devices {
                dev.update(&driver.socket)?;
            }
        }

        for cmd in args.commands.try_iter() {
            cmd(&mut driver).context("command error")?;
        }
    }
}
