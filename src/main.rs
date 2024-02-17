use std::collections::VecDeque;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, Condvar};
use std::thread;

use crate::database::Database;

mod handler;
mod request;
mod router;
mod database;

pub struct Queue<T> {
    store: Mutex<VecDeque<T>>,
    emitter: Condvar
}

impl<T> Queue<T> {
    fn new() -> Queue<T> {
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

fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Listening on the port 3000");

    let queue: Arc<Queue<TcpStream>> = Arc::new(Queue::new());
    let db_pool: Arc<Queue<Database>> = Arc::new(Queue::new());

    (0..10).for_each(|_| {
        let db = Database::new();
        db_pool.push(db);
    });

    (0..5).for_each(|_| {
        let queue = Arc::clone(&queue);
        let pool = Arc::clone(&db_pool);

        thread::spawn(move || {
            loop {
                let client = queue.pop().unwrap();
                handle(client, pool.clone());
            }
        });
    });

    for client in listener.incoming() {
        let client = client.unwrap();
        queue.push(client);
    }
}

fn handle(mut client: TcpStream, db_pool: Arc<Queue<Database>>) {
    let (status, body) = handler::handle_connection(&mut client, db_pool);

    let response = 
        format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\n\r\n{body}");

    let _ = client.write_all(response.as_bytes());
}
