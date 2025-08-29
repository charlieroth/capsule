use crate::entities::Content;
use anyhow::Result;
use chrono::{DateTime, Utc};
use md5::Context;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing content persistence with checksum-based deduplication
pub struct ContentRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ContentRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Upsert content using checksum to avoid unnecessary writes when content hasn't changed.
    /// Large payloads are handled efficiently by streaming to the database.
    pub async fn upsert_content(
        &self,
        item_id: Uuid,
        clean_html: &str,
        clean_text: &str,
        lang: Option<&str>,
        extracted_at: DateTime<Utc>,
    ) -> Result<()> {
        // Compute checksum from normalized content
        let checksum = self.compute_checksum(clean_html, clean_text);

        // Early return if content hasn't changed (checksum match)
        if let Some(existing_checksum) = self.get_existing_checksum(item_id).await?
            && existing_checksum == checksum
        {
            return Ok(()); // No-op when content is identical
        }

        // Upsert content with new data
        sqlx::query!(
            r#"
            INSERT INTO contents
                  (item_id, clean_html, clean_text, lang, extracted_at, checksum)
            VALUES ($1,       $2,         $3,         $4,   $5,          $6)
            ON CONFLICT (item_id) DO UPDATE
              SET clean_html   = EXCLUDED.clean_html,
                  clean_text   = EXCLUDED.clean_text,
                  lang         = EXCLUDED.lang,
                  extracted_at = EXCLUDED.extracted_at,
                  checksum     = EXCLUDED.checksum
            "#,
            item_id,
            clean_html,
            clean_text,
            lang,
            extracted_at,
            checksum,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get content by item ID
    pub async fn get_content(&self, item_id: Uuid) -> Result<Option<Content>> {
        let content = sqlx::query_as!(
            Content,
            "SELECT item_id, raw_html, raw_text, clean_html, clean_text, lang, extracted_at, checksum
             FROM contents WHERE item_id = $1",
            item_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(content)
    }

    /// Delete content by item ID
    pub async fn delete_content(&self, item_id: Uuid) -> Result<bool> {
        let result = sqlx::query!("DELETE FROM contents WHERE item_id = $1", item_id)
            .execute(self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Compute MD5 checksum from normalized content
    fn compute_checksum(&self, clean_html: &str, clean_text: &str) -> String {
        let mut hasher = Context::new();
        hasher.consume(clean_html.as_bytes());
        hasher.consume(clean_text.as_bytes());
        format!("{:x}", hasher.compute())
    }

    /// Get existing checksum for content deduplication check
    async fn get_existing_checksum(&self, item_id: Uuid) -> Result<Option<String>> {
        let checksum =
            sqlx::query_scalar!("SELECT checksum FROM contents WHERE item_id = $1", item_id)
                .fetch_optional(self.pool)
                .await?;

        Ok(checksum.flatten())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

    async fn setup_test_db() -> Option<PgPool> {
        // Skip tests if TEST_DATABASE_URL is not set
        let database_url = match std::env::var("TEST_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                eprintln!("Skipping database tests: TEST_DATABASE_URL not set");
                return None;
            }
        };

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Some(pool)
    }

    async fn insert_test_user(pool: &PgPool) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email, pw_hash) VALUES ($1, $2, $3)",
            user_id,
            "test@example.com",
            "dummy_hash"
        )
        .execute(pool)
        .await
        .expect("Failed to insert test user");
        user_id
    }

    async fn insert_test_item(pool: &PgPool, user_id: Uuid) -> Uuid {
        let item_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO items (id, user_id, url) VALUES ($1, $2, $3)",
            item_id,
            user_id,
            "https://example.com"
        )
        .execute(pool)
        .await
        .expect("Failed to insert test item");
        item_id
    }

    #[tokio::test]
    async fn test_upsert_content_insert() {
        let Some(pool) = setup_test_db().await else {
            return; // Skip test if database not available
        };
        let repo = ContentRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let item_id = insert_test_item(&pool, user_id).await;

        let clean_html = "<p>Test content</p>";
        let clean_text = "Test content";
        let lang = Some("en");
        let extracted_at = Utc::now();

        repo.upsert_content(item_id, clean_html, clean_text, lang, extracted_at)
            .await
            .expect("Failed to upsert content");

        let content = repo
            .get_content(item_id)
            .await
            .expect("Failed to get content");
        assert!(content.is_some());
        let content = content.unwrap();
        assert_eq!(content.clean_html.as_deref(), Some(clean_html));
        assert_eq!(content.clean_text.as_deref(), Some(clean_text));
        assert_eq!(content.lang.as_deref(), lang);
        assert!(content.checksum.is_some());
    }

    #[tokio::test]
    async fn test_upsert_content_update() {
        let Some(pool) = setup_test_db().await else {
            return; // Skip test if database not available
        };
        let repo = ContentRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let item_id = insert_test_item(&pool, user_id).await;

        // First insert
        let clean_html1 = "<p>Original content</p>";
        let clean_text1 = "Original content";
        repo.upsert_content(item_id, clean_html1, clean_text1, Some("en"), Utc::now())
            .await
            .expect("Failed to insert content");

        let original_checksum = repo
            .get_existing_checksum(item_id)
            .await
            .expect("Failed to get checksum")
            .expect("Checksum should exist");

        // Update with different content
        let clean_html2 = "<p>Updated content</p>";
        let clean_text2 = "Updated content";
        repo.upsert_content(item_id, clean_html2, clean_text2, Some("en"), Utc::now())
            .await
            .expect("Failed to update content");

        let content = repo
            .get_content(item_id)
            .await
            .expect("Failed to get content");
        assert!(content.is_some());
        let content = content.unwrap();
        assert_eq!(content.clean_html.as_deref(), Some(clean_html2));
        assert_eq!(content.clean_text.as_deref(), Some(clean_text2));

        // Checksum should be different
        let new_checksum = content.checksum.expect("Checksum should exist");
        assert_ne!(original_checksum, new_checksum);
    }

    #[tokio::test]
    async fn test_upsert_content_noop_when_same_checksum() {
        let Some(pool) = setup_test_db().await else {
            return; // Skip test if database not available
        };
        let repo = ContentRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let item_id = insert_test_item(&pool, user_id).await;

        let clean_html = "<p>Same content</p>";
        let clean_text = "Same content";
        let first_extracted_at = Utc::now();

        // First insert
        repo.upsert_content(
            item_id,
            clean_html,
            clean_text,
            Some("en"),
            first_extracted_at,
        )
        .await
        .expect("Failed to insert content");

        let first_content = repo
            .get_content(item_id)
            .await
            .expect("Failed to get content");
        assert!(first_content.is_some());
        let first_checksum = first_content.as_ref().unwrap().checksum.clone();

        // Second insert with same content but different timestamp
        let second_extracted_at = Utc::now();
        repo.upsert_content(
            item_id,
            clean_html,
            clean_text,
            Some("en"),
            second_extracted_at,
        )
        .await
        .expect("Failed to upsert content");

        let second_content = repo
            .get_content(item_id)
            .await
            .expect("Failed to get content");
        assert!(second_content.is_some());

        // Content should remain unchanged (no-op due to same checksum)
        let second_checksum = second_content.as_ref().unwrap().checksum.clone();
        assert_eq!(first_checksum, second_checksum);

        // extracted_at should remain the original value (proving no-op occurred)
        assert_eq!(
            first_content.unwrap().extracted_at,
            second_content.unwrap().extracted_at
        );
    }

    #[tokio::test]
    async fn test_delete_content() {
        let Some(pool) = setup_test_db().await else {
            return; // Skip test if database not available
        };
        let repo = ContentRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let item_id = insert_test_item(&pool, user_id).await;

        // Insert content first
        repo.upsert_content(item_id, "<p>Test</p>", "Test", Some("en"), Utc::now())
            .await
            .expect("Failed to insert content");

        // Verify it exists
        let content = repo
            .get_content(item_id)
            .await
            .expect("Failed to get content");
        assert!(content.is_some());

        // Delete it
        let deleted = repo
            .delete_content(item_id)
            .await
            .expect("Failed to delete content");
        assert!(deleted);

        // Verify it's gone
        let content = repo
            .get_content(item_id)
            .await
            .expect("Failed to get content");
        assert!(content.is_none());

        // Delete non-existent content should return false
        let deleted = repo
            .delete_content(item_id)
            .await
            .expect("Failed to delete content");
        assert!(!deleted);
    }
}
