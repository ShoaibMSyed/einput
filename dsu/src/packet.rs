use std::fmt::Display;

use crc::Crc;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub const BUFFER_SIZE: usize = size_of::<Header>() + size_of::<SendControllerData>();

const MAGIC_SERVER: [u8; 4] = *b"DSUS";
const MAGIC_CLIENT: [u8; 4] = *b"DSUC";

const PROTOCOL: u16 = 1001;

const KIND_PROTOCOL: u32 = 0x100000;
const KIND_INFO: u32 = 0x100001;
const KIND_DATA: u32 = 0x100002;

#[derive(Clone, Debug)]
pub enum Error {
    Header(Box<Error>),
    Packet {
        kind: &'static str,
        error: Box<Error>,
    },
    SizeError,
    InvalidField {
        field: &'static str,
        got: String,
        expected: String,
    },
}

impl Error {
    fn header(self) -> Self {
        Self::Header(Box::new(self))
    }

    fn packet(self, kind: &'static str) -> Self {
        Self::Packet {
            kind,
            error: Box::new(self),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Header(error) => write!(f, "error parsing header: {error}"),
            Error::Packet { kind, error } => write!(f, "error parsing packet {kind}: {error}"),
            Error::SizeError => write!(f, "invalid packet size"),
            Error::InvalidField { field, got, expected } => {
                write!(f, "invalid field '{field}'")?;
                if !expected.is_empty() || !got.is_empty() {
                    write!(f, ": ")?;
                }
                if !expected.is_empty() {
                    write!(f, " expected {expected}")?;
                    if !got.is_empty() {
                        write!(f, ", ")?;
                    }
                }
                if !got.is_empty() {
                    write!(f, "got {got}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum Packet {
    Get(Get),
    Send(Send),
}

impl Packet {
    pub fn parse(full_bytes: &[u8], validate: bool) -> Result<(Self, Header), Error> {
        let (header, bytes) = match Header::read_from_prefix(full_bytes) {
            Ok(ok) => ok,
            Err(_) => {
                return Err(Error::SizeError.header());
            }
        };

        let is_get = match header.magic {
            MAGIC_CLIENT => true,
            MAGIC_SERVER => false,
            magic => {
                return Err(Error::InvalidField {
                    field: "magic",
                    got: format!("{:#X}", u32::from_le_bytes(magic)),
                    expected: format!(
                        "{:#X} or {:#X}",
                        u32::from_le_bytes(MAGIC_CLIENT),
                        u32::from_le_bytes(MAGIC_SERVER)
                    ),
                }
                .header());
            }
        };

        if header.protocol != PROTOCOL {
            return Err(Error::InvalidField {
                field: "protocol",
                got: format!("{}", header.protocol),
                expected: format!("{}", PROTOCOL),
            }
            .header());
        }

        if (header.length as usize) < bytes.len() + 4 {
            return Err(Error::SizeError);
        }

        let packet = match is_get {
            true => Packet::Get(match header.kind {
                KIND_PROTOCOL => Get::GetProtocolVersionInfo,
                KIND_INFO => Get::GetControllerInfo(
                    GetControllerInfo::read(bytes).map_err(|e| e.packet("GetControllerInfo"))?,
                ),
                KIND_DATA => Get::GetControllerData(
                    GetControllerData::read_from_prefix(bytes)
                        .map_err(|_| Error::SizeError.packet("GetControllerData"))?
                        .0,
                ),
                kind => {
                    return Err(Error::InvalidField {
                        field: "kind",
                        got: format!("{kind:X}"),
                        expected: String::new(),
                    });
                }
            }),
            false => Packet::Send(match header.kind {
                KIND_PROTOCOL => Send::SendProtocolVersionInfo(
                    SendProtocolVersionInfo::read_from_prefix(bytes)
                        .map_err(|_| Error::SizeError.packet("SendProtocolVersionInfo"))?
                        .0,
                ),
                KIND_INFO => Send::SendControllerInfo(
                    SendControllerInfo::read_from_prefix(bytes)
                        .map_err(|_| Error::SizeError.packet("SendControllerInfo"))?
                        .0,
                ),
                KIND_DATA => Send::SendControllerData(
                    SendControllerData::read_from_prefix(bytes)
                        .map_err(|_| Error::SizeError.packet("SendControllerData"))?
                        .0,
                ),
                kind => {
                    return Err(Error::InvalidField {
                        field: "kind",
                        got: format!("{kind:X}"),
                        expected: String::new(),
                    });
                }
            }),
        };

        if validate {
            let to_check = header.length as usize + packet.length();

            const CRC_OFFSET: usize = 8;
    
            let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
            let mut digest = crc.digest();
            digest.update(&full_bytes[..CRC_OFFSET]);
            digest.update(&[0; 4]);
            digest.update(&full_bytes[CRC_OFFSET + 4..to_check]);
            
            let expected_crc = digest.finalize();
    
            if header.crc != expected_crc {
                return Err(Error::InvalidField {
                    field: "crc",
                    got: format!("{:#X}", header.crc),
                    expected: format!("{expected_crc:#X}"),
                });
            }
        }

        Ok((packet, header))
    }

    pub fn write(&self, id: u32, bytes: &mut Vec<u8>) {
        bytes.clear();

        let magic = match self {
            Self::Get(_) => MAGIC_CLIENT,
            Self::Send(_) => MAGIC_SERVER,
        };
        let length = (self.length() + 4) as u16;

        let mut header = Header {
            magic,
            protocol: PROTOCOL,
            length,
            crc: 0,
            id,
            kind: self.kind(),
        };
        bytes.extend(header.as_bytes());

        match self {
            Self::Get(Get::GetProtocolVersionInfo) => {}
            Self::Get(Get::GetControllerInfo(packet)) => packet.write(bytes),
            Self::Get(Get::GetControllerData(packet)) => bytes.extend_from_slice(packet.as_bytes()),
            Self::Send(Send::SendProtocolVersionInfo(packet)) => {
                bytes.extend_from_slice(packet.as_bytes())
            }
            Self::Send(Send::SendControllerInfo(packet)) => {
                bytes.extend_from_slice(packet.as_bytes())
            }
            Self::Send(Send::SendControllerData(packet)) => {
                bytes.extend_from_slice(packet.as_bytes())
            }
        }

        let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        header.crc = crc.checksum(&bytes);

        if header.crc == 0xe6b4159c {
            if matches!(self, Self::Send(Send::SendControllerData(_))) {
                // info!("data, {}", header.length);
            }
        }

        bytes[..size_of::<Header>()].copy_from_slice(header.as_bytes());
    }

    fn length(&self) -> usize {
        match self {
            Self::Get(Get::GetProtocolVersionInfo) => 0,
            Self::Get(Get::GetControllerInfo(packet)) => packet.length(),
            Self::Get(Get::GetControllerData(_)) => size_of::<GetControllerData>(),
            Self::Send(Send::SendProtocolVersionInfo(_)) => size_of::<SendProtocolVersionInfo>(),
            Self::Send(Send::SendControllerInfo(_)) => size_of::<SendControllerInfo>(),
            Self::Send(Send::SendControllerData(_)) => size_of::<SendControllerData>(),
        }
    }

    fn kind(&self) -> u32 {
        match self {
            Self::Get(Get::GetProtocolVersionInfo)
            | Self::Send(Send::SendProtocolVersionInfo(_)) => KIND_PROTOCOL,
            Self::Get(Get::GetControllerInfo(_)) | Self::Send(Send::SendControllerInfo(_)) => {
                KIND_INFO
            }
            Self::Get(Get::GetControllerData(_)) | Self::Send(Send::SendControllerData(_)) => {
                KIND_DATA
            }
        }
    }
}

#[derive(Debug)]
pub enum Get {
    GetProtocolVersionInfo,
    GetControllerInfo(GetControllerInfo),
    GetControllerData(GetControllerData),
}

#[derive(Debug)]
pub enum Send {
    SendProtocolVersionInfo(SendProtocolVersionInfo),
    SendControllerInfo(SendControllerInfo),
    SendControllerData(SendControllerData),
}

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Header {
    magic: [u8; 4],
    protocol: u16,
    length: u16,
    crc: u32,
    pub(crate) id: u32,
    kind: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct SendProtocolVersionInfo {
    protocol: u16,
}

impl Default for SendProtocolVersionInfo {
    fn default() -> Self {
        Self { protocol: PROTOCOL }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct ControllerInfo {
    pub slot: u8,
    pub state: u8,
    pub model: u8,
    pub connection: u8,
    pub mac: [u8; 6],
    pub battery: u8,
}

impl ControllerInfo {
    pub const STATE_DISCONNECTED: u8 = 0;
    pub const STATE_RESERVED: u8 = 1;
    pub const STATE_CONNECTED: u8 = 2;

    pub const MODEL_NA: u8 = 0;
    pub const MODEL_NO_GYRO: u8 = 1;
    pub const MODEL_FULL_GYRO: u8 = 2;

    pub const CONNECTION_NA: u8 = 0;
    pub const CONNECTION_USB: u8 = 1;
    pub const CONNECTION_BLUETOOTH: u8 = 2;

    pub const BATTERY_NA: u8 = 0x00;
    pub const BATTERY_DYING: u8 = 0x01;
    pub const BATTEY_LOW: u8 = 0x02;
    pub const BATTERY_MEDIUM: u8 = 0x03;
    pub const BATTERY_HIGH: u8 = 0x04;
    pub const BATTERY_FULL: u8 = 0x05;
    pub const BATTERY_CHARGING: u8 = 0xEE;
    pub const BATTERY_CHARGED: u8 = 0xEF;

    pub fn disconnected(slot: u8) -> Self {
        Self {
            slot,
            state: Self::STATE_DISCONNECTED,
            model: Self::MODEL_NA,
            connection: Self::CONNECTION_NA,
            mac: [0; 6],
            battery: Self::BATTERY_NA,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GetControllerInfo {
    ports: u32,
    slots: [u8; 4],
}

impl GetControllerInfo {
    pub fn new(slots: &[u8]) -> Result<Self, Error> {
        let mut this = Self {
            ports: 0,
            slots: [0; 4],
        };

        if slots.len() > 4 {
            return Err(Error::InvalidField {
                field: "slots",
                got: format!("{}", slots.len()),
                expected: format!("0, 1, 2, or 3"),
            });
        }

        this.ports = slots.len() as u32;

        for i in 0..slots.len() {
            if slots[i] > 4 {
                return Err(Error::InvalidField {
                    field: "slots",
                    got: format!("{}", slots[i]),
                    expected: format!("0, 1, 2, or 3"),
                });
            }
            this.slots[i] = slots[i];
        }

        Ok(this)
    }

    pub fn slots(&self) -> &[u8] {
        &self.slots[..self.ports as usize]
    }

    fn read(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 4 {
            return Err(Error::SizeError);
        }
        let ports = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let bytes = &bytes[4..];
        if ports > 4 {
            return Err(Error::InvalidField {
                field: "ports",
                got: format!("ports"),
                expected: format!("0, 1, 2, or 3"),
            });
        }
        if bytes.len() < ports as usize {
            return Err(Error::SizeError);
        }
        let mut slots = [0; 4];
        for i in 0..ports as usize {
            slots[i] = bytes[i];
        }
        Ok(Self { ports, slots })
    }

    fn write(&self, bytes: &mut Vec<u8>) {
        let ports = self.ports.to_le_bytes();
        bytes.extend_from_slice(&ports);
        for i in 0..self.ports as usize {
            bytes.push(self.slots[i]);
        }
    }

    fn length(&self) -> usize {
        4 + self.ports as usize
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct SendControllerInfo {
    pub info: ControllerInfo,
    null: u8,
}

impl SendControllerInfo {
    pub fn new(info: ControllerInfo) -> Self {
        Self { info, null: 0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct GetControllerData {
    registration: u8,
    slot: u8,
    mac: [u8; 6],
}

impl GetControllerData {
    pub fn all() -> Self {
        Self {
            registration: 0,
            slot: 0,
            mac: [0; 6],
        }
    }

    pub fn new(slot: Option<u8>, mac: Option<[u8; 6]>) -> Result<Self, Error> {
        let mut this = Self::all();

        if let Some(slot) = slot {
            if slot > 4 {
                return Err(Error::InvalidField {
                    field: "slot",
                    got: format!("{}", slot),
                    expected: format!("0, 1, 2, or 3"),
                });
            }
            this.registration |= 0b1;
            this.slot = slot;
        }

        if let Some(mac) = mac {
            this.registration |= 0b01;
            this.mac = mac;
        }

        Ok(this)
    }

    pub fn slots(&self, macs: [[u8; 6]; 4]) -> [bool; 4] {
        if self.registration == 0 {
            return [true; 4];
        }

        let mut slots = [false; 4];

        if self.registration & 0b1 != 0 {
            if self.slot < 4 {
                slots[self.slot as usize] = true;
            }
        }

        if self.registration & 0b01 != 0 {
            for i in 0..4 {
                if self.mac == macs[i] {
                    slots[i] = true;
                }
            }
        }

        slots
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct SendControllerData {
    pub info: ControllerInfo,
    pub connected: u8,
    pub packet: u32,
    pub buttons: u16,
    pub home: u8,
    pub touch: u8,
    pub lsx: u8,
    pub lsy: u8,
    pub rsx: u8,
    pub rsy: u8,
    pub left: u8,
    pub down: u8,
    pub right: u8,
    pub up: u8,
    pub y: u8,
    pub b: u8,
    pub a: u8,
    pub x: u8,
    pub r1: u8,
    pub l1: u8,
    pub r2: u8,
    pub l2: u8,
    pub touch1: Touch,
    pub touch2: Touch,
    pub timestamp: u64,
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub gyro_pitch: f32,
    pub gyro_yaw: f32,
    pub gyro_roll: f32,
}

impl SendControllerData {
    pub fn new(info: ControllerInfo) -> Self {
        let connected = info.state != ControllerInfo::STATE_DISCONNECTED;
        Self {
            info,
            connected: connected as u8,
            packet: 0,
            buttons: 0,
            home: 0,
            touch: 0,
            lsx: 127,
            lsy: 127,
            rsx: 127,
            rsy: 127,
            left: 0,
            down: 0,
            right: 0,
            up: 0,
            y: 0,
            b: 0,
            a: 0,
            x: 0,
            r1: 0,
            l1: 0,
            r2: 0,
            l2: 0,
            touch1: Touch::default(),
            touch2: Touch::default(),
            timestamp: 0,
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 0.0,
            gyro_pitch: 0.0,
            gyro_yaw: 0.0,
            gyro_roll: 0.0,
        }
    }

    pub fn update_connected(&mut self) {
        self.connected = (self.info.state != ControllerInfo::STATE_DISCONNECTED) as u8;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Touch {
    pub active: u8,
    pub id: u8,
    pub x: u16,
    pub y: u16,
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Share = 1 << 0,
    L3 = 1 << 1,
    R3 = 1 << 2,
    Options = 1 << 3,
    Up = 1 << 4,
    Right = 1 << 5,
    Down = 1 << 6,
    Left = 1 << 7,
    L2 = 1 << 8,
    R2 = 1 << 9,
    L1 = 1 << 10,
    R1 = 1 << 11,
    X = 1 << 12,
    A = 1 << 13,
    B = 1 << 14,
    Y = 1 << 15,
}
