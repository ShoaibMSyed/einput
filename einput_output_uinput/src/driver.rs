use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
    thread::JoinHandle,
    time::Duration,
};

use anyhow::{Context, Result};
use bevy::{
    ecs::entity::Entity,
    log::{info, warn},
    prelude::default,
    utils::HashMap,
};
use einput_components::gamepad::{Button, Gamepad};
use einput_core::{core_device::DeviceHandle, device::ComponentId, sync::EventListener};
use input_linux::{AbsoluteAxis, AbsoluteInfo, AbsoluteInfoSetup, EventKind, Key, UInputHandle};

pub struct Args {
    pub commands: Receiver<DriverCommand>,
}

pub struct Driver {
    uinput: PathBuf,
    pub listener: EventListener,
    pub devices: HashMap<Entity, Vec<UDevice>>,
}

impl Driver {
    pub fn uinput(&self) -> &Path {
        &self.uinput
    }
}

pub type DriverCommand = Box<dyn FnOnce(&mut Driver) -> Result<()> + Send>;

pub struct UDevice {
    handle: DeviceHandle,
    device: UInputHandle<File>,

    gamepad: ComponentId<Gamepad>,
}

impl UDevice {
    pub fn new(handle: DeviceHandle, uinput: &Path) -> Result<UDevice> {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(uinput)
            .context("error opening uinput device")?;

        let device = UInputHandle::new(device);

        device.set_evbit(EventKind::Key)?;
        for key in [
            Key::ButtonNorth,
            Key::ButtonSouth,
            Key::ButtonEast,
            Key::ButtonWest,
            Key::ButtonDpadDown,
            Key::ButtonDpadLeft,
            Key::ButtonDpadRight,
            Key::ButtonDpadUp,
            Key::ButtonStart,
            Key::ButtonSelect,
            Key::ButtonTL,
            Key::ButtonTR,
            Key::ButtonTL2,
            Key::ButtonTR2,
            Key::ButtonThumbl,
            Key::ButtonThumbr,
            Key::ButtonMode,
        ] {
            device.set_keybit(key)?;
        }

        device.set_evbit(EventKind::Absolute)?;
        for axis in [
            AbsoluteAxis::X,
            AbsoluteAxis::Y,
            AbsoluteAxis::RX,
            AbsoluteAxis::RY,
            AbsoluteAxis::Z,
            AbsoluteAxis::RZ,
        ] {
            device.set_absbit(axis)?;
        }

        const DEFAULT_INFO: AbsoluteInfo = AbsoluteInfo {
            value: 0,
            minimum: 0,
            maximum: 255,
            fuzz: 0,
            flat: 0,
            resolution: 0,
        };

        device
            .create(
                &input_linux::InputId::default(),
                handle.info().name().as_bytes(),
                0,
                &[
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::X,
                        info: DEFAULT_INFO,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::Y,
                        info: DEFAULT_INFO,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::RX,
                        info: DEFAULT_INFO,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::RY,
                        info: DEFAULT_INFO,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::Z,
                        info: DEFAULT_INFO,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::RZ,
                        info: DEFAULT_INFO,
                    },
                ],
            )
            .context("error creating uinput device")?;

        Ok(UDevice {
            handle,
            device,
            gamepad: default(),
        })
    }

    fn update(&self) -> Result<()> {
        use input_linux::sys as ils;

        let device = self.handle.device();
        let Some(data) = self.gamepad.get(&device, 0) else {
            return Ok(());
        };

        macro_rules! make_events {
            (
                buttons { $($btnfrom:expr => $btnto:expr),* $(,)? }
                analogs { $($afrom:expr => $ato:expr),* $(,)? }
            ) => {
                [
                    $(ils::input_event {
                        time: ils::timeval { tv_sec: 0, tv_usec: 0 },
                        type_: ils::EV_KEY as _,
                        code: $btnto as _,
                        value: data.buttons.pressed($btnfrom) as _,
                    },)*
                    $(ils::input_event {
                        time: ils::timeval { tv_sec: 0, tv_usec: 0 },
                        type_: ils::EV_ABS as _,
                        code: $ato as _,
                        value: $afrom as _,
                    },)*
                    ils::input_event {
                        time: ils::timeval { tv_sec: 0, tv_usec: 0 },
                        type_: ils::EV_SYN as _,
                        code: ils::SYN_REPORT as _,
                        value: 0,
                    }
                ]
            };
        }

        let events = make_events! {
            buttons {
                Button::A      => ils::BTN_SOUTH,
                Button::B      => ils::BTN_EAST,
                Button::X      => ils::BTN_WEST,
                Button::Y      => ils::BTN_NORTH,
                Button::Up     => ils::BTN_DPAD_UP,
                Button::Down   => ils::BTN_DPAD_DOWN,
                Button::Left   => ils::BTN_DPAD_LEFT,
                Button::Right  => ils::BTN_DPAD_RIGHT,
                Button::Start  => ils::BTN_START,
                Button::Select => ils::BTN_SELECT,
                Button::L1     => ils::BTN_TL,
                Button::R1     => ils::BTN_TR,
                Button::L2     => ils::BTN_TL2,
                Button::R2     => ils::BTN_TR2,
                Button::LStick => ils::BTN_THUMBL,
                Button::RStick => ils::BTN_THUMBR,
                Button::Home   => ils::BTN_MODE,
            }
            analogs {
                data.left_stick.x          => ils::ABS_X,
                (255 - data.left_stick.y)  => ils::ABS_Y,
                data.right_stick.x         => ils::ABS_RX,
                (255 - data.right_stick.y) => ils::ABS_RY,
                data.l2.0             => ils::ABS_Z,
                data.r2.0             => ils::ABS_RZ,
            }
        };

        let mut written = 0;
        while written < events.len() {
            written += self.device.write(&events[written..])?;
        }

        Ok(())
    }
}

impl Drop for UDevice {
    fn drop(&mut self) {
        match self.device.dev_destroy() {
            Ok(()) => {}
            Err(err) => warn!("error destroying uinput device: {}", err),
        }
    }
}

pub fn start(args: Args) -> JoinHandle<Result<()>> {
    std::thread::spawn(move || driver(args))
}

fn driver(args: Args) -> Result<()> {
    let mut driver = Driver {
        uinput: init_uinput()?,
        listener: EventListener::new(),
        devices: HashMap::new(),
    };

    info!("driver started");

    loop {
        for entity in driver.listener.listen(Duration::from_millis(200)) {
            let Some(devices) = driver.devices.get(&entity) else {
                continue;
            };

            for dev in devices {
                dev.update()?;
            }
        }

        for cmd in args.commands.try_iter() {
            cmd(&mut driver)?;
        }
    }
}

fn init_uinput() -> Result<PathBuf> {
    let mut udev = udev::Enumerator::new()?;
    udev.match_subsystem("misc")?;
    udev.match_sysname("uinput")?;

    let mut devices = udev.scan_devices()?;
    let device = devices.next().ok_or(anyhow::anyhow!("uinput not found"))?;

    let node = device
        .devnode()
        .ok_or(anyhow::anyhow!("uinput does not have devnode"))?;

    Ok(node.to_owned())
}
