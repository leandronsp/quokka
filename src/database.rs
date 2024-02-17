use std::env;

use postgres::{Client, NoTls};

pub struct Database {
    pub conn: Client,
}

impl Database {
    pub fn new() -> Database {
        Self {
            conn: Client::connect(Self::configuration().as_str(), NoTls).expect("Database is down")
        }
    }

    fn configuration() -> String {
        let configmap = [
            ("host", "DATABASE_HOST"),
            ("user", "DATABASE_USER"),
            ("password", "DATABASE_PASSWORD"),
            ("dbname", "DATABASE_NAME"),
        ];

        let mut str_config = String::new();

        for (pg_attr, env_var) in configmap.iter() {
            if let Ok(env_value) = env::var(env_var) {
                str_config.push_str(&format!("{}={} ", pg_attr, env_value).to_string());
            } else {
                str_config.push_str(&format!("{}=postgres ", pg_attr).to_string());
            }
        }

        str_config
    }
}
