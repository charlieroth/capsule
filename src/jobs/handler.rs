use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;
use tracing::Span;

/// Trait for handling specific job types
#[async_trait]
pub trait JobHandler: Send + Sync + 'static {
    /// Execute the job
    async fn run(&self, payload: Value, pool: &PgPool, span: Span) -> anyhow::Result<()>;

    /// Get the job kind this handler processes
    fn kind(&self) -> &'static str;
}

/// Type-erased job handler factory
pub type JobHandlerFactory =
    Box<dyn Fn(Value) -> anyhow::Result<Box<dyn JobHandler>> + Send + Sync>;
