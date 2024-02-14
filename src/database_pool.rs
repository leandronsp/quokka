use std::{sync::{Mutex, Condvar}, collections::VecDeque};

use postgres::{Client, NoTls};

pub struct Pool {
    store: Mutex<VecDeque<Client>>,
    emitter: Condvar,
}

impl Pool {
    pub fn new() -> Pool {
        Pool {
            store: Mutex::new(VecDeque::new()),
            emitter: Condvar::new(),
        }
    }

    pub fn setup(&self) {
        let connection_str = "host=postgres user=postgres password=postgres dbname=postgres";

        (0..10).for_each(|_| {
            let connection = Client::connect(connection_str, NoTls).unwrap();
            self.release(connection);
        });
    }

    pub fn release(&self, connection: Client) {
        self.store.lock().unwrap().push_back(connection);
        self.emitter.notify_one();
    }

    pub fn checkout(&self) -> Option<Client> {
        let mut store = self.store.lock().unwrap();

        while store.is_empty() {
            store = self.emitter.wait(store).unwrap();
        }

        store.pop_front()
    }
}

pub fn setup() -> Pool {
    let pool = Pool::new();
    pool.setup();

    return pool;
}
