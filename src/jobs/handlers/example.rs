use crate::jobs::JobHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tracing::{Span, info};

/// Example job payload for demonstrating the job system
#[derive(Debug, Serialize, Deserialize)]
pub struct ExampleJobPayload {
    pub message: String,
    pub delay_ms: Option<u64>,
}

/// Example job handler that logs a message and optionally sleeps
#[derive(Clone, Debug)]
pub struct ExampleJobHandler;

#[async_trait]
impl JobHandler for ExampleJobHandler {
    async fn run(&self, payload: Value, _pool: &PgPool, _span: Span) -> anyhow::Result<()> {
        let payload: ExampleJobPayload = serde_json::from_value(payload)?;

        info!("Processing example job: {}", payload.message);

        if let Some(delay_ms) = payload.delay_ms {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            info!("Finished sleeping for {}ms", delay_ms);
        }

        info!("Example job completed successfully");
        Ok(())
    }

    fn kind(&self) -> &'static str {
        "example_job"
    }
}
