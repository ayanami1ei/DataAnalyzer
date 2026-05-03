use mysql::prelude::*;
use mysql::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DatabaseConfigFile {
    host: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db_name: Option<String>,
}

pub struct DbFetcher {
    pub pool: Pool,
}

impl DbFetcher {
    fn read_db_opts_from_config() -> OptsBuilder {
        let text = std::fs::read_to_string("config/database/database_config.json")
            .unwrap_or_else(|_| "{}".to_string());
        let cfg: DatabaseConfigFile = serde_json::from_str(&text).unwrap_or(DatabaseConfigFile {
            host: None,
            user: None,
            password: None,
            db_name: None,
        });

        let host = cfg
            .host
            .expect("missing host in config/database/database_config.json");
        let user = cfg
            .user
            .expect("missing user in config/database/database_config.json");
        let password = cfg
            .password
            .expect("missing password in config/database/database_config.json");
        let db_name = cfg
            .db_name
            .expect("missing db_name in config/database/database_config.json");

        OptsBuilder::new()
            .ip_or_hostname(Some(host))
            .user(Some(user))
            .pass(Some(password))
            .db_name(Some(db_name))
    }

    pub fn new() -> Self {
        let opts = Self::read_db_opts_from_config();
        DbFetcher {
            pool: Pool::new(opts).unwrap(),
        }
    }

    pub fn fetch_row(&self, batch_no: &str, material_name: &str) -> Option<Row> {
        let mut conn = self.pool.get_conn().unwrap();
        let sql = "SELECT * FROM work_record WHERE `批号` = ? AND `物料品名` = ?";
        conn.exec_first(sql, (batch_no, material_name)).unwrap()
    }
}
