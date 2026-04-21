use chrono::{self, NaiveDate, Utc};
use rand::RngExt;
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

    pub async fn short_link(&self, host: String, full_url: String) -> Result<String, DbError> {
        let url_id = self.generate_unique_url_id().await;
        let date = Utc::now();
        sqlx::query!(
            "insert into urls (url_id, full_url, date) values ($1, $2, $3)",
            url_id,
            full_url,
            date.date_naive()
        )
        .execute(&self.pool)
        .await
        .map_err(|_| DbError::AlreadyExists)?;

        let url = host + "/" + &url_id;
        Ok(format!("{}", url))
    }

    pub async fn get_link(&self, url_id: String) -> Result<String, DbError> {
        let row = sqlx::query!("select full_url from urls where url_id = $1", url_id)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        if let Some(full_url) = row.full_url {
            Ok(format!("{}", full_url))
        } else {
            Err(DbError::NoSuchLink)
        }
    }

    async fn generate_unique_url_id(&self) -> String {
        let mut url_id = Db::generate_url_id();
        while !self.check_url_id(&url_id).await.unwrap() {
            url_id = Db::generate_url_id();
        }
        url_id
    }

    fn generate_url_id() -> String {
        const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        const LENGHT: i8 = 10;
        let mut rng = rand::rng();
        (0..LENGHT)
            .map(|_| {
                let idx = rng.random_range(0..ALPHABET.len());
                ALPHABET[idx] as char
            })
            .collect()
    }

    async fn check_url_id(&self, url_id: &String) -> Result<bool, DbError> {
        match sqlx::query!("select * from urls where url_id = $1", url_id)
            .fetch_optional(&self.pool)
            .await
            .unwrap()
        {
            Some(_) => Ok(false),
            None => Ok(true),
        }
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
