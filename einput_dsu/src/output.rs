use std::{collections::HashMap, sync::{Arc, Mutex}, time::Duration};

use anyhow::Result;
use dsu::{packet::{Button as DsuButton, ControllerInfo, SendControllerData}, server::Server};
use einput_core::{device::{Device, DeviceReader}, output::Output};
use einput_device::{input::buttons::Button, DeviceId, DeviceInput};
use einput_util::axis::StickAxis;
use log::{info, warn};


type Devices = Arc<Mutex<DeviceList>>;

#[derive(Default)]
struct DeviceList {
    list: Vec<Device>,
    changed: bool,
}

pub struct DsuOutput {
    devices: Devices,
}

impl DsuOutput {
    pub fn new() -> Self {
        let devices = Devices::default();
        start(devices.clone());

        Self { devices }
    }
}

impl Output for DsuOutput {
    fn name(&self) -> &str {
        "dsu"
    }

    fn max_devices(&self) -> usize {
        4
    }

    fn update(&mut self, devices: &[Device]) {
        let mut lock = self.devices.lock().unwrap();
        lock.list.clear();
        lock.list.extend_from_slice(devices);
        lock.changed = true;
    }
}

fn start(devices: Devices) {
    std::thread::spawn(move || {
        loop {
            devices.lock().unwrap().changed = true;

            let devices = devices.clone();

            info!("starting dsu server thread");

            match Thread::new(devices).and_then(Thread::run) {
                Ok(()) => info!("dsu server thread exited"),
                Err(e) => info!("dsu server thread error: {e:?}, restarting..."),
            }
            
            std::thread::sleep(Duration::from_secs(3));
        }
    });
}

struct Thread {
    devices: Devices,
    indexes: HashMap<DeviceId, usize>,
    reader: DeviceReader,
    server: Server,
}

impl Thread {
    const SERVER_ID: u32 = 0xDEDEDE00;

    fn new(devices: Devices) -> Result<Self> {
        Ok(Self {
            devices,
            indexes: HashMap::new(),
            reader: DeviceReader::new(),
            server: Server::new(true, Self::SERVER_ID, dsu::server::default_address())?,
        })
    }

    fn run(mut self) -> Result<()> {
        loop {
            self.update_reader()?;

            if let Some(map) = self.reader.wait_timeout(Duration::from_millis(20)) {
                for (id, input) in map {
                    let Some(&index) = self.indexes.get(id)
                    else { continue };

                    let data = &mut self.server.controllers[index];
                    Self::update(data, input);
                }
            }
            
            self.server.remove_old_clients();
            match self.server.receive() {
                Ok(()) => {}
                Err(e) => warn!("dsu server receive error: {e:?}"),
            }
            self.server.send();
        }
    }

    fn update_reader(&mut self) -> Result<()> {
        let macs: [[u8; 6]; 4] = std::array::from_fn(|i| [i as u8 + 1; 6]);

        let mut lock = self.devices.lock().unwrap();

        if !lock.changed {
            return Ok(());
        }

        lock.changed = false;
    
        self.reader = DeviceReader::new();
        self.indexes.clear();

        self.server.controllers = std::array::from_fn(|i| SendControllerData::new(ControllerInfo::disconnected(i as u8)));
    
        for (i, device) in lock.list.iter().enumerate() {
            self.indexes.insert(device.info().id().clone(), i);
            device.register_reader(&mut self.reader);

            if i > 4 { continue; }

            self.server.controllers[i].info = ControllerInfo {
                slot: i as u8,
                state: ControllerInfo::STATE_CONNECTED,
                model: ControllerInfo::MODEL_FULL_GYRO,
                connection: ControllerInfo::CONNECTION_USB,
                mac: macs[i],
                battery: ControllerInfo::BATTERY_FULL,
            };
            self.server.controllers[i].update_connected();
        }

        Ok(())
    }

    fn update(data: &mut SendControllerData, input: &DeviceInput) {
        data.l1 = 0;
        data.r1 = 0;
        data.l2 = 0;
        data.r2 = 0;

        if let Some(acceleration) = input.acceleration() {
            data.accel_x = acceleration.x;
            data.accel_y = acceleration.y;
            data.accel_z = acceleration.z;
        }

        if let Some(buttons) = input.buttons() {
            for button in Button::ALL {
                let pressed = buttons.is_pressed(button);

                let Some(mask) = Self::button(button)
                else { continue };

                data.buttons &= !(mask as u16);
                if pressed {
                    data.buttons |= mask as u16;
                }

                Self::set_analog_button(mask, pressed, data);
            }
        }

        if let Some(gyroscope) = input.gyroscope() {
            data.gyro_pitch = gyroscope.pitch;
            data.gyro_roll = gyroscope.roll;
            data.gyro_yaw = gyroscope.yaw;
        }

        if let Some(sticks) = input.sticks() {
            data.lsx = sticks.left.x.to_u8();
            data.lsy = sticks.left.y.to_u8();
            data.rsx = sticks.right.x.to_u8();
            data.rsy = sticks.right.y.to_u8();
        }

        if let Some(triggers) = input.triggers() {
            if data.l1 != 255 {
                data.l1 = triggers.l1.0;
            }
            if data.r1 != 255 {
                data.r1 = triggers.r1.0;
            }
            if data.l2 != 255 {
                data.l2 = triggers.l2.0;
            }
            if data.r2 != 255 {
                data.r2 = triggers.r2.0;
            }
        }
    }

    fn button(button: Button) -> Option<DsuButton> {
        Some(match button {
            Button::Select => DsuButton::Share,
            Button::LStick => DsuButton::L3,
            Button::RStick => DsuButton::R3,
            Button::Start => DsuButton::Options,
            Button::Up => DsuButton::Up,
            Button::Right => DsuButton::Right,
            Button::Down => DsuButton::Down,
            Button::Left => DsuButton::Left,
            Button::L2 => DsuButton::L2,
            Button::R2 => DsuButton::R2,
            Button::L1 => DsuButton::L1,
            Button::R1 => DsuButton::R1,
            Button::X => DsuButton::X,
            Button::A => DsuButton::A,
            Button::B => DsuButton::B,
            Button::Y => DsuButton::Y,
            _ => return None,
        })
    }

    fn set_analog_button(button: DsuButton, pressed: bool, data: &mut SendControllerData) {
        let analog = match button {
            DsuButton::Up => &mut data.up,
            DsuButton::Right => &mut data.right,
            DsuButton::Down => &mut data.down,
            DsuButton::Left => &mut data.left,
            DsuButton::L2 => &mut data.l2,
            DsuButton::R2 => &mut data.r2,
            DsuButton::L1 => &mut data.l1,
            DsuButton::R1 => &mut data.r1,
            DsuButton::X => &mut data.x,
            DsuButton::A => &mut data.a,
            DsuButton::B => &mut data.b,
            DsuButton::Y => &mut data.y,
            _ => return,
        };

        *analog = (pressed as u8) * 255;
    }
}