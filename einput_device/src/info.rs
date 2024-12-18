use std::{
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::input::buttons::Buttons;

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub struct DeviceId(Arc<str>, u64);

impl DeviceId {
    pub fn new(id: String) -> Self {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        Self(id.into(), hasher.finish())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for DeviceId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<String> for DeviceId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&'_ str> for DeviceId {
    fn from(value: &'_ str) -> Self {
        Self::new(value.into())
    }
}

impl Into<String> for DeviceId {
    fn into(self) -> String {
        self.as_str().to_owned()
    }
}

impl Debug for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for DeviceId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for DeviceId {}

impl Hash for DeviceId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    name: String,
    id: DeviceId,
    product_name: String,
    pub kind: DeviceKind,
    pub input: DeviceInputInfo,
    pub output: DeviceOutputInfo,
}

impl DeviceInfo {
    pub fn new(name: String, product_name: String, id: DeviceId, kind: DeviceKind) -> Self {
        DeviceInfo {
            name,
            id,
            kind,
            product_name,
            input: DeviceInputInfo::default(),
            output: DeviceOutputInfo::default(),
        }
    }

    pub fn with_input(mut self, input: DeviceInputInfo) -> Self {
        self.input = input;
        self
    }

    pub fn with_output(mut self, output: DeviceOutputInfo) -> Self {
        self.output = output;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &DeviceId {
        &self.id
    }

    pub fn product_name(&self) -> &str {
        &self.product_name
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceInputInfo {
    pub acceleration: bool,
    pub buttons: Buttons,
    pub gyroscope: bool,
    pub sticks: bool,
    pub triggers: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceOutputInfo {
    pub rumble_motors: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    Mouse,
    Keyboard,
    Gamepad,
    Unknown,
}