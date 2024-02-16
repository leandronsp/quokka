use std::io::Write;
use std::{sync::Arc, thread};
use std::net::TcpListener;

mod thread_pool;
mod handler;
mod router;

pub(crate) mod database_pool;
pub(crate) mod request;

fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Listening on the port 3000");

    let db_pool = Arc::new(database_pool::setup());
    let thread_channel = Arc::new(thread_pool::Channel::new());

    (0..5).for_each(|_| {
        let channel = thread_channel.clone();
        let db_pool = db_pool.clone();

        thread::spawn(move || {
            loop {
                let mut client = channel.recv().unwrap(); 
                let (status, body) = handler::handle_connection(&mut client, db_pool.clone());

                let response = 
                    format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\n\r\n{body}");

                let _ = client.write_all(response.as_bytes());
            }
        });
    });

    for client in listener.incoming() {
        let client = client.unwrap();
        thread_channel.send(client);
    }
}
