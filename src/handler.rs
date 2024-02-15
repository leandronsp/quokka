use std::{net::TcpStream, io::{BufReader, BufRead, Read, Write}, collections::HashMap, sync::Arc};

use chrono::Local;
use regex::Regex;
use serde_json::{Value, json};

use crate::database_pool::Pool;

pub fn handle_connection(mut client: TcpStream, db_pool: Arc<Pool>) {
    let mut reader = BufReader::new(&client);
    let mut headline = String::new();
    let mut params: HashMap<&str, String> = HashMap::new();
    
    let _ = reader.read_line(&mut headline);
    //println!("{}", headline);

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
        //println!("{}", line);

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
        let amount = parsed["valor"].as_u64().unwrap_or(0);
        let transaction_type: &str = parsed["tipo"].as_str().unwrap_or("");
        let description: &str = parsed["descricao"].as_str().unwrap_or("");

        params.insert("valor", amount.to_string());
        params.insert("tipo", transaction_type.to_string());
        params.insert("descricao", description.to_string());
    }

    //println!("Params: {:?}", params);

    let mut status = 200;
    let mut response_body = json!({}).to_string();

    match request_constraint.as_str() {
        "GET /clientes/:id/extrato" => {
            let mut db_conn = db_pool.clone().checkout().unwrap();
            let account_id = params["id"].parse::<i32>().unwrap();

            let account_query = r#"
                SELECT 
                    limit_amount,
                    balance
                FROM accounts
                WHERE accounts.id = $1
            "#;

            let mut db_transaction = db_conn.transaction().unwrap();

            if let Ok(account) = db_transaction.query_one(account_query, &[&account_id]) {
                let limit_amount: i32 = account.get("limit_amount");
                let balance: i32 = account.get("balance");

                let ten_transactions_query = r#"
                    SELECT
                        amount,
                        transaction_type,
                        description,
                        TO_CHAR(date, 'YYYY-MM-DD HH:MI:SS.US') AS date
                    FROM transactions
                    WHERE account_id = $1
                    ORDER BY date DESC
                    LIMIT 10
                "#;

                let ten_transactions = db_transaction.query(ten_transactions_query, &[&account_id]).unwrap();

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
            }

            db_transaction.commit().unwrap();
            db_pool.clone().release(db_conn);
        },
        "POST /clientes/:id/transacoes" => {
            let mut db_conn = db_pool.clone().checkout().unwrap();
            let account_id = params["id"].parse::<i32>().unwrap();

            let account_query = r#"
                SELECT 
                    limit_amount,
                    balance
                FROM accounts
                WHERE accounts.id = $1
                FOR UPDATE
            "#;

            let mut db_transaction = db_conn.transaction().unwrap();

            if let Ok(account) = db_transaction.query_one(account_query, &[&account_id]) {
                let amount = params["valor"].parse::<i32>().unwrap();
                let transaction_type = params["tipo"].as_str();
                let description = params["descricao"].as_str();

                let limit_amount: i32 = account.get("limit_amount");
                let balance: i32 = account.get("balance");

                if amount.is_negative() || 
                        amount == 0 ||
                        !vec!["c", "d"].contains(&transaction_type) ||
                        description.is_empty() ||
                        description.len() > 10 ||
                        (transaction_type == "d" && reached_limit(balance, limit_amount, amount)) {
                    status = 422
                } else {
                    let insert_stmt = r#"
                        INSERT INTO transactions (account_id, amount, transaction_type, description)
                        VALUES ($1, $2, $3, $4)
                    "#;

                    let _ = db_transaction.execute(insert_stmt, &[&account_id, &amount, &transaction_type, &description]).unwrap();

                    if transaction_type == "c" {
                        let update_stmt = r#"
                            UPDATE accounts 
                            SET balance = balance + $2
                            WHERE accounts.id = $1
                        "#;

                        let _ = db_transaction.execute(update_stmt, &[&account_id, &amount]).unwrap();
                    } else {
                        let update_stmt = r#"
                            UPDATE accounts 
                            SET balance = balance - $2
                            WHERE accounts.id = $1
                        "#;

                        let _ = db_transaction.execute(update_stmt, &[&account_id, &amount]).unwrap();
                    }

                    let account = db_transaction.query_one(account_query, &[&account_id]).unwrap();
                    let limit_amount: i32 = account.get("limit_amount");
                    let balance: i32 = account.get("balance");


                    response_body = json!({
                        "limite": limit_amount,
                        "saldo": balance
                    }).to_string();
                }
            } else {
                status = 404;
            }

            db_transaction.commit().unwrap();
            db_pool.clone().release(db_conn);
        },
        _ => {
            status = 404;
        }
    }

    let response = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\n\r\n{response_body}");
    let _ = client.write_all(response.as_bytes());
}

fn reached_limit(balance: i32, limit_amount: i32, amount: i32) -> bool {
    if (balance - amount) > limit_amount {
        return false
    }

    return (balance - amount).abs() > limit_amount
}
