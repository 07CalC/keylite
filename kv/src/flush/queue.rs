use crate::memtable::Memtable;
use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;

pub enum FlushMessage {
    Flush(Arc<Memtable>),
    Shutdown,
}

pub struct FlushQueue {
    sender: Sender<FlushMessage>,
    receiver: Receiver<FlushMessage>,
}

impl FlushQueue {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }

    pub fn sender(&self) -> Sender<FlushMessage> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> Receiver<FlushMessage> {
        self.receiver.clone()
    }
}
