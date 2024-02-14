use std::{sync::{Mutex, Condvar}, collections::VecDeque};

pub struct Channel<T> {
    store: Mutex<VecDeque<T>>,
    emitter: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Channel<T> {
        Channel {
            store: Mutex::new(VecDeque::new()),
            emitter: Condvar::new(),
        }
    }

    pub fn send(&self, data: T) {
        self.store.lock().unwrap().push_back(data);
        self.emitter.notify_one();
    }

    pub fn recv(&self) -> Option<T> {
        let mut store = self.store.lock().unwrap();

        while store.is_empty() {
            store = self.emitter.wait(store).unwrap();
        }

        store.pop_front()
    }
}
