use crate::cleanup::start_cleanup_task;
use chrono::{self, Utc};
use rand::RngExt;
use sqlx::PgPool;
use url::Url;

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new(db_url: String) -> Result<Self, DbError> {
        let pool = PgPool::connect(db_url.as_str())
            .await
            .map_err(|err| DbError::NoSuchDatabase(err.to_string()))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|err| DbError::MigrateError(err.to_string()))?;

        start_cleanup_task(pool.clone());
        Ok(Self { pool })
    }

    pub async fn short_link(&self, host: &String, full_url: &String) -> Result<String, DbError> {
        if !Url::parse(host.as_str()).is_ok() {
            return Err(DbError::HostIsNotUrl);
        }
        if !Url::parse(full_url.as_str()).is_ok() {
            return Err(DbError::UrlIsNotUrl);
        }
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

        let url = host.to_string() + "/" + &url_id;
        Ok(format!("{}", url))
    }

    pub async fn get_link(&self, url_id: &str) -> Result<String, DbError> {
        let full_url = sqlx::query!("select full_url from urls where url_id = $1", url_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| DbError::NoSuchLink)?
            .map(|row| row.full_url)
            .ok_or(DbError::NoSuchLink)?
            .ok_or(DbError::NoSuchLink)?;
        Ok(format!("{}", full_url))
    }

    pub async fn get_url_id(&self, host: &String, full_url: &String) -> Result<String, DbError> {
        let row = sqlx::query!("select url_id from urls where full_url = $1", full_url)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        if let Some(url_id) = row.url_id {
            Ok(format!("{}/{}", host, url_id))
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
    #[error("Provided host url is not a valid url.")]
    HostIsNotUrl,
    #[error("Provided url is not a valid url.")]
    UrlIsNotUrl,
}
