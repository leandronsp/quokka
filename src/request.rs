use std::{io::{BufReader, BufRead, Read}, net::TcpStream, collections::HashMap};

use regex::Regex;
use serde_json::Value;

#[derive(Debug)]
pub struct Request {
    pub route: String,
    pub params: HashMap<&'static str, String>,
}

impl Request {
    fn new() -> Self {
        Self { route: String::new(), params: HashMap::new(), }
    }

    pub fn parse(mut reader: BufReader<&mut TcpStream>) -> Request {
        let mut request = Self::new();
        let mut headline = String::new();

        let _ = reader.read_line(&mut headline);
        //println!("{}", headline);

        let headline_pattern = 
            Regex::new(r"^(GET|POST)\s\/clientes\/(\d+)\/(.*?)\sHTTP.*?").unwrap();

        if let Some(captures) = headline_pattern.captures(&headline) {
            let verb = captures.get(1).unwrap().as_str();
            let id = captures.get(2).unwrap().as_str();
            let suffix = captures.get(3).unwrap().as_str();

            request.route = format!("{verb} /clientes/:id/{suffix}");
            request.params.insert("id", id.to_string());

            let mut content_length: u64 = 0;

            for line in reader.by_ref().lines() {
                let line = line.unwrap();
                let parts: Vec<_> = line.split(":").collect();

                let header_name = parts.get(0).unwrap();

                if header_name == &"Content-Length" {
                    content_length = parts.get(1).unwrap().trim().parse::<u64>().unwrap();
                }

                if line.is_empty() {
                    break;
                }
            }

            if content_length > 0 {
                let mut body = String::new();
                let _ = reader.take(content_length).read_to_string(&mut body);

                let parsed: Value = serde_json::from_str(&body).unwrap();
                let amount = parsed["valor"].to_string();
                let transaction_type = parsed["tipo"].as_str().unwrap_or("").to_string();
                let description = parsed["descricao"].as_str().unwrap_or("").to_string();

                request.params.insert("valor", amount);
                request.params.insert("tipo", transaction_type);
                request.params.insert("descricao", description);
            }
        } 

        request
    }
}
