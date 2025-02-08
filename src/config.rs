use argon2::{Argon2, PasswordHash, PasswordVerifier as _};
use chacha20poly1305::{
    aead::{Aead as _, Nonce},
    ChaCha20Poly1305, KeyInit,
};
use libsql::{params::IntoParams, Connection, Rows, Transaction};

#[derive(Clone)]
pub struct Config {
    conn: Connection,
    admin_hash: String,
    admin_token: String,
    cipher: ChaCha20Poly1305,
    nonce: Nonce<ChaCha20Poly1305>,
}
impl Config {
    pub async fn query(&self, sql: &str, params: impl IntoParams) -> Option<Rows> {
        self.conn
            .query(sql, params)
            .await
            .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
            .ok()
    }
    pub async fn execute(&self, sql: &str, params: impl IntoParams) {
        if let Err(e) = self.conn.execute(sql, params).await {
            tracing::error!("{e} {} {}", file!(), line!())
        }
    }
    pub async fn transaction(&self) -> Option<Transaction> {
        self.conn
            .transaction()
            .await
            .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
            .ok()
    }
    pub fn get_admin_token(&self, password: &str) -> Option<String> {
        match Self::argon2_verify(password, &self.admin_hash) {
            Some(true) => Some(self.admin_token.clone()),
            _ => None,
        }
    }
    pub fn verify_admin_token(&self, token: &str) -> bool {
        token == self.admin_token
    }
    pub fn encrypt(&self, plaintext: &str) -> Option<String> {
        self.cipher
            .encrypt(&self.nonce, plaintext.as_bytes())
            .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
            .ok()
            .map(hex::encode)
    }
    pub fn decrypt(&self, ciphertext: &str) -> Option<String> {
        hex::decode(ciphertext)
            .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
            .ok()
            .map(|ciphertext| {
                self.cipher
                    .decrypt(&self.nonce, ciphertext.as_ref())
                    .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
                    .ok()
            })
            .flatten()
            .map(String::from_utf8)
            .map(|res| {
                res.inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
                    .ok()
            })
            .flatten()
    }
    pub fn argon2_generate(password: &str) -> Option<String> {
        use argon2::{
            password_hash::{rand_core::OsRng, SaltString},
            PasswordHasher as _,
        };
        Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
            .ok()
            .as_ref()
            .map(PasswordHash::to_string)
    }
    pub fn argon2_verify(password: &str, hash: &str) -> Option<bool> {
        Some(
            Argon2::default()
                .verify_password(
                    password.as_bytes(),
                    PasswordHash::new(hash)
                        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
                        .ok()
                        .as_ref()?,
                )
                .is_ok(),
        )
    }
    pub fn hash(text: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hex::encode(hasher.finalize())
    }
    pub async fn new() -> Self {
        use chacha20poly1305::{ChaCha20Poly1305, Key};
        use libsql::{params, Builder};
        use std::env;
        tracing::info!("Initializing configuration");
        let output = Self {
            conn: Builder::new_local(env::current_dir().unwrap().join("revil.db"))
                .build()
                .await
                .unwrap()
                .connect()
                .unwrap(),
            admin_hash: Config::argon2_generate(&env::var("ADMIN_PASS").unwrap()).unwrap(),
            admin_token: env::var("ADMIN_TOKEN").unwrap(),
            cipher: ChaCha20Poly1305::new(Key::from_slice(
                &hex::decode(Config::hash(&env::var("SECRET_KEY").unwrap())).unwrap(),
            )),
            nonce: Nonce::<ChaCha20Poly1305>::clone_from_slice(
                &hex::decode(Config::hash(&env::var("NONCE").unwrap())).unwrap()[..12],
            ),
        };
        output.execute(include_str!("init.sql"), params![]).await;
        output
    }
}
