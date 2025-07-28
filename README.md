# Topsy-Turvy Backend Server

This is a backend server for a competitive programming platform, built with Rust and Axum. It supports user authentication, code submission, admin management, and problem/testcase handling. The server can be run locally or via Docker.

---

## Table of Contents
- [Setup](#setup)
- [Environment Variables](#environment-variables)
- [Database Schema](#database-schema)
- [Test Cases](#test-cases)
- [Running the Server](#running-the-server)
- [API Routes](#api-routes)
  - [User Routes](#user-routes)
  - [Admin Routes](#admin-routes)
- [Docker Usage](#docker-usage)

---

## Setup

1. **Install Rust**: [https://rustup.rs/](https://rustup.rs/)
2. **Clone the repository**:
   ```sh
   git clone <repo-url>
   cd topsy-turvy
   ```
3. **Set up environment variables** (see below).
4. **Build and run**:
   ```sh
   cargo run --release
   ```
   The server will listen on `0.0.0.0:3000` by default.

---

## Environment Variables
Create a `.env` file in the project root with the following keys:

```
ADMIN_PASS=toor         # Admin password (hashed internally)
ADMIN_TOKEN=secret      # Token used for admin authentication
SECRET_KEY=secret       # Key for encryption (32 bytes recommended)
NONCE=nonce             # Nonce for encryption (12 bytes recommended)
```

---

## Database Schema

The server uses SQLite (via `libsql`). The following tables are required:

- **players** (`src/players.sql`):
  ```sql
  CREATE TABLE IF NOT EXISTS players (
      user_id TEXT PRIMARY KEY,
      password TEXT UNIQUE,
      solved INT
  );
  ```
- **submissions** (`src/submissions.sql`):
  ```sql
  CREATE TABLE IF NOT EXISTS submissions (
      user_id TEXT,
      problem INT,
      language TEXT,
      code TEXT,
      timestamp TEXT,
      FOREIGN KEY (user_id) REFERENCES players (user_id)
  );
  ```

---

## Test Cases

Test cases for problems are defined in `test_cases.ron` using the following format:

```
[
    TestCases (
        public: [TestCase (input: ..., output: ...), ...],
        hidden: TestCase (input: ..., output: ...),
    ),
    ...
]
```

- **public**: List of public test cases for each problem.
- **hidden**: One hidden test case per problem.

---

## Running the Server

### Locally
- Build and run with `cargo run --release`.
- Ensure `.env` and database files are present.

### With Docker
- Build and run using Docker Compose:
  ```sh
  docker-compose up --build
  ```
- The server will be available at `localhost:3000`.

---

## API Routes

### User Routes

- `POST /api/auth`
  - **Body**: `{ "user_id": string, "password": string }`
  - **Response**: `{ "status": "Token", "message": string }` (token)

- `POST /api/submit`
  - **Headers**: `Authorization: <token>` (from `/api/auth`)
  - **Body**: `{ "code": string, "language": "Rust"|"Cpp"|"Javascript"|"Python"|"Java" }`
  - **Response**: Various status (see below)

- `GET /api/solved`
  - **Headers**: `Authorization: <token>`
  - **Response**: `{ "status": "Solved", "message": number }`

### Admin Routes
All admin routes require `Authorization: <ADMIN_TOKEN>` header.

- `POST /admin/auth`
  - **Body**: `"<ADMIN_PASS>"`
  - **Response**: `{ "status": "Token", "message": string }` (admin token)

- `POST /admin/add_player`
  - **Headers**: `Authorization: <ADMIN_TOKEN>`
  - **Body**: `{ "user_id": string, "password": string }`
  - **Response**: Success/Failure

- `POST /admin/change_password`
  - **Headers**: `Authorization: <ADMIN_TOKEN>`
  - **Body**: `{ "user_id": string, "password": string }`
  - **Response**: Success/Failure

- `GET /admin/get_players`
  - **Headers**: `Authorization: <ADMIN_TOKEN>`
  - **Response**: List of all player user_ids

- `GET /admin/get_submissions`
  - **Headers**: `Authorization: <ADMIN_TOKEN>`
  - **Response**: List of all submissions

---

## Output Status Types

- `ServerError`, `InvalidProblem`, `Completed`, `CannotCompile`, `RuntimeError`, `Timeout`, `WrongAnswer`, `Hidden`, `HiddenTimeout`, `Accepted`, `Unauthorized`, `Solved`, `Token`

---

## Notes
- The server supports code execution in Rust, C++, JavaScript, Python, and Java.
- All code is run in a sandboxed temporary directory.
- For production, use strong secrets for `SECRET_KEY` and `NONCE`.
- For more details, see the source code and comments.
