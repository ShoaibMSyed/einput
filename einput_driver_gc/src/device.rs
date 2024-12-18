use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bytemuck::{Pod, Zeroable};
use einput_core::{device::DeviceOwner, EInput};
use einput_device::{input::buttons::{Button, Buttons}, DeviceInfo, DeviceInputInfo, DeviceKind, DeviceOutputInfo};
use einput_util::axis::{Stick, StickAxis};
use log::warn;

use crate::UsbDeviceHandle;

const EP_IN: u8 = 0x81;
const EP_OUT: u8 = 0x02;

const INITIALIZE: [u8; 1] = [0x13];

const STATE_NORMAL: u8 = 0x10;
const STATE_WAVEBIRD: u8 = 0x20;

pub struct DeviceDriver {
    einput: EInput,
    device: UsbDeviceHandle,
    number: usize,
    serial: Option<String>,

    packet: InputPacket,
    controllers: [Option<Controller>; 4],
}

impl DeviceDriver {
    pub fn new(einput: EInput, device: UsbDeviceHandle, number: usize, serial: Option<String>) -> Self {
        DeviceDriver {
            einput,
            device,
            number,
            serial,
            packet: InputPacket::zeroed(),
            controllers: [None, None, None, None],
        }
    }

    pub fn run(mut self) -> Result<()> {
        self.initialize()?;

        loop {
            self.read()?;
        }
    }

    fn initialize(&mut self) -> Result<()> {
        if self
            .device
            .write_interrupt(EP_OUT, &INITIALIZE, Duration::from_secs(5))
            .context("error initializing device: check for correct driver")?
            != 1
        {
            return Err(anyhow!("error initializing device"));
        }

        Ok(())
    }

    fn read(&mut self) -> Result<()> {
        let read = match self.device.read_interrupt(
            EP_IN,
            bytemuck::bytes_of_mut(&mut self.packet),
            Duration::from_millis(16),
        ) {
            Ok(read) => read,
            Err(rusb::Error::Timeout) => return Ok(()),
            Err(e) => anyhow::bail!("error reading: {e}"),
        };

        if read != std::mem::size_of::<InputPacket>() {
            return Ok(());
        }

        if self.packet.id != 0x21 {
            warn!("invalid packet data");
            return Ok(());
        }

        for i in 0..4 {
            if self.packet.cons[i].is_connected() {
                match &mut self.controllers[i] {
                    None => {
                        let mut con = Controller::new(&self.einput, self.number, i, self.serial.as_ref())?;
                        con.update(&self.packet.cons[i]);
                        self.controllers[i] = Some(con);
                    }
                    Some(con) => {
                        con.update(&self.packet.cons[i]);
                    }
                }
            } else {
                self.controllers[i] = None;
            }
        }

        Ok(())
    }
}

struct Controller {
    device: DeviceOwner,
}

impl Controller {
    fn new(einput: &EInput, adapter: usize, controller: usize, serial: Option<&String>) -> Result<Self> {
        let id = match &serial {
            Some(serial) => format!("gc{serial}::{controller}"),
            None => format!("gc{adapter}::{controller}"),
        };

        let name_suffix = match adapter {
            0 => String::new(),
            _ => format!(" (Adapter {})", adapter + 1),
        };
        
        let info = DeviceInfo::new(
            format!("GameCube Controller {}{}", controller + 1, name_suffix),
            "GameCube Controller".into(),
            id.into(),
            DeviceKind::Gamepad,
        )
            .with_input(DeviceInputInfo {
                buttons: Buttons::ABXY | Buttons::DPAD | Buttons::TRIGGERS | Button::R1 | Button::Start,
                sticks: true,
                triggers: true,
                ..Default::default()
            })
            .with_output(DeviceOutputInfo {
                rumble_motors: 1,
            });
        
        let device = einput.create_device(info)
            .context("device already exists")?;

        Ok(Self { device })
    }

    fn update(&mut self, packet: &ControllerInputPacket) {
        self.device.update(|input| {
            *input.buttons_mut().unwrap() = packet.buttons();

            let sticks = input.sticks_mut().unwrap();
            sticks.left = Stick::from_xy(packet.lsx, packet.lsy.invert());
            sticks.right = Stick::from_xy(packet.rsx, packet.rsy.invert());
            
            let triggers = input.triggers_mut().unwrap();
            triggers.l2 = packet.lt.into();
            triggers.r2 = packet.rt.into();
        });
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InputPacket {
    id: u8,
    cons: [ControllerInputPacket; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ControllerInputPacket {
    state: u8,
    buttons: [u8; 2],
    lsx: u8,
    lsy: u8,
    rsx: u8,
    rsy: u8,
    lt: u8,
    rt: u8,
}

impl ControllerInputPacket {
    const BUTTONS: [Button; 12] = [
        Button::A,
        Button::B,
        Button::X,
        Button::Y,
        Button::Left,
        Button::Right,
        Button::Down,
        Button::Up,
        Button::Start,
        Button::R1,
        Button::R2,
        Button::L2,
    ];

    fn buttons(&self) -> Buttons {
        let mut buttons = Buttons::default();

        let bits = u16::from_le_bytes(self.buttons);

        for i in 0..12 {
            if bits & (1 << i) != 0 {
                buttons |= Self::BUTTONS[i];
            }
        }

        buttons
    }

    fn is_connected(&self) -> bool {
        matches!(self.state, STATE_NORMAL | STATE_WAVEBIRD)
    }
}
