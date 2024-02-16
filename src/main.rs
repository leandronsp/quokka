use std::io::Write;
use std::net::TcpListener;

mod handler;
mod request;
mod router;

fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Listening on the port 3000");

    for client in listener.incoming() {
        let mut client = client.unwrap();
        let (status, body) = handler::handle_connection(&mut client);

        let response = 
            format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\n\r\n{body}");

        let _ = client.write_all(response.as_bytes());
    }
}
