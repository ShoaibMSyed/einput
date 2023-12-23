use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

pub struct LazySpin<T: Send + Sync> {
    state: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T: Send + Sync> LazySpin<T> {
    const UNINIT: u8 = 0;
    const INITING: u8 = 1;
    const INIT: u8 = 2;

    pub const fn new() -> Self {
        LazySpin {
            state: AtomicU8::new(0),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    #[track_caller]
    pub fn init(&self, value: T) {
        let acquire = self.state.compare_exchange(
            Self::UNINIT,
            Self::INITING,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        if acquire.is_err() {
            panic!("LazySpin already initialized");
        } else {
            let uninit = unsafe { &mut *self.value.get() };
            uninit.write(value);

            self.state.store(Self::INIT, Ordering::Release);
        }
    }

    #[track_caller]
    pub fn get(&self) -> &T {
        match self.state.load(Ordering::Acquire) {
            Self::INITING => {
                while self.state.load(Ordering::Acquire) == Self::INITING {}

                if self.state.load(Ordering::Acquire) != Self::INIT {
                    panic!("LazySpin not initialized")
                }
            }
            Self::INIT => {}
            _ => panic!("LazySpin not initialized"),
        }

        // SAFETY: self.state is only set to INIT after self.value is initialized
        unsafe { (&*self.value.get()).assume_init_ref() }
    }
}

impl<T: Send + Sync> Drop for LazySpin<T> {
    fn drop(&mut self) {
        let state = self.state.get_mut();
        let prev_state = *state;
        *state = Self::UNINIT;

        if prev_state == Self::INIT {
            // SAFETY: self.state is only set to INIT after self.value is initialized
            unsafe {
                (&mut *self.value.get()).assume_init_drop();
            }
        }
    }
}

// SAFETY: T is Send and Sync
unsafe impl<T: Send + Sync> Send for LazySpin<T> {}
unsafe impl<T: Send + Sync> Sync for LazySpin<T> {}
