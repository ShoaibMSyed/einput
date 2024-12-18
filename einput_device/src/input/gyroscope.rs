/// Units are degrees/second
#[derive(Clone, Copy, Debug, Default)]
pub struct Gyroscope {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
}
