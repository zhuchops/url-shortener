use std::time::Duration;

use sqlx::PgPool;
use tokio::time::interval;

pub fn start_cleanup_task(pool: PgPool) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;

            sqlx::query!("delete from urls where date < NOW() - INTERVAL '1 minute'")
                .execute(&pool)
                .await
                .unwrap();
        }
    });
}
