use std::{ops::ControlFlow, thread::JoinHandle, time::Duration};

use anyhow::{Context, Result};
use einput_components::{
    gamepad::{Button, Gamepad},
    map_to_buttons,
};
use einput_core::{core_device::DeviceOwner, device::ComponentId};
use einput_driver_usb::UsbDeviceHandle;
use einput_util::StickAxis;

const EP_IN: u8 = 0x81;

pub struct Args {
    pub handle: UsbDeviceHandle,
    pub owner: DeviceOwner,
}

pub fn start(args: Args) -> JoinHandle<Result<()>> {
    std::thread::spawn(move || driver(args))
}

fn driver(args: Args) -> Result<()> {
    let mut driver = Driver::new(args.handle, args.owner);

    driver.init()?;

    loop {
        match driver.update(Duration::from_millis(500))? {
            ControlFlow::Continue(()) => continue,
            ControlFlow::Break(()) => break,
        }
    }

    Ok(())
}

struct Driver {
    usb_handle: UsbDeviceHandle,
    owner: DeviceOwner,
    packet: [u8; 8],

    gamepad: ComponentId<Gamepad>,
}

impl Driver {
    fn new(usb_handle: UsbDeviceHandle, owner: DeviceOwner) -> Self {
        Driver {
            usb_handle,
            owner,
            packet: [0; 8],

            gamepad: Default::default(),
        }
    }

    fn init(&mut self) -> Result<()> {
        match self.usb_handle.set_auto_detach_kernel_driver(true) {
            Ok(()) => {}
            Err(rusb::Error::NotSupported) => {}
            Err(err) => {
                Err(err).context("failed to auto-detach kernel drivers")?;
            }
        }

        let iface = self
            .usb_handle
            .device()
            .active_config_descriptor()
            .context("failed to get active config descriptor")?
            .interfaces()
            .flat_map(|interface| interface.descriptors())
            .find(|desc| {
                desc.class_code() == 3 && desc.sub_class_code() == 0 && desc.protocol_code() == 0
            })
            .context("failed to find interface descriptor")?
            .interface_number();

        self.usb_handle
            .claim_interface(iface)
            .context("failed to claim interface")?;

        Ok(())
    }

    fn update(&mut self, timeout: Duration) -> Result<ControlFlow<()>> {
        let size = match self
            .usb_handle
            .read_interrupt(EP_IN, &mut self.packet, timeout)
        {
            Ok(size) => size,
            Err(rusb::Error::Timeout) => return Ok(ControlFlow::Continue(())),
            Err(rusb::Error::NoDevice) => return Ok(ControlFlow::Break(())),
            Err(err) => return Err(err.into()),
        };

        if size == 8 {
            self.update_device()?;
        }

        Ok(ControlFlow::Continue(()))
    }

    fn update_device(&mut self) -> Result<()> {
        let buttons = u16::from_le_bytes([self.packet[0], self.packet[1]]);
        let mut buttons = map_to_buttons! {
            buttons =>
            0 = Button::Y,
            1 = Button::B,
            2 = Button::A,
            3 = Button::X,
            4 = Button::L1,
            5 = Button::R1,
            6 = Button::L2,
            7 = Button::R2,
            8 = Button::Select,
            9 = Button::Start,
            10 = Button::LStick,
            11 = Button::RStick,
            12 = Button::Home,
            13 = Button::Capture,
        };

        let dpad = self.packet[2];

        if matches!(dpad, 7 | 0 | 1) {
            buttons |= Button::Up;
        }

        if matches!(dpad, 1 | 2 | 3) {
            buttons |= Button::Right;
        }

        if matches!(dpad, 3 | 4 | 5) {
            buttons |= Button::Down;
        }

        if matches!(dpad, 5 | 6 | 7) {
            buttons |= Button::Left;
        }

        let lx = self.packet[3];
        let ly = self.packet[4];
        let rx = self.packet[5];
        let ry = self.packet[6];

        self.owner.update(|device| {
            let con = self.gamepad.get_mut(device, 0).unwrap();
            con.buttons = buttons;
            con.left_stick.x = lx;
            con.left_stick.y = ly.invert();
            con.right_stick.x = rx;
            con.right_stick.y = ry.invert();
            con.set_shoulders_from_buttons();
            con.set_triggers_from_buttons();
        });

        Ok(())
    }
}
