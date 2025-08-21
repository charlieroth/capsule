use axum::{Router, routing::get};
use capsule::config;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = config::Config::from_env().expect("Failed to load configuration");

    let pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .connect(&config.database_url())
        .await
        .unwrap();

    let app = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind(config.bind_addr())
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello from capsule!"
}
