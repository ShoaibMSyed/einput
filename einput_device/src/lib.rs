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
    ($typename:ident ; single { $($sname:ident: $styp:ty ;)* } multi { $($uname:ident : $utyp:ty ;)* } map { $($mname:ident : $mtyp:ty [ $mid:ty ] ;)* }) => {
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

                $(
                    pub fn [< $mname >](&self, id: $mid) -> Option<&$mtyp> {
                        let offset = self.header().[< $mname s >]?;
                        unsafe { offset.get_one(self.ptr.as_ptr(), id) }
                    }

                    pub fn [< $mname _mut >](&mut self, id: $mid) -> Option<&mut $mtyp> {
                        let offset = self.header().[< $mname s >]?;
                        unsafe { offset.get_one_mut(self.ptr.as_ptr(), id) }
                    }

                    pub fn [< $mname s >](&self) -> crate::util::MapRef<$mid, $mtyp> {
                        let Some(offset) = self.header().[< $mname s >]
                        else { return crate::util::MapRef::new(&[], [0; <$mid as crate::util::InputId>::LEN as usize]) };
                        unsafe { offset.get(self.ptr.as_ptr()) }
                    }

                    pub fn [< $mname s _mut >](&mut self) -> crate::util::MapMut<$mid, $mtyp> {
                        let Some(offset) = self.header().[< $mname s >]
                        else { return crate::util::MapMut::new(&mut [], [0; <$mid as crate::util::InputId>::LEN as usize]) };
                        unsafe { offset.get_mut(self.ptr.as_ptr()) }
                    }
                )*
            }
        }
    }
}

pub(crate) use component_accessors;
