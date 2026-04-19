use sqlx::{PgConnection, PgPool};

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
    db_url: String
}

impl Db {
    pub async fn new(db_url: String) -> Self {
        let pool = PgPool::connect(db_url.as_str()).await.unwrap();
        Self { pool, db_url }
    }

    pub async fn short_link(&self, url: String) -> Result<String, DbError> {
        Ok(format!("shorted version of {}", url))
    }

    pub async fn get_link(&self, url: String) -> Result<String, DbError> {
        Ok(format!("real url of {}", url))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("This link already exists in database")]
    AlreadyExists,
    #[error("This link do not exists in database")]
    NoSuchLink,
}
