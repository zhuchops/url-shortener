use sqlx::{PgConnection, PgPool};

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
    db_url: String,
}

impl Db {
    pub async fn new(db_url: String) -> Result<Self, DbError> {
        let pool = PgPool::connect(db_url.as_str())
            .await
            .map_err(|err| DbError::NoSuchDatabase(err.to_string()))?;

        sqlx::migrate!()
            .run(&pool)
            .await
            .map_err(|err| DbError::MigrateError(err.to_string()))?;
        Ok(Self { pool, db_url })
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
    #[error("This link already exists in database.")]
    AlreadyExists,
    #[error("This link do not exists in database.")]
    NoSuchLink,
    #[error("DB_URL is wrong, cant find such database. Exact error - {0}")]
    NoSuchDatabase(String),
    #[error("Migration went wrong. check your sql files. Exact error - {0}")]
    MigrateError(String),
}
