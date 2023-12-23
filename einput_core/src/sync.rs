use std::{
    sync::mpsc::{Receiver, SyncSender},
    time::Duration,
};

use bevy::prelude::Entity;

pub struct EventListener {
    send: SyncSender<Entity>,
    recv: Receiver<Entity>,
}

impl EventListener {
    pub fn new() -> Self {
        let (send, recv) = std::sync::mpsc::sync_channel(16);

        EventListener { send, recv }
    }

    pub fn handle(&self) -> EventHandle {
        EventHandle {
            send: self.send.clone(),
        }
    }

    pub fn listen(&self, timeout: Duration) -> impl Iterator<Item = Entity> + '_ {
        let first = self.recv.recv_timeout(timeout).ok();

        self.recv.try_iter().chain(first)
    }
}

pub struct EventHandle {
    pub(crate) send: SyncSender<Entity>,
}
