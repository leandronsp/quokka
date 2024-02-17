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
        self.store.lock().unwrap().push_back(item);
        self.emitter.notify_one()
    }

    pub fn pop(&self) -> Option<T> {
        let mut store = self.store.lock().unwrap();

        if store.is_empty() {
            store = self.emitter.wait(store).unwrap();
        }

        store.pop_front()
    }
}
