use anyhow::Result;
use capsule::{
    config::Config,
    jobs::{ExampleJobPayload, JobRepository},
};
use serde_json::json;

/// Demo program that enqueues some example jobs to test the job runner
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
        .max_connections(5)
        .connect(config.database_url())
        .await?;

    println!("Enqueuing example jobs...");

    // Enqueue a simple job
    let job1_payload = ExampleJobPayload {
        message: "Hello from job 1!".to_string(),
        delay_ms: Some(2000),
    };

    let job1_id =
        JobRepository::enqueue(&pool, "example_job", json!(job1_payload), None, None).await?;

    println!("Enqueued job 1: {}", job1_id);

    // Enqueue another job
    let job2_payload = ExampleJobPayload {
        message: "Hello from job 2!".to_string(),
        delay_ms: Some(1000),
    };

    let job2_id =
        JobRepository::enqueue(&pool, "example_job", json!(job2_payload), None, None).await?;

    println!("Enqueued job 2: {}", job2_id);

    // Enqueue a job that will fail (invalid payload)
    let job3_id = JobRepository::enqueue(
        &pool,
        "example_job",
        json!({"invalid": "payload"}),
        None,
        Some(3), // Max 3 attempts
    )
    .await?;

    println!("Enqueued failing job: {}", job3_id);

    println!("Jobs enqueued! Start the worker with: cargo run --bin worker");
    println!("Monitor job status with SQL queries on the jobs table.");

    Ok(())
}
