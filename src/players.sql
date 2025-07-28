CREATE TABLE IF NOT EXISTS players (
    user_id TEXT PRIMARY KEY,
    password TEXT UNIQUE,
    solved INT
);
