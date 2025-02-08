use libsql::Connection;

#[derive(Clone)]
pub struct Config {
    pub conn: Connection,
    pub admin_hash: String,
    pub admin_token: String,
    pub secret_key: String,
}
