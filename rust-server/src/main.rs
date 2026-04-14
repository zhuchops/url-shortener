use axum::{Router, http::StatusCode, routing::{get, post}};


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    _ = axum::serve(listener, app).await;
}

async fn root() -> &'static str {
    return "Hello world"
}

async fn health() -> StatusCode {
    return StatusCode::OK
}
