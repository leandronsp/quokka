pub mod get {
    use std::sync::Arc;

    use chrono::Local;
    use serde_json::json;

    use crate::{request::Request, database_pool::Pool};

    pub fn bank_statement(request: Request, db_pool: Arc<Pool>) -> (u16, String) {
        let mut status = 200;
        let mut body = json!({}).to_string();

        let mut db_conn = db_pool.clone().checkout().unwrap();
        let account_id: i32 = request.params["id"].parse::<i32>().unwrap();

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

            body = json!({
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

        (status, body)
    }
    
    pub fn not_found() -> (u16, String) {
        (404, json!({}).to_string())
    }
}

pub mod post {
    use std::sync::Arc;

    use serde_json::json;

    use crate::{request::Request, database_pool::Pool};

    pub fn transaction(request: Request, db_pool: Arc<Pool>) -> (u16, String) {
        let mut status = 200;
        let mut body = json!({}).to_string();

        let mut db_conn = db_pool.clone().checkout().unwrap();
        let account_id: i32 = request.params["id"].parse::<i32>().unwrap();

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
            let amount: i32 = request.params["valor"].parse::<i32>().unwrap_or(0);
            let transaction_type: &str = request.params["tipo"].as_str();
            let description: &str = request.params["descricao"].as_str();

            let limit_amount: i32 = account.get("limit_amount");
            let balance: i32 = account.get("balance");

            if amount == 0 ||
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


                body = json!({
                    "limite": limit_amount,
                    "saldo": balance
                }).to_string();
            }
        } else {
            status = 404;
        }

        db_transaction.commit().unwrap();
        db_pool.clone().release(db_conn);

        (status, body)
    }

    fn reached_limit(balance: i32, limit_amount: i32, amount: i32) -> bool {
        if (balance - amount) > limit_amount {
            return false
        }

        return (balance - amount).abs() > limit_amount
    }
}
