use std::{collections::HashMap, time::Duration};

use anyhow::{Context, Result};
use einput_core::device::DeviceReader;
use einput_device::{input::{buttons::Button, stick::StickId, triggers::TriggerId}, DeviceInput};
use einput_util::axis::StickAxis;
use vigem_client::{TargetId, XButtons, XGamepad, Xbox360Wired};

use crate::Devices;

pub fn run(devices: Devices) -> Result<()> {
    let client = vigem_client::Client::connect()
        .context("error connecting vigem client")?;

    let mut targets = Vec::new();
    let mut index_map = HashMap::new();
    let mut reader = DeviceReader::new();

    loop {
        {
            let mut lock = devices.lock().unwrap();
            if lock.changed {
                lock.changed = false;

                reader = DeviceReader::new();
                index_map.clear();

                for (i, device) in lock.list.iter().enumerate() {
                    index_map.insert(device.info().id().clone(), i);
                    device.register_reader(&mut reader);
                }

                if lock.list.len() < targets.len() {
                    for _ in 0..(targets.len() - lock.list.len()) {
                        targets.pop();
                    }
                } else if targets.len() < lock.list.len() {
                    for _ in 0..(lock.list.len() - targets.len()) {
                        let mut target = Xbox360Wired::new(&client, TargetId::XBOX360_WIRED);
                        target.plugin().context("error plugging in target")?;
                        target.wait_ready().context("error while waiting for target")?;
                        targets.push(target);
                    }
                }
            }
        }

        if let Some(map) = reader.wait_timeout(Duration::from_millis(20)) {
            for (id, input) in map {
                let Some(&index) = index_map.get(id)
                else { continue };

                let gamepad = input_to_gamepad(input);

                let target = &mut targets[index];
                match target.update(&gamepad) {
                    Ok(()) => {}
                    Err(e) => {
                        log::warn!("error updating target: {e}");
                    }
                }
            }
        }
    }
}

fn input_to_gamepad(input: &DeviceInput) -> XGamepad {
    let mut gamepad = XGamepad::default();

    for button in input.buttons().copied().unwrap_or_default().get_pressed() {
        let xbutton = match button {
            Button::Up => XButtons::UP,
            Button::Down => XButtons::DOWN,
            Button::Left => XButtons::LEFT,
            Button::Right => XButtons::RIGHT,
            Button::Start => XButtons::START,
            Button::Select => XButtons::BACK,
            Button::LStick => XButtons::LTHUMB,
            Button::RStick => XButtons::RTHUMB,
            Button::L1 => XButtons::LB,
            Button::R1 => XButtons::RB,
            Button::Home => XButtons::GUIDE,
            Button::A => XButtons::A,
            Button::B => XButtons::B,
            Button::X => XButtons::X,
            Button::Y => XButtons::Y,
            _ => 0,
        };
        gamepad.buttons.raw |= xbutton;
    }

    let l2 = if input.get(Button::L2).unwrap_or(false) {
        255
    } else if let Some(l2) = input.get(TriggerId::L2) {
        l2.0
    } else {
        0
    };

    let r2 = if input.get(Button::R2).unwrap_or(false) {
        255
    } else if let Some(r2) = input.get(TriggerId::R2) {
        r2.0
    } else {
        0
    };

    gamepad.left_trigger = l2;
    gamepad.right_trigger = r2;

    let left = input.stick(StickId::Left).copied().unwrap_or_default();
    let right = input.stick(StickId::Right).copied().unwrap_or_default();
    gamepad.thumb_lx = StickAxis::from_f32(left.x);
    gamepad.thumb_ly = StickAxis::from_f32(left.y);
    gamepad.thumb_rx = StickAxis::from_f32(right.x);
    gamepad.thumb_ry = StickAxis::from_f32(right.y);

    gamepad
}