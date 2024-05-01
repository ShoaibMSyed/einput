use std::{fmt::Debug, ops::{BitOr, BitOrAssign}};

use serde::{Deserialize, Serialize};

use crate::{util::DeviceIndex, DeviceInput};

#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Buttons(pub u32);

impl Buttons {
    pub const ALL: Self = Self(u32::MAX);
    pub const ABXY: Self = Self::from_buttons(&[Button::A, Button::B, Button::X, Button::Y]);
    pub const DPAD: Self = Self::from_buttons(&[Button::Up, Button::Down, Button::Left, Button::Right]);
    pub const BUMPERS: Self = Self::from_buttons(&[Button::L1, Button::R1]);
    pub const TRIGGERS: Self = Self::from_buttons(&[Button::L2, Button::R2]);

    const fn from_buttons(buttons: &[Button]) -> Self {
        let mut this = Buttons(0);

        let mut i = 0;
        while i < buttons.len() {
            this.0 = this.0 | (1 << buttons[i] as u8);
            i += 1;
        }

        this
    }

    pub fn is_pressed(self, button: Button) -> bool {
        self.0 & button.bit() != 0
    }

    pub fn set(&mut self, button: Button, pressed: bool) {
        self.0 &= !button.bit();
        if pressed {
            self.0 |= button.bit();
        }
    }

    pub fn get_pressed(self) -> impl Iterator<Item = Button> {
        Button::ALL.into_iter().filter(move |b| self.is_pressed(*b))
    }
}

impl Debug for Buttons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#022b}", self.0)
    }
}

impl BitOr<Button> for Buttons {
    type Output = Self;

    fn bitor(mut self, rhs: Button) -> Self::Output {
        self.set(rhs, true);
        self
    }
}

impl BitOrAssign<Button> for Buttons {
    fn bitor_assign(&mut self, rhs: Button) {
        *self = *self | rhs;
    }
}

impl BitOr for Buttons {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Button {
    A,
    B,
    X,
    Y,
    Up,
    Down,
    Left,
    Right,
    Start,
    Select,
    Home,
    Share,
    LStick,
    RStick,
    L1,
    R1,
    L2,
    R2,
    L3,
    R3,
    L4,
    R4,
}

impl Button {
    pub const ALL: [Self; 22] = [
        Self::A,
        Self::B,
        Self::X,
        Self::Y,
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::Start,
        Self::Select,
        Self::Home,
        Self::Share,
        Self::LStick,
        Self::RStick,
        Self::L1,
        Self::R1,
        Self::L2,
        Self::R2,
        Self::L3,
        Self::R3,
        Self::L4,
        Self::R4,
    ];

    pub fn bit(self) -> u32 {
        1 << self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::X => "X",
            Self::Y => "Y",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Start => "Start",
            Self::Select => "Select",
            Self::Home => "Home",
            Self::Share => "Share",
            Self::LStick => "LStick",
            Self::RStick => "RStick",
            Self::L1 => "L1",
            Self::R1 => "R1",
            Self::L2 => "L2",
            Self::R2 => "R2",
            Self::L3 => "L3",
            Self::R3 => "R3",
            Self::L4 => "L4",
            Self::R4 => "R4",
        }
    }
}

impl DeviceIndex<DeviceInput> for Button {
    type Output<'a> = Option<bool>;

    fn index<'a>(self, device: &'a DeviceInput) -> Self::Output<'a> {
        Some(device.buttons()?.is_pressed(self))
    }
}

impl BitOr for Button {
    type Output = Buttons;

    fn bitor(self, rhs: Self) -> Self::Output {
        Buttons::default() | self | rhs
    }
}