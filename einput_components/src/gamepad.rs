use core::ops::{BitOr, BitOrAssign};

use bytemuck::{Pod, Zeroable};
use einput_device::impl_apply_config;
use einput_util::{Stick, StickAxis, Trigger, TriggerAxis};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct Gamepad {
    pub buttons: Buttons,
    pub left_stick: Stick,
    pub right_stick: Stick,
    pub l1: Trigger,
    pub r1: Trigger,
    pub l2: Trigger,
    pub r2: Trigger,
}

#[cfg(feature = "info")]
#[derive(Clone, Debug, Default)]
pub struct GamepadInfo {
    pub name: String,
}

#[cfg(feature = "config")]
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct GamepadConfig {}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct Buttons(pub u64);

#[repr(u64)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    A = 1 << 0,
    B = 1 << 1,
    X = 1 << 2,
    Y = 1 << 3,
    Up = 1 << 4,
    Down = 1 << 5,
    Left = 1 << 6,
    Right = 1 << 7,
    Start = 1 << 8,
    Select = 1 << 9,
    L1 = 1 << 10,
    R1 = 1 << 11,
    L2 = 1 << 12,
    R2 = 1 << 13,
    L3 = 1 << 14,
    R3 = 1 << 15,
    L4 = 1 << 16,
    R4 = 1 << 17,
    LStick = 1 << 18,
    RStick = 1 << 19,
    Home = 1 << 20,
    Capture = 1 << 21,
}

#[cfg(feature = "device")]
impl einput_device::Component for Gamepad {
    #[cfg(feature = "info")]
    type Info = GamepadInfo;
    #[cfg(feature = "config")]
    type Config = GamepadConfig;
}

#[cfg(feature = "device")]
impl einput_device::ComponentConfig for GamepadConfig {
    type Component = Gamepad;
    fn apply(&self, _: &mut Gamepad) {}
}

impl_apply_config!(GamepadConfig);

impl Gamepad {
    /// Sets L1 and R1 to their minimum or maximum values based on their
    /// button values.
    #[inline]
    pub fn set_shoulders_from_buttons(&mut self) {
        self.l1 = self.buttons.pressed(Button::L1).to_u8().into();
        self.r1 = self.buttons.pressed(Button::R1).to_u8().into();
    }

    /// Sets L2 and R2 to their minimum or maximum values based on their
    /// button values.
    #[inline]
    pub fn set_triggers_from_buttons(&mut self) {
        self.l2 = self.buttons.pressed(Button::L2).to_u8().into();
        self.r2 = self.buttons.pressed(Button::R2).to_u8().into();
    }

    /// Sets L1 and R1 buttons based on their respective trigger's value.
    #[inline]
    pub fn set_buttons_from_shoulders(
        &mut self,
        left_threshold: Trigger,
        right_threshold: Trigger,
    ) {
        self.buttons.set(Button::L1, self.l1.0 >= left_threshold.0);
        self.buttons.set(Button::R1, self.r1.0 >= right_threshold.0);
    }

    /// Sets L2 and R2 buttons based on their respective trigger's value.
    #[inline]
    pub fn set_buttons_from_triggers(&mut self, left_threshold: Trigger, right_threshold: Trigger) {
        self.buttons.set(Button::L2, self.l2.0 >= left_threshold.0);
        self.buttons.set(Button::R2, self.r2.0 >= right_threshold.0);
    }

    /// Sets the left stick axes based on the dpad.
    #[inline]
    pub fn set_left_stick_from_dpad(&mut self) {
        self.left_stick.x = if self.buttons.pressed(Button::Left) {
            StickAxis::min()
        } else if self.buttons.pressed(Button::Right) {
            StickAxis::max()
        } else {
            StickAxis::neutral()
        };
        self.left_stick.y = if self.buttons.pressed(Button::Up) {
            StickAxis::min()
        } else if self.buttons.pressed(Button::Down) {
            StickAxis::max()
        } else {
            StickAxis::neutral()
        };
    }
}

impl Buttons {
    pub fn pressed(self, button: Button) -> bool {
        self.0 & button.bit() != 0
    }

    pub fn set(&mut self, button: Button, pressed: bool) {
        if pressed {
            self.0 |= button.bit();
        } else {
            self.0 &= !button.bit();
        }
    }
}

impl BitOr<Button> for Buttons {
    type Output = Buttons;

    fn bitor(mut self, rhs: Button) -> Self::Output {
        self.0 |= rhs.bit();
        self
    }
}

impl BitOr<Buttons> for Buttons {
    type Output = Buttons;

    fn bitor(mut self, rhs: Buttons) -> Self::Output {
        self.0 |= rhs.0;
        self
    }
}

impl BitOrAssign<Button> for Buttons {
    fn bitor_assign(&mut self, rhs: Button) {
        *self = *self | rhs;
    }
}

impl BitOrAssign<Buttons> for Buttons {
    fn bitor_assign(&mut self, rhs: Buttons) {
        *self = *self | rhs;
    }
}

impl From<u64> for Buttons {
    fn from(value: u64) -> Self {
        Buttons(value)
    }
}
impl From<Buttons> for u64 {
    fn from(value: Buttons) -> Self {
        value.0
    }
}

impl Button {
    pub const ALL: [Self; 22] = [
        Button::A,
        Button::B,
        Button::X,
        Button::Y,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
        Button::Start,
        Button::Select,
        Button::L1,
        Button::R1,
        Button::L2,
        Button::R2,
        Button::L3,
        Button::R3,
        Button::L4,
        Button::R4,
        Button::LStick,
        Button::RStick,
        Button::Home,
        Button::Capture,
    ];

    pub fn bit(self) -> u64 {
        self as _
    }
}

impl BitOr<Buttons> for Button {
    type Output = Buttons;

    fn bitor(self, rhs: Buttons) -> Self::Output {
        rhs | self
    }
}

impl BitOr<Button> for Button {
    type Output = Buttons;

    fn bitor(self, rhs: Button) -> Self::Output {
        Buttons::default() | self | rhs
    }
}

impl TryFrom<u64> for Button {
    type Error = u64;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(match value {
            0b1 => Button::A,
            0b10 => Button::B,
            0b100 => Button::X,
            0b1000 => Button::Y,
            0b10000 => Button::Up,
            0b100000 => Button::Down,
            0b1000000 => Button::Left,
            0b10000000 => Button::Right,
            0b100000000 => Button::Start,
            0b1000000000 => Button::Select,
            0b10000000000 => Button::L1,
            0b100000000000 => Button::R1,
            0b1000000000000 => Button::R2,
            0b10000000000000 => Button::L2,
            0b100000000000000 => Button::L3,
            0b1000000000000000 => Button::R3,
            0b10000000000000000 => Button::L4,
            0b100000000000000000 => Button::R4,
            0b1000000000000000000 => Button::LStick,
            0b10000000000000000000 => Button::RStick,
            0b100000000000000000000 => Button::Home,
            0b1000000000000000000000 => Button::Capture,
            _ => return Err(value),
        })
    }
}

impl From<Button> for u64 {
    fn from(value: Button) -> u64 {
        value.bit()
    }
}

impl core::fmt::Display for Button {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:#?}")
    }
}

#[macro_export]
macro_rules! map_to_buttons {
    ($input:expr => $($bit:literal = $but:expr),* $(,)?) => {
        $crate::gamepad::Buttons::default()
        $(| if $input & (1 << $bit) != 0 { $crate::gamepad::Buttons($but.bit()) } else { $crate::gamepad::Buttons::default() })*
    };
}
