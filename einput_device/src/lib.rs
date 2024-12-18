#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

mod info;
pub mod input;
pub mod output;
pub mod util;

pub use self::info::{DeviceId, DeviceInfo, DeviceInputInfo, DeviceOutputInfo, DeviceKind};
pub use self::input::DeviceInput;
pub use self::output::DeviceOutput;

macro_rules! component_accessors {
    ($typename:ident ; single { $($sname:ident: $styp:ty ;)* } multi { $($uname:ident : $utyp:ty ;)* }) => {
        paste::paste! {
            impl $typename {
                $(
                    pub fn $sname(&self) -> Option<&$styp> {
                        let offset = self.header().$sname?;
                        Some(unsafe { offset.get(self.ptr.as_ptr()) })
                    }

                    pub fn [< $sname _mut >](&mut self) -> Option<&mut $styp> {
                        let offset = self.header().$sname?;
                        Some(unsafe { offset.get_mut(self.ptr.as_ptr()) })
                    }
                )*

                $(
                    pub fn [< $uname s >](&self) -> &[$utyp] {
                        let Some(offset) = self.header().$uname
                        else { return &[] };

                        unsafe { offset.get(self.ptr.as_ptr()) }
                    }

                    pub fn [< $uname s_mut >](&mut self) -> &mut [$utyp] {
                        let Some(offset) = self.header().$uname
                        else { return &mut [] };

                        unsafe { offset.get_mut(self.ptr.as_ptr()) }
                    }
                )*
            }
        }
    }
}

pub(crate) use component_accessors;
