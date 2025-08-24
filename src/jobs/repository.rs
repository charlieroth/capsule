use crate::entities::{Job, JobStatus};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub struct JobRepository;

impl JobRepository {
    /// Enqueue a new job
    pub async fn enqueue(
        pool: &PgPool,
        kind: &str,
        payload: Value,
        run_at: Option<DateTime<Utc>>,
        max_attempts: Option<i32>,
    ) -> Result<Uuid> {
        let run_at = run_at.unwrap_or_else(Utc::now);
        let max_attempts = max_attempts.unwrap_or(25);

        let result = sqlx::query!(
            r#"
            INSERT INTO jobs (kind, payload, run_at, max_attempts)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            kind,
            payload,
            run_at,
            max_attempts
        )
        .fetch_one(pool)
        .await?;

        Ok(result.id)
    }

    /// Fetch due jobs and reserve them for processing
    pub async fn fetch_due_jobs(
        pool: &PgPool,
        limit: i64,
        worker_id: Uuid,
        visibility_timeout_secs: i64,
    ) -> Result<Vec<Job>> {
        let visibility_till = Utc::now() + chrono::Duration::seconds(visibility_timeout_secs);

        let jobs = sqlx::query_as!(
            Job,
            r#"
            UPDATE jobs
            SET status = 'running'::job_status,
                visibility_till = $3,
                reserved_by = $2,
                updated_at = now()
            WHERE id IN (
                SELECT id
                FROM jobs
                WHERE (status = 'queued'::job_status OR 
                      (status = 'running'::job_status AND visibility_till < now()))
                  AND run_at <= now()
                ORDER BY run_at
                FOR UPDATE SKIP LOCKED
                LIMIT $1
            )
            RETURNING 
                id,
                kind,
                payload,
                run_at,
                attempts,
                max_attempts,
                backoff_seconds,
                status as "status: JobStatus",
                last_error,
                visibility_till,
                reserved_by,
                created_at,
                updated_at
            "#,
            limit,
            worker_id,
            visibility_till
        )
        .fetch_all(pool)
        .await?;

        Ok(jobs)
    }

    /// Mark job as succeeded
    pub async fn mark_success(pool: &PgPool, job_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = 'succeeded'::job_status,
                visibility_till = NULL,
                reserved_by = NULL,
                updated_at = now()
            WHERE id = $1
            "#,
            job_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark job as failed and schedule retry or mark as permanently failed
    pub async fn mark_failure(
        pool: &PgPool,
        job_id: Uuid,
        error_message: &str,
        next_run_at: Option<DateTime<Utc>>,
        backoff_seconds: i32,
    ) -> Result<()> {
        let (status, next_run) = if let Some(run_at) = next_run_at {
            (JobStatus::Queued, Some(run_at))
        } else {
            (JobStatus::Failed, None)
        };

        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $2,
                attempts = attempts + 1,
                last_error = $3,
                run_at = COALESCE($4, run_at),
                backoff_seconds = $5,
                visibility_till = NULL,
                reserved_by = NULL,
                updated_at = now()
            WHERE id = $1
            "#,
            job_id,
            status as JobStatus,
            error_message,
            next_run,
            backoff_seconds
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Extend visibility timeout for a running job
    pub async fn extend_visibility(
        pool: &PgPool,
        job_id: Uuid,
        visibility_timeout_secs: i64,
    ) -> Result<()> {
        let new_visibility_till = Utc::now() + chrono::Duration::seconds(visibility_timeout_secs);

        sqlx::query!(
            r#"
            UPDATE jobs
            SET visibility_till = $2,
                updated_at = now()
            WHERE id = $1 AND status = 'running'::job_status
            "#,
            job_id,
            new_visibility_till
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
