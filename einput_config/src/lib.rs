pub mod input;

use einput_core::device::DeviceTransformer;
use serde::{Deserialize, Serialize};

use self::input::DeviceInputConfig;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DeviceConfig {
    pub input: DeviceInputConfig,
}

impl DeviceConfig {
    pub fn compile(&self) -> DeviceTransformer {
        let this = self.clone();

        DeviceTransformer::new(move || {
            let this = this.clone();

            Box::new(move |input| { this.input.apply(input); })
        })
    }
}