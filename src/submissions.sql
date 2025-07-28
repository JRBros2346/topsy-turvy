CREATE TABLE IF NOT EXISTS submissions (
    user_id TEXT,
    problem INT,
    language TEXT,
    code TEXT,
    timestamp TEXT,
    FOREIGN KEY (user_id) REFERENCES players (user_id)
);
