use crate::device::Device;


pub trait Output {
    fn name(&self) -> &str;
    fn max_devices(&self) -> usize;
    fn update(&mut self, devices: &[Device]);
}