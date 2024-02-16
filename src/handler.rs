use std::{net::TcpStream, io::BufReader};

use crate::{request::Request, router};

pub fn handle_connection( client: &mut TcpStream) -> (u16, String) {
    let reader = BufReader::new(client);
    let request = Request::parse(reader);

    match request.route.as_str() {
        "GET /clientes/:id/extrato" => router::get::bank_statement(request),
        "POST /clientes/:id/transacoes" => router::post::transaction(request),
        _ => router::get::not_found()
    }
}
