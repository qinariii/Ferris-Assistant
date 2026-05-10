pub mod models;
pub mod queries;

use sqlx::postgres::{PgPool, PgPoolOptions};

pub type Pool = PgPool;

pub async fn init_db(database_url: &str) -> Result<Pool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    Ok(pool)
}
