use std::{net::TcpListener, io::{Write, Read, BufReader, BufRead}, collections::HashMap};

use chrono::Local;
use postgres::{Client, NoTls};
use regex::Regex;
use serde_json::{Value, json};

fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Listening on the port 3000");

    for client in listener.incoming() {
        let mut client = client.unwrap();
        let mut reader = BufReader::new(&client);
        let mut headline = String::new();
        let mut params: HashMap<&str, String> = HashMap::new();
        
        let _ = reader.read_line(&mut headline);
        print!("{}", headline);

        let headline_pattern = Regex::new(r"^(GET|POST)\s\/clientes\/(\d+)\/(.*?)\sHTTP.*?").unwrap();
        let captures = headline_pattern.captures(&headline).unwrap();

        let verb = captures.get(1).unwrap().as_str();
        let id = captures.get(2).unwrap().as_str();
        let suffix = captures.get(3).unwrap().as_str();

        let request_constraint = format!("{verb} /clientes/:id/{suffix}");

        params.insert("id", id.to_string());

        let mut content_length: u64 = 0;

        for line in reader.by_ref().lines() {
            let line = line.unwrap();
            println!("{}", line);

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
            let amount = &parsed["valor"];
            let transaction_type: &str = parsed["tipo"].as_str().unwrap();
            let description: &str = parsed["descricao"].as_str().unwrap();

            params.insert("valor", amount.to_string());
            params.insert("tipo", transaction_type.to_string());
            params.insert("descricao", description.to_string());
        }

        println!("Params: {:?}", params);

        let mut status = 200;
        let mut response_body = String::new();

        match request_constraint.as_str() {
            "GET /clientes/:id/extrato" => {
                let mut db = Client::connect("host=localhost user=postgres password=postgres dbname=postgres", NoTls).unwrap();

                let account_id = params["id"].parse::<i32>().unwrap();

                let account_query = r#"
                    SELECT 
                        accounts.limit_amount AS limit_amount,
                        balances.amount AS balance
                    FROM accounts
                    JOIN balances ON balances.account_id = accounts.id 
                    WHERE accounts.id = $1
                "#;

                if let Ok(account) = db.query_one(account_query, &[&account_id]) {
                    let limit_amount: i32 = account.get("limit_amount");
                    let balance: i32 = account.get("balance");

                    let ten_transactions_query = r#"
                        SELECT
                            amount,
                            transaction_type,
                            description,
                            TO_CHAR(date, 'YYYY-MM-DD') AS date
                        FROM transactions
                        WHERE account_id = $1
                        ORDER BY date DESC
                        LIMIT 10
                    "#;

                    let ten_transactions = db.query(ten_transactions_query, &[&account_id]).unwrap();

                    let ten_transactions_json: Vec<_> = ten_transactions.into_iter().map(|transaction| {
                        let amount: i32 = transaction.get("amount");
                        let description: &str = transaction.get("description");
                        let transaction_type: &str = transaction.get("transaction_type");
                        let transaction_date: &str = transaction.get("date");

                        json!({
                            "valor": amount,
                            "tipo": transaction_type,
                            "descricao": description,
                            "realizada_em": transaction_date
                        })
                    }).collect();

                    response_body = json!({
                        "saldo": json!({
                            "limite": limit_amount,
                            "total": balance,
                            "data_extrato": Local::now().to_string()
                        }),
                        "ultimas_transacoes": ten_transactions_json
                    }).to_string();
                } else {
                    status = 404;
                    response_body = json!({}).to_string();
                }
            },
            "POST /clientes/:id/transacoes" => {
                let mut db = Client::connect("host=localhost user=postgres password=postgres dbname=postgres", NoTls).unwrap();

                let account_id = params["id"].parse::<i32>().unwrap();

                let account_query = r#"
                    SELECT 
                        accounts.limit_amount AS limit_amount,
                        balances.amount AS balance
                    FROM accounts
                    JOIN balances ON balances.account_id = accounts.id 
                    WHERE accounts.id = $1
                "#;

                if let Ok(account) = db.query_one(account_query, &[&account_id]) {
                    let amount = params["valor"].parse::<i32>().unwrap();
                    let transaction_type = &params["tipo"];
                    let description = &params["descricao"];

                    let insert_stmt = r#"
                        INSERT INTO transactions (account_id, amount, transaction_type, description)
                        VALUES ($1, $2, $3, $4)
                    "#;

                    let _ = db.execute(insert_stmt, &[&account_id, &amount, &transaction_type, &description]).unwrap();

                    println!("Type: {}", transaction_type);

                    if transaction_type == &"c" {
                        let update_stmt = r#"
                            UPDATE balances 
                            SET amount = amount + $2
                            WHERE account_id = $1
                        "#;

                        let _ = db.execute(update_stmt, &[&account_id, &amount]).unwrap();
                    } else {
                        let update_stmt = r#"
                            UPDATE balances 
                            SET amount = amount - $2
                            WHERE account_id = $1
                        "#;

                        let _ = db.execute(update_stmt, &[&account_id, &amount]).unwrap();
                    }

                    let account = db.query_one(account_query, &[&account_id]).unwrap();
                    let limit_amount: i32 = account.get("limit_amount");
                    let balance: i32 = account.get("balance");

                    response_body = json!({
                        "limite": limit_amount,
                        "saldo": balance
                    }).to_string();
                } else {
                    status = 404;
                    response_body = json!({}).to_string();
                }

            },
            _ => {
                status = 404;
            }
        }

        let response = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\n\r\n{response_body}");
        let _ = client.write_all(response.as_bytes());
    }
}
