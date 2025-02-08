CREATE TABLE IF NOT EXISTS submissions (
    email TEXT,
    problem INT,
    language TEXT,
    code TEXT,
    timestamp TEXT,
    FOREIGN KEY (email) REFERENCES players (email)
);
