use std::ops::Mul;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stick<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T: StickAxis> Default for Stick<T> {
    fn default() -> Self {
        Self {
            x: T::neutral(),
            y: T::neutral(),
        }
    }
}

impl<T> From<[T; 2]> for Stick<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Stick { x, y }
    }
}

impl Stick<f32> {
    pub fn from_xy<T: StickAxis>(x: T, y: T) -> Self {
        Stick {
            x: x.to_f32(),
            y: y.to_f32(),
        }
    }

    pub fn length(&self) -> f32 {
        f32::sqrt(self.x.powi(2) + self.y.powi(2))
    }

    pub fn normalized(self) -> Self {
        if self.length() <= std::f32::EPSILON {
            return Self::default();
        }

        self * (1.0 / self.length())
    }
}

impl Mul<f32> for Stick<f32> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::from_xy(self.x * rhs, self.y * rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trigger<T = u8>(pub T);

impl<T: TriggerAxis> Trigger<T> {
    pub fn min() -> Self {
        Self(T::min())
    }

    pub fn max() -> Self {
        Self(T::max())
    }
}

impl<T: TriggerAxis> Default for Trigger<T> {
    fn default() -> Self {
        Trigger(T::min())
    }
}

impl<T: TriggerAxis> From<T> for Trigger<T> {
    fn from(value: T) -> Self {
        Trigger(value)
    }
}

macro_rules! axis_to_int {
    (stick $($int:ty $(=> $mul:literal)?),* $(,)?) => {
        paste::paste! {
            $(
                fn [< to_ $int >](self) -> $int
                where
                    Self: Sized
                {
                    let range = libm::fabs($int::MIN as f64) + $int::MAX as f64;
                    let subtract = range * 0.5 $(* $mul)?;
                    let value = (((self.to_f64() + 1.0) / 2.0) * range) - subtract;
                    value as $int
                }
            )*
        }
    };
    (trigger $($int:ty),* $(,)?) => {
        paste::paste! {
            $(
                fn [< to_ $int >](self) -> $int
                where
                    Self: Sized
                {
                    let range = $int::MAX as f64;
                    let value = self.to_f64() * range;
                    value as $int
                }
            )*
        }
    };
}

pub trait StickAxis: Copy + Clone {
    fn neutral() -> Self;
    fn min() -> Self;
    fn max() -> Self;

    fn invert(self) -> Self;

    fn from_f32(value: f32) -> Self;

    /// Returns a floating point value between -1.0 and 1.0
    fn to_f32(self) -> f32
    where
        Self: Sized,
    {
        self.to_f64() as _
    }

    /// Returns a floating point value between -1.0 and 1.0
    fn to_f64(self) -> f64;

    axis_to_int! {
        stick
        i8, i16, i32, i64,
        u8 => 0.0,
        u16 => 0.0,
        u32 => 0.0,
        u64 => 0.0,
    }
}

pub trait TriggerAxis: Copy + Clone {
    fn min() -> Self;
    fn max() -> Self;

    fn from_f32(value: f32) -> Self {
        Self::from_f64(value as _)
    }

    fn from_f64(value: f64) -> Self;

    /// Returns a floating point value between 0.0 and 1.0
    fn to_f32(self) -> f32
    where
        Self: Sized,
    {
        self.to_f64() as _
    }

    /// Returns a floating point value between 0.0 and 1.0
    fn to_f64(self) -> f64;

    axis_to_int! {
        trigger
        u8, u16, u32, u64,
    }
}

macro_rules! impl_axis {
    (stick unsigned) => {
        fn neutral() -> Self {
            Self::MAX / 2
        }

        fn min() -> Self {
            Self::MIN
        }

        fn max() -> Self {
            Self::MAX
        }

        fn invert(self) -> Self {
            Self::MAX - self
        }

        fn from_f32(value: f32) -> Self {
            (((value + 1.0) / 2.0) * Self::MAX as f32) as _
        }

        fn to_f64(self) -> f64 {
            let half = Self::MAX as f64 / 2.0;
            (self as f64 - half) / half
        }
    };
    (stick signed) => {
        fn neutral() -> Self {
            0
        }

        fn min() -> Self {
            Self::MIN
        }

        fn max() -> Self {
            Self::MAX
        }

        fn invert(self) -> Self {
            (0 as Self).saturating_sub(self)
        }

        fn from_f32(value: f32) -> Self {
            (value * Self::MAX as f32) as _
        }

        fn to_f64(self) -> f64 {
            (self as f64) / (Self::MAX as f64)
        }
    };
    (trigger unsigned) => {
        fn from_f64(value: f64) -> Self {
            let range = libm::fabs(Self::MIN as f64) + Self::MAX as f64;
            let value = value * range + (Self::MIN as f64);
            value as Self
        }

        fn min() -> Self {
            0
        }

        fn max() -> Self {
            Self::MAX
        }

        fn to_f64(self) -> f64 {
            let range = libm::fabs(Self::MIN as f64) + Self::MAX as f64;
            (self as f64) / range
        }
    };
}

impl StickAxis for u8 {
    impl_axis!(stick unsigned);

    fn to_u8(self) -> u8 {
        self
    }

    fn to_u16(self) -> u16 {
        let this = self;
        (this as u16) | ((this as u16) << 8)
    }

    fn to_u32(self) -> u32 {
        let this = StickAxis::to_u16(self);
        (this as u32) | ((this as u32) << 16)
    }

    fn to_u64(self) -> u64 {
        let this = StickAxis::to_u32(self);
        (this as u64) | ((this as u64) << 32)
    }
}

impl StickAxis for u16 {
    impl_axis!(stick unsigned);

    fn to_u8(self) -> u8 {
        (self >> 8) as u8
    }

    fn to_u16(self) -> u16 {
        self
    }

    fn to_u32(self) -> u32 {
        let this = self;
        (this as u32) | ((this as u32) << 16)
    }

    fn to_u64(self) -> u64 {
        let this = StickAxis::to_u32(self);
        (this as u64) | ((this as u64) << 32)
    }
}

impl StickAxis for u32 {
    impl_axis!(stick unsigned);

    fn to_u8(self) -> u8 {
        (self >> 24) as u8
    }

    fn to_u16(self) -> u16 {
        (self >> 16) as u16
    }

    fn to_u32(self) -> u32 {
        self
    }

    fn to_u64(self) -> u64 {
        let this = self;
        (this as u64) | ((this as u64) << 32)
    }
}

impl StickAxis for u64 {
    impl_axis!(stick unsigned);

    fn to_u8(self) -> u8 {
        (self >> 56) as u8
    }

    fn to_u16(self) -> u16 {
        (self >> 48) as u16
    }

    fn to_u32(self) -> u32 {
        (self >> 32) as u32
    }

    fn to_u64(self) -> u64 {
        self
    }
}

impl StickAxis for i8 {
    impl_axis!(stick signed);

    fn to_i8(self) -> i8 {
        self
    }
}

impl StickAxis for i16 {
    impl_axis!(stick signed);

    fn to_i16(self) -> i16 {
        self
    }
}

impl StickAxis for i32 {
    impl_axis!(stick signed);

    fn to_i32(self) -> i32 {
        self
    }
}

impl StickAxis for i64 {
    impl_axis!(stick signed);

    fn to_i64(self) -> i64 {
        self
    }
}

impl StickAxis for f32 {
    fn neutral() -> Self {
        0.0
    }

    fn min() -> Self {
        -1.0
    }

    fn max() -> Self {
        1.0
    }

    fn invert(self) -> Self {
        -self
    }

    fn from_f32(value: f32) -> Self {
        value
    }

    fn to_f32(self) -> f32 {
        self
    }

    fn to_f64(self) -> f64 {
        self as _
    }
}

impl StickAxis for f64 {
    fn neutral() -> Self {
        0.0
    }

    fn min() -> Self {
        -1.0
    }

    fn max() -> Self {
        1.0
    }

    fn invert(self) -> Self {
        -self
    }

    fn from_f32(value: f32) -> Self {
        value as _
    }

    fn to_f64(self) -> f64 {
        self
    }
}

impl TriggerAxis for u8 {
    impl_axis!(trigger unsigned);

    fn to_u8(self) -> u8 {
        self
    }

    fn to_u16(self) -> u16 {
        let this = self;
        (this as u16) | ((this as u16) << 8)
    }

    fn to_u32(self) -> u32 {
        let this = TriggerAxis::to_u16(self);
        (this as u32) | ((this as u32) << 16)
    }

    fn to_u64(self) -> u64 {
        let this = TriggerAxis::to_u32(self);
        (this as u64) | ((this as u64) << 32)
    }
}

impl TriggerAxis for u16 {
    impl_axis!(trigger unsigned);

    fn to_u8(self) -> u8 {
        (self >> 8) as u8
    }

    fn to_u16(self) -> u16 {
        self
    }

    fn to_u32(self) -> u32 {
        let this = self;
        (this as u32) | ((this as u32) << 16)
    }

    fn to_u64(self) -> u64 {
        let this = TriggerAxis::to_u32(self);
        (this as u64) | ((this as u64) << 32)
    }
}

impl TriggerAxis for u32 {
    impl_axis!(trigger unsigned);

    fn to_u8(self) -> u8 {
        (self >> 24) as u8
    }

    fn to_u16(self) -> u16 {
        (self >> 16) as u16
    }

    fn to_u32(self) -> u32 {
        self
    }

    fn to_u64(self) -> u64 {
        let this = self;
        (this as u64) | ((this as u64) << 32)
    }
}

impl TriggerAxis for u64 {
    impl_axis!(trigger unsigned);

    fn to_u8(self) -> u8 {
        (self >> 56) as u8
    }

    fn to_u16(self) -> u16 {
        (self >> 48) as u16
    }

    fn to_u32(self) -> u32 {
        (self >> 32) as u32
    }

    fn to_u64(self) -> u64 {
        self
    }
}

impl TriggerAxis for bool {
    fn min() -> Self {
        false
    }

    fn max() -> Self {
        true
    }

    fn from_f64(value: f64) -> Self {
        value >= 1.0
    }

    fn to_f32(self) -> f32 {
        self as u8 as f32
    }

    fn to_f64(self) -> f64 {
        self as u8 as f64
    }

    fn to_u8(self) -> u8 {
        self as u8 * u8::MAX
    }

    fn to_u16(self) -> u16 {
        self as u8 as u16 * u16::MAX
    }

    fn to_u32(self) -> u32 {
        self as u8 as u32 * u32::MAX
    }

    fn to_u64(self) -> u64 {
        self as u8 as u64 * u64::MAX
    }
}
