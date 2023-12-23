use std::{
    marker::PhantomData,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Condvar, Mutex,
    },
};

pub fn new<T>() -> (TCSender<T>, TCReceiver<T>) {
    let (send, recv) = std::sync::mpsc::channel();

    (
        TCSender { send },
        TCReceiver {
            recv: Mutex::new(recv),
        },
    )
}

pub struct ThreadCommand<F, T, R>(F, PhantomData<(T, R)>);

impl<T, F> ThreadCommand<F, T, ()>
where
    F: FnOnce(&mut T) + Send + 'static,
{
    pub fn nonblocking(func: F) -> Self {
        ThreadCommand(func, PhantomData)
    }
}

impl<T, R, F> ThreadCommand<F, T, R>
where
    R: Send + 'static,
    F: FnOnce(&mut T) -> R + Send + 'static,
{
    pub fn blocking(func: F) -> Self {
        ThreadCommand(func, PhantomData)
    }
}

pub struct TCSender<T> {
    send: Sender<Box<dyn FnOnce(&mut T) + Send>>,
}

impl<T> TCSender<T> {
    pub fn send_nonblocking<F: FnOnce(&mut T) + Send + 'static>(
        &self,
        command: ThreadCommand<F, T, ()>,
    ) {
        let _ = self.send.send(Box::new(command.0));
    }

    pub fn send_blocking<R: Send + 'static, F: FnOnce(&mut T) -> R + Send + 'static>(
        &self,
        command: ThreadCommand<F, T, R>,
    ) -> R {
        let pair = Arc::new((Mutex::new(None), Condvar::new()));
        let pair2 = Arc::clone(&pair);

        let _ = self.send.send(Box::new(move |t: &mut T| {
            let r = command.0(t);

            let (lock, cvar) = &*pair2;

            *lock.lock().unwrap() = Some(r);

            cvar.notify_one();
        }));

        let (lock, cvar) = &*pair;
        let mut ret = lock.lock().unwrap();
        while ret.is_none() {
            ret = cvar.wait(ret).unwrap();
        }

        let ret = ret.take().unwrap();

        ret
    }
}

pub struct TCReceiver<T> {
    recv: Mutex<Receiver<Box<dyn FnOnce(&mut T) + Send>>>,
}

impl<T> TCReceiver<T> {
    pub fn execute(&mut self, t: &mut T) {
        let Ok(receiver) = self.recv.get_mut() else {
            return;
        };

        for command in receiver.try_iter() {
            command(t);
        }
    }
}
