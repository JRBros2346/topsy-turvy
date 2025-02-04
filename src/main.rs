use axum::{body::Body, http::Response, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

mod code;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Language {
    Rust,
    Cpp,
    Javascript,
    Python,
    Java,
}

#[derive(Deserialize)]
struct Submission {
    problem: String,
    code: String,
    language: Language,
}

#[derive(Serialize)]
enum Output {
    ServerError,
    CannotCompile(String),
}

impl IntoResponse for Output {
    fn into_response(self) -> Response<Body> {
        use axum::http::StatusCode;
        match self {
            Output::ServerError => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
            Output::CannotCompile(err) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(err))
                .unwrap(),
        }
    }
}

async fn submit(Json(payload): Json<Submission>) -> Output {
    use code::wasm::wasm_test;
    match payload.language {
        Language::Rust => {
            use code::rust::rust_compile;
            let wasm = rust_compile(&payload.code).await;
            match wasm {
                Ok(wasm) => wasm_test(wasm, 100, 100, vec![(vec![], "10".to_string())]).await,
                Err(err) => err,
            }
        }
        Language::Cpp => {
            use code::cpp::cpp_compile;
            let wasm = cpp_compile(&payload.code).await;
            match wasm {
                Ok(wasm) => wasm_test(wasm, 100, 100, vec![(vec![], "10".to_string())]).await,
                Err(err) => err,
            }
        }
        Language::Javascript => Output::ServerError,
        Language::Python => Output::ServerError,
        Language::Java => Output::ServerError,
    }
}

#[tokio::main]
async fn main() {
    use axum::routing;
    use axum::Router;
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        Router::new().route("/api/submit", routing::post(submit)),
    )
    .await
    .unwrap();
}
