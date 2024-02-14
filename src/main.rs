use std::{sync::Arc, thread};
use std::net::TcpListener;

mod thread_pool;
mod handler;

pub(crate) mod database_pool;

fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Listening on the port 3000");

    // Thread Pool
    // Can't use mpsc due to ownerhsip issues: multiple producers can 
    // take ownership of "tx", but only one consumer (one thread) can 
    // take ownership of "rx"

    let channel = Arc::new(thread_pool::Channel::new());

    (0..5).for_each(|_| {
        let channel = channel.clone();

        thread::spawn(move || {
            loop {
                let client = channel.recv().unwrap(); 
                handler::handle_connection(client);
            }
        });
    });

    for client in listener.incoming() {
        let client = client.unwrap();
        channel.send(client);
    }
}
