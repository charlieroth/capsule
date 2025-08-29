use crate::{fetcher::fetch, jobs::handler::JobHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{Span, info, instrument, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchPagePayload {
    pub item_id: Uuid,
}

#[derive(Clone)]
pub struct FetchPageJobHandler;

#[async_trait]
impl JobHandler for FetchPageJobHandler {
    #[instrument(skip(self, pool, span), fields(item_id))]
    async fn run(
        &self,
        payload: serde_json::Value,
        pool: &PgPool,
        span: Span,
    ) -> anyhow::Result<()> {
        let payload: FetchPagePayload = serde_json::from_value(payload)?;

        // Record item_id in the span
        span.record("item_id", tracing::field::display(payload.item_id));

        // Get the item URL with a lock to prevent concurrent processing
        let item_url: Option<String> = sqlx::query_scalar!(
            "SELECT url FROM items WHERE id = $1 FOR UPDATE",
            payload.item_id
        )
        .fetch_optional(pool)
        .await?;

        let Some(url) = item_url else {
            anyhow::bail!("Item {} not found", payload.item_id);
        };

        info!(
            "Fetching content for item {} from URL: {}",
            payload.item_id, url
        );

        // Fetch the page content
        match fetch(&url).await {
            Ok(response) => {
                info!(
                    "Successfully fetched content from {} (status: {}, charset: {:?}, size: {} bytes)",
                    response.url_final,
                    response.status,
                    response.charset,
                    response.body_utf8.len()
                );

                // Calculate a simple checksum of the content
                let checksum = format!("{:x}", md5::compute(response.body_raw.as_ref()));

                // Insert the content
                sqlx::query!(
                    r#"
                    INSERT INTO contents (item_id, raw_html, raw_text, lang, extracted_at, checksum)
                    VALUES ($1, $2, NULL, NULL, NOW(), $3)
                    ON CONFLICT (item_id) 
                    DO UPDATE SET 
                        raw_html = EXCLUDED.raw_html,
                        extracted_at = EXCLUDED.extracted_at,
                        checksum = EXCLUDED.checksum
                    "#,
                    payload.item_id,
                    response.body_utf8,
                    checksum
                )
                .execute(pool)
                .await?;

                // Update item status to fetched
                sqlx::query!(
                    "UPDATE items SET status = 'fetched', updated_at = NOW() WHERE id = $1",
                    payload.item_id
                )
                .execute(pool)
                .await?;

                info!("Successfully stored content for item {}", payload.item_id);
                Ok(())
            }
            Err(fetch_error) => {
                warn!(
                    "Failed to fetch content for item {}: {}",
                    payload.item_id, fetch_error
                );

                if fetch_error.should_retry() {
                    // Return error to trigger retry by job runner
                    anyhow::bail!("Retryable fetch error: {}", fetch_error);
                } else {
                    // Mark as permanent failure - don't retry
                    warn!(
                        "Permanent failure for item {}: {}",
                        payload.item_id, fetch_error
                    );

                    // Could optionally update item status to indicate permanent failure
                    // For now, just let the job be marked as failed
                    anyhow::bail!("Permanent fetch error: {}", fetch_error);
                }
            }
        }
    }

    fn kind(&self) -> &'static str {
        "fetch_page"
    }
}

impl FetchPageJobHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FetchPageJobHandler {
    fn default() -> Self {
        Self::new()
    }
}
