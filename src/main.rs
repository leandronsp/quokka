use std::{sync::Arc, thread};
use std::net::TcpListener;

mod thread_pool;
mod handler;

pub(crate) mod database_pool;

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
                let client = channel.recv().unwrap(); 
                handler::handle_connection(client, db_pool.clone());
            }
        });
    });

    for client in listener.incoming() {
        let client = client.unwrap();
        thread_channel.send(client);
    }
}
