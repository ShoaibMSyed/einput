use std::{collections::HashMap, io::ErrorKind, net::{SocketAddr, UdpSocket}, time::{Duration, Instant}};

use anyhow::{bail, Result};
use log::{debug, info, warn};

use crate::packet::{ControllerInfo, Get, Packet, Send, SendControllerData, SendControllerInfo, SendProtocolVersionInfo, BUFFER_SIZE};

pub const DEFAULT_PORT: u16 = 26760;

pub fn default_address() -> SocketAddr {
    "0.0.0.0:26760".parse().unwrap()
}

const TIMEOUT: Duration = Duration::from_secs(5);

pub struct Server {
    validate_packets: bool,
    id: u32,
    socket: UdpSocket,
    pub controllers: [SendControllerData; 4],
    clients: HashMap<u32, Client>,
    buf: [u8; BUFFER_SIZE],
}

impl Server {
    pub fn new(validate_packets: bool, id: u32, addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        
        Ok(Self {
            validate_packets,
            id,
            socket,
            controllers: std::array::from_fn(|i| SendControllerData::new(ControllerInfo::disconnected(i as u8))),
            clients: HashMap::new(),
            buf: [0; BUFFER_SIZE],
        })
    }

    pub fn remove_old_clients(&mut self) {
        self
            .clients
            .retain(|_, client| {
                let now = Instant::now();
                
                for last_request in &mut client.requesting {
                    if let Some(instant) = last_request {
                        if now - *instant > TIMEOUT {
                            *last_request = None;
                        }
                    }
                }

                client.requesting.iter().any(|r| r.is_some())
            });
    }

    pub fn receive(&mut self) -> Result<()> {
        let (len, addr) = match self.socket.recv_from(&mut self.buf) {
            Ok(ok) => ok,
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e.into())
        };

        let bytes = &self.buf[..len];
        let (packet, header) = Packet::parse(bytes, self.validate_packets)?;

        let client = self.clients.entry(header.id)
        .or_insert_with(move || {
            info!("client connected with address {addr}");
            Client {
                addr,
                requesting: [None; 4],
                packet: 0,
            }
        });

        match packet {
            Packet::Get(Get::GetProtocolVersionInfo) => {
                let mut bytes = Vec::new();
                Packet::Send(Send::SendProtocolVersionInfo(SendProtocolVersionInfo::default())).write(self.id, &mut bytes);
                self.socket.send_to(&bytes, addr)?;
            },
            Packet::Get(Get::GetControllerInfo(packet)) => {
                let mut bytes = Vec::new();

                for &slot in packet.slots() {
                    if slot > 4 {
                        warn!("client requested info for invalid slot {slot}");
                        continue;
                    }
                    
                    let info = self.controllers[slot as usize].info.clone();
                    Packet::Send(Send::SendControllerInfo(SendControllerInfo::new(info))).write(self.id, &mut bytes);
                    self.socket.send_to(&bytes, addr)?;
                }
            }
            Packet::Get(Get::GetControllerData(packet)) => {
                let macs = std::array::from_fn(|i| self.controllers[i].info.mac);
                let slots = packet.slots(macs);

                for i in 0..4 {
                    if slots[i] {
                        if client.requesting[i].is_none() {
                            debug!("client requested controller data for slot {i}");
                        }
                        
                        client.requesting[i] = Some(Instant::now());
                    }
                }
            }
            Packet::Send(_) => bail!("received packet from a server?"),
        }

        Ok(())
    }

    pub fn send(&mut self) {
        let mut bytes = Vec::new();

        for i in 0..4 {
            self.controllers[i].update_connected();
            let mut data = self.controllers[i].clone();

            for client in self.clients.values_mut() {
                if client.requesting[i].is_none() { continue; }

                data.packet = client.packet;
                Packet::Send(Send::SendControllerData(data.clone())).write(self.id, &mut bytes);
                client.packet += 1;

                match self.socket.send_to(&bytes, client.addr) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("error sending data to client: {e}")
                    }
                }
            }
        }
    }
}

struct Client {
    addr: SocketAddr,
    requesting: [Option<Instant>; 4],
    packet: u32,
}