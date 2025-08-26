use anyhow::Result;
use capsule::{
    config::Config,
    jobs::{ExampleJobHandler, FetchPageJobHandler, JobRegistry, WorkerConfig, WorkerSupervisor},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    // Create database connection pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(config.database_url())
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Create job registry and register handlers
    let mut registry = JobRegistry::new();
    registry.register(ExampleJobHandler);
    registry.register(FetchPageJobHandler::new());

    // Create worker configuration
    let worker_config = WorkerConfig {
        concurrency: std::env::var("WORKER_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4),
        poll_interval_ms: std::env::var("WORKER_POLL_INTERVAL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000),
        visibility_timeout_secs: std::env::var("WORKER_VISIBILITY_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300),
        base_backoff_secs: std::env::var("WORKER_BASE_BACKOFF_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30),
    };

    // Create and run supervisor
    let supervisor = WorkerSupervisor::new(pool, registry, worker_config);
    supervisor.run().await
}
