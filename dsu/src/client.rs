use std::{
    io::ErrorKind, net::{SocketAddr, UdpSocket}, time::{Duration, Instant}
};

use anyhow::Result;
use log::{info, warn};

use crate::packet::{ControllerInfo, Get, GetControllerData, GetControllerInfo, Packet, Send, SendControllerData, BUFFER_SIZE};

const REQUEST_DURATION: Duration = Duration::from_secs(1);

pub struct Client {
    validate_packets: bool,
    id: u32,
    socket: UdpSocket,
    addr: SocketAddr,
    last_request: Instant,
    buf: [u8; BUFFER_SIZE],

    request: [bool; 4],
    info: [ControllerInfo; 4],
}

impl Client {
    pub fn new(validate_packets: bool, id: u32, addr: SocketAddr, timeout: Option<Duration>) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.set_read_timeout(timeout)?;

        Ok(Self {
            validate_packets,
            id,
            socket,
            addr,
            last_request: Instant::now(),
            buf: [0; BUFFER_SIZE],

            request: [false; 4],
            info: std::array::from_fn(|i| ControllerInfo::disconnected(i as u8)),
        })
    }

    pub fn info(&self) -> &[ControllerInfo; 4] {
        &self.info
    }

    pub fn set_request(&mut self, request: [bool; 4]) {
        self.request = request;
    }

    pub fn poll(&mut self) -> Vec<SendControllerData> {
        let mut out = Vec::new();

        loop {
            if Instant::now() - self.last_request > REQUEST_DURATION {
                let mut bytes = Vec::new();

                Packet::Get(Get::GetControllerInfo(GetControllerInfo::new(&[0, 1, 2, 3]).unwrap()))
                    .write(self.id, &mut bytes);

                match self.socket.send_to(&bytes, self.addr) {
                    Ok(_) => {}
                    Err(e) => warn!("error requesting controller info: {e:?}"),
                }

                for i in 0..self.request.len() {
                    if !self.request[i] { continue; }

                    Packet::Get(Get::GetControllerData(GetControllerData::new(Some(i as u8), None).unwrap()))
                        .write(self.id, &mut bytes);
                    match self.socket.send_to(&bytes, self.addr) {
                        Ok(_) => {}
                        Err(e) => warn!("error requesting controller data: {e:?}"),
                    }
                }

                self.last_request = Instant::now();
            }

            let (len, _) = match self.socket.recv_from(&mut self.buf) {
                Ok(ok) => ok,
                Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => break,
                Err(e) => {
                    warn!("error receiving data: {e:?}");
                    break;
                }
            };

            let bytes = &self.buf[..len];
            let (packet, _) = match Packet::parse(bytes, self.validate_packets) {
                Ok(ok) => ok,
                Err(e) => {
                    warn!("error parsing packet: {e:?}");
                    continue;
                }
            };

            match packet {
                Packet::Get(_) => info!("received client packet on client"),
                Packet::Send(Send::SendProtocolVersionInfo(_)) => {}
                Packet::Send(Send::SendControllerInfo(info)) => {
                    let index = info.info.slot as usize;
                    if index > 4 {
                        warn!("received info for slot > 4");
                        continue;
                    }
                    self.info[index] = info.info;
                }
                Packet::Send(Send::SendControllerData(data)) => {
                    out.push(data);
                }
            }
        }

        out
    }
}
