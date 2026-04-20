mod db;

use std::{default, env};

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};

use serde::Deserialize;
use sqlx::PgPool;

use crate::db::{Db, DbError};

#[tokio::main]
async fn main() {
    let db_url = env::var("DB_URL").expect("DB_URL must be set");
    let db = Db::new(db_url).await;
    let app = Router::new()
        .route("/short", post(short))
        .route("/get/{id}", get(get_url))
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

async fn short(Query(params): Query<PostUrl>, State(db): State<Db>) -> Result<String, StatusCode> {
    let true_url = params.link;
    let shorted_url_result = db.short_link(true_url);
    match shorted_url_result.await {
        Ok(url) => Ok(url),
        Err(err) => match err {
            DbError::AlreadyExists => Err(StatusCode::CONFLICT),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

async fn get_url(State(db): State<Db>, Path(id): Path<String>) -> Result<String, StatusCode> {
    let true_url_result = db.get_link(id).await;
    match true_url_result {
        Ok(true_url) => Ok(true_url),
        Err(err) => match err {
            DbError::NoSuchLink => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}
