use std::{net::TcpStream, io::BufReader, sync::Arc};

use crate::{database_pool::Pool, request::Request, router};

pub fn handle_connection( client: &mut TcpStream, db_pool: Arc<Pool>) -> (u16, String) {
    let reader = BufReader::new(client);
    let request = Request::parse(reader);

    match request.route.as_str() {
        "GET /clientes/:id/extrato" => router::get::bank_statement(request, db_pool),
        "POST /clientes/:id/transacoes" => router::post::transaction(request, db_pool),
        _ => router::get::not_found()
    }
}
