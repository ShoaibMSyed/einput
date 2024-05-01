use std::thread::JoinHandle;

use einput_core::{device::Device, output::Output, EInput};

pub struct XboxOutput {

}

impl XboxOutput {
    pub fn new(einput: EInput) -> Self {

    }
}

impl Output for XboxOutput {
    fn name(&self) -> &str {
        "Xbox"
    }

    fn max_devices(&self) -> usize {
        4
    }

    fn update(&mut self, devices: &[Device]) {
        
    }
}

struct Shared {
    pub controllers: (),
}

pub fn start(einput: EInput) -> JoinHandle<()> {
    std::thread::spawn(move || run(einput))
}

fn run(einput: EInput) {
    
}