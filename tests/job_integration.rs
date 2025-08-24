use chrono::Utc;
use serde_json::json;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use capsule::{entities::JobStatus, jobs::JobRepository};

/// Test that basic job repository operations work correctly
#[sqlx::test]
async fn test_job_enqueue_and_fetch(pool: Pool<Postgres>) {
    // Test enqueuing a job
    let job_id = JobRepository::enqueue(&pool, "test_job", json!({"test": "data"}), None, None)
        .await
        .expect("Failed to enqueue job");

    // Verify job was created correctly
    let job = sqlx::query!(
        "SELECT kind, payload, status::text as status, attempts FROM jobs WHERE id = $1",
        job_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch created job");

    assert_eq!(job.kind, "test_job");
    assert_eq!(job.payload, json!({"test": "data"}));
    assert_eq!(job.status, Some("queued".to_string()));
    assert_eq!(job.attempts, 0);

    // Test fetching due jobs
    let worker_id = Uuid::new_v4();
    let jobs = JobRepository::fetch_due_jobs(&pool, 10, worker_id, 300)
        .await
        .expect("Failed to fetch due jobs");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job_id);
    assert_eq!(jobs[0].status, JobStatus::Running);
    assert_eq!(jobs[0].reserved_by, Some(worker_id));
    assert!(jobs[0].visibility_till.is_some());
}

/// Test job success marking
#[sqlx::test]
async fn test_job_success(pool: Pool<Postgres>) {
    // Enqueue a job
    let job_id = JobRepository::enqueue(&pool, "test_job", json!({"test": "data"}), None, None)
        .await
        .expect("Failed to enqueue job");

    // Mark it as successful
    JobRepository::mark_success(&pool, job_id)
        .await
        .expect("Failed to mark job as successful");

    // Verify the status
    let job = sqlx::query!(
        "SELECT status::text as status, reserved_by, visibility_till FROM jobs WHERE id = $1",
        job_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch job after success");

    assert_eq!(job.status, Some("succeeded".to_string()));
    assert!(job.reserved_by.is_none());
    assert!(job.visibility_till.is_none());
}

/// Test job failure marking with retry
#[sqlx::test]
async fn test_job_failure_with_retry(pool: Pool<Postgres>) {
    // Enqueue a job
    let job_id = JobRepository::enqueue(&pool, "test_job", json!({"test": "data"}), None, Some(3))
        .await
        .expect("Failed to enqueue job");

    // Mark it as failed with retry
    let next_run_at = Utc::now() + chrono::Duration::minutes(5);
    JobRepository::mark_failure(&pool, job_id, "Test error", Some(next_run_at), 60)
        .await
        .expect("Failed to mark job as failed");

    // Verify the status
    let job = sqlx::query!(
        "SELECT status::text as status, attempts, last_error, backoff_seconds FROM jobs WHERE id = $1",
        job_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch job after failure");

    assert_eq!(job.status, Some("queued".to_string())); // Should be queued for retry
    assert_eq!(job.attempts, 1);
    assert_eq!(job.last_error, Some("Test error".to_string()));
    assert_eq!(job.backoff_seconds, 60);
}

/// Test job failure marking without retry (permanent failure)
#[sqlx::test]
async fn test_job_permanent_failure(pool: Pool<Postgres>) {
    // Enqueue a job
    let job_id = JobRepository::enqueue(
        &pool,
        "test_job",
        json!({"test": "data"}),
        None,
        Some(1), // Only 1 attempt
    )
    .await
    .expect("Failed to enqueue job");

    // Mark it as failed without retry (permanent failure)
    JobRepository::mark_failure(
        &pool,
        job_id,
        "Permanent error",
        None, // No next run time = permanent failure
        0,
    )
    .await
    .expect("Failed to mark job as permanently failed");

    // Verify the status
    let job = sqlx::query!(
        "SELECT status::text as status, attempts, last_error FROM jobs WHERE id = $1",
        job_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch job after permanent failure");

    assert_eq!(job.status, Some("failed".to_string()));
    assert_eq!(job.attempts, 1);
    assert_eq!(job.last_error, Some("Permanent error".to_string()));
}

/// Test job visibility timeout behavior
#[sqlx::test]
async fn test_job_visibility_timeout(pool: Pool<Postgres>) {
    // Enqueue a job
    let job_id = JobRepository::enqueue(&pool, "test_job", json!({"test": "data"}), None, None)
        .await
        .expect("Failed to enqueue job");

    // Fetch it with a short visibility timeout
    let worker_id = Uuid::new_v4();
    let jobs = JobRepository::fetch_due_jobs(&pool, 1, worker_id, 1)
        .await // 1 second timeout
        .expect("Failed to fetch due jobs");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, JobStatus::Running);

    // Wait for the visibility timeout to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Try to fetch again with a different worker - should succeed
    let worker_id_2 = Uuid::new_v4();
    let jobs = JobRepository::fetch_due_jobs(&pool, 1, worker_id_2, 300)
        .await
        .expect("Failed to fetch due jobs after timeout");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job_id);
    assert_eq!(jobs[0].reserved_by, Some(worker_id_2)); // Should be reserved by new worker
}

/// Test that multiple jobs can be fetched and processed
#[sqlx::test]
async fn test_multiple_job_processing(pool: Pool<Postgres>) {
    // Enqueue multiple jobs
    let mut job_ids = Vec::new();
    for i in 0..5 {
        let job_id = JobRepository::enqueue(&pool, "test_job", json!({"index": i}), None, None)
            .await
            .expect("Failed to enqueue job");
        job_ids.push(job_id);
    }

    // Fetch all jobs at once
    let worker_id = Uuid::new_v4();
    let jobs = JobRepository::fetch_due_jobs(&pool, 10, worker_id, 300)
        .await
        .expect("Failed to fetch due jobs");

    assert_eq!(jobs.len(), 5);
    for job in &jobs {
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.reserved_by, Some(worker_id));
        assert!(job_ids.contains(&job.id));
    }

    // Mark all jobs as successful
    for job in jobs {
        JobRepository::mark_success(&pool, job.id)
            .await
            .expect("Failed to mark job as successful");
    }

    // Verify all are marked as succeeded
    for job_id in job_ids {
        let job = sqlx::query!(
            "SELECT status::text as status FROM jobs WHERE id = $1",
            job_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch job");

        assert_eq!(job.status, Some("succeeded".to_string()));
    }
}
