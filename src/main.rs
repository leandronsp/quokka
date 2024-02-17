
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc};
use std::thread;

use crate::database::Database;
use crate::queue::Queue;

mod queue;
mod handler;
mod request;
mod router;
mod database;

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
                let client = queue.pop();
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
