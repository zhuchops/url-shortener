mod cleanup;
mod db;

use std::env;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};

use serde::Deserialize;

use crate::db::{Db, DbError};

#[tokio::main]
async fn main() {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Db::new(db_url).await.unwrap();
    let app = Router::new()
        .route("/short", post(short))
        .route("/get/{id}", get(get_url))
        .route("/{id}", get(get_url))
        .with_state(db)
        .route("/", get(root))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    _ = axum::serve(listener, app).await;
}

async fn root() -> String {
    return "Hello world".to_string();
}

async fn health() -> StatusCode {
    return StatusCode::OK;
}

#[derive(Deserialize)]
struct PostUrl {
    link: String,
}

async fn short(
    Query(params): Query<PostUrl>,
    State(db): State<Db>,
) -> Result<(StatusCode, String), StatusCode> {
    let full_url = params.link;
    let host = env::var("HOST_URL").expect("HOST_URL env var must be set.");
    let shorted_url_result = db.short_link(&host, &full_url);
    match shorted_url_result.await {
        Ok(url) => Ok((StatusCode::CREATED, url)),
        Err(err) => match err {
            DbError::AlreadyExists => {
                let url = db.get_url_id(&host, &full_url).await.unwrap();
                Ok((StatusCode::FOUND, url))
            }
            DbError::HostIsNotUrl => Err(StatusCode::BAD_REQUEST),
            DbError::UrlIsNotUrl => Err(StatusCode::BAD_REQUEST),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

async fn get_url(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let full_url_result = db.get_link(&id).await;
    match full_url_result {
        Ok(full_url) => Ok(Redirect::permanent(full_url.as_str())),
        Err(err) => match err {
            DbError::NoSuchLink => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}
