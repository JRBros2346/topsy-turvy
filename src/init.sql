CREATE TABLE IF NOT EXISTS players (
    email TEXT PRIMARY KEY,
    number TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS submissions (
    email TEXT,
    code TEXT,
    language TEXT,
    status TEXT,
    FOREIGN KEY(email) REFERENCES players(email)
);
