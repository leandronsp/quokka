use std::{sync::{Mutex, Condvar}, collections::VecDeque};

pub struct Queue<T> {
    store: Mutex<VecDeque<T>>,
    emitter: Condvar
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Self {
            store: Mutex::new(VecDeque::new()),
            emitter: Condvar::new(),
        }
    }

    pub fn push(&self, item: T) {
        self.store.lock().expect("Cloud not acquire lock on mutex").push_back(item);
        self.emitter.notify_one()
    }

    pub fn pop(&self) -> T {
        let mut store = self.store.lock().expect("Could not acquire lock on mutex");

        while store.is_empty() {
            store = self.emitter.wait(store).unwrap();
        }

        store.pop_front().expect("The queue is empty")
    }
}
