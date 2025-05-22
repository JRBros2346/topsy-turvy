CREATE TABLE IF NOT EXISTS players (
    email TEXT PRIMARY KEY,
    number TEXT UNIQUE,
    solved INT
);
