use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use bevy::{ecs::world::World, hierarchy::DespawnRecursiveExt, log::warn, utils::HashMap};
use einput_core::{
    core_device::DeviceOwner,
    thread_command::{TCSender, ThreadCommand},
    DeviceBundle,
};
use einput_device::DeviceInfo;
use einput_enet::PacketBuf;

use crate::DriverEntities;

const PACKET_SIZE: usize = 256;
const PORT: u16 = 29810;

pub struct Args {
    pub send: TCSender<World>,
}

pub fn start(args: Args) -> JoinHandle<Result<()>> {
    std::thread::spawn(move || driver(args))
}

fn driver(args: Args) -> Result<()> {
    let mut driver = Driver::new(args.send)?;

    loop {
        driver.update(Duration::from_millis(500))?;

        let now = Instant::now();
        let duration = Duration::from_secs(5);

        let mut to_remove = Vec::new();

        for (key, device) in &driver.devices {
            if now - device.last_update > duration {
                to_remove.push((*key, device.owner.id()));
            }
        }

        for (key, _) in &to_remove {
            driver.devices.remove(key);
        }

        if to_remove.len() > 0 {
            driver
                .send
                .send_nonblocking(ThreadCommand::nonblocking(move |world: &mut World| {
                    for (_, entity) in to_remove {
                        world.entity_mut(entity).despawn_recursive();
                    }
                }));
        }
    }
}

struct NetDevice {
    owner: DeviceOwner,
    last_update: Instant,
}

impl NetDevice {
    fn new(owner: DeviceOwner) -> Self {
        NetDevice {
            owner,
            last_update: Instant::now(),
        }
    }
}

struct Driver {
    packet: PacketBuf,
    devices: HashMap<DeviceIndex, NetDevice>,
    send: TCSender<World>,
    socket: UdpSocket,
}

impl Driver {
    fn new(send: TCSender<World>) -> Result<Self> {
        Ok(Driver {
            packet: PacketBuf::new(PACKET_SIZE),
            devices: HashMap::new(),
            send,
            socket: UdpSocket::bind(format!("0.0.0.0:{PORT}"))?,
        })
    }

    fn update(&mut self, timeout: Duration) -> Result<()> {
        let mut addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

        self.socket
            .set_read_timeout(Some(timeout))
            .context("error setting read timeout")?;

        let result: Result<(), std::io::Error> = self.packet.write(|bytes| {
            let (written, from_addr) = self.socket.recv_from(bytes)?;
            addr = from_addr;
            Ok(written)
        });

        match result {
            Ok(()) => {
                self.update_device(addr);
            }
            Err(err)
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut => {}
            Err(err) => return Err(err.into()),
        }

        Ok(())
    }

    fn update_device(&mut self, addr: SocketAddr) {
        let index = DeviceIndex {
            addr,
            id: self.packet.id(),
        };

        let device_ptr = match self.packet.device() {
            Ok(device) => device,
            Err(()) => {
                warn!("invalid packet");
                return;
            }
        };

        match self.devices.get_mut(&index) {
            Some(device) => {
                match device.owner.update(|device| device.update(device_ptr)) {
                    Ok(()) => {}
                    Err(()) => {
                        warn!("bad packet");
                    }
                }

                device.last_update = Instant::now();
            }
            None => {
                let device = device_ptr.to_owned();
                let info = DeviceInfo::default_from(
                    &device,
                    format!("ENet Device"),
                    format!("enet/{}/{}", addr, self.packet.id()),
                );

                let owner =
                    self.send
                        .send_blocking(ThreadCommand::blocking(|world: &mut World| {
                            let driver = world.resource::<DriverEntities>().driver;

                            let entity = world.spawn_empty().id();
                            let (bundle, owner) = DeviceBundle::new(entity, device, info, driver);
                            world.entity_mut(entity).insert(bundle);

                            owner
                        }));

                self.devices.insert(index, NetDevice::new(owner));
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct DeviceIndex {
    addr: SocketAddr,
    id: u64,
}
