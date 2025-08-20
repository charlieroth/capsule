use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");

    let pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // runs all pending migrations; no-op if up-to-date
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(())
}
