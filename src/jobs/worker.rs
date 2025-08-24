use crate::jobs::{JobRegistry, JobRepository, calculate_backoff_delay};
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tokio::{
    signal,
    sync::{Semaphore, mpsc},
    time::{interval, sleep},
};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, debug, error, info, info_span, warn};
use uuid::Uuid;

/// Worker configuration
#[derive(Clone)]
pub struct WorkerConfig {
    pub concurrency: usize,
    pub poll_interval_ms: u64,
    pub visibility_timeout_secs: i64,
    pub base_backoff_secs: u32,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            concurrency: 4,
            poll_interval_ms: 1000,
            visibility_timeout_secs: 300, // 5 minutes
            base_backoff_secs: 30,
        }
    }
}

/// Main worker supervisor that orchestrates job processing
pub struct WorkerSupervisor {
    pool: PgPool,
    registry: Arc<JobRegistry>,
    config: WorkerConfig,
    worker_id: Uuid,
    shutdown_token: CancellationToken,
}

impl WorkerSupervisor {
    pub fn new(pool: PgPool, registry: JobRegistry, config: WorkerConfig) -> Self {
        Self {
            pool,
            registry: Arc::new(registry),
            config,
            worker_id: Uuid::new_v4(),
            shutdown_token: CancellationToken::new(),
        }
    }

    /// Start the worker supervisor
    pub async fn run(self) -> Result<()> {
        info!("Starting worker supervisor with ID: {}", self.worker_id);
        info!(
            "Configuration - concurrency: {}, poll_interval: {}ms, visibility_timeout: {}s",
            self.config.concurrency,
            self.config.poll_interval_ms,
            self.config.visibility_timeout_secs
        );

        // Create bounded channel for jobs
        let (job_sender, job_receiver) = mpsc::channel(self.config.concurrency * 2);

        // Semaphore to limit concurrent job processing
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));

        // Spawn shutdown handler
        let shutdown_token = self.shutdown_token.clone();
        tokio::spawn(async move {
            if let Err(e) = signal::ctrl_c().await {
                error!("Failed to listen for shutdown signal: {}", e);
                return;
            }
            info!("Received shutdown signal, initiating graceful shutdown...");
            shutdown_token.cancel();
        });

        // Spawn job fetcher
        let fetcher_handle = {
            let pool = self.pool.clone();
            let worker_id = self.worker_id;
            let config = self.config.clone();
            let shutdown_token = self.shutdown_token.clone();
            tokio::spawn(
                WorkerSupervisor::run_fetcher_static(
                    pool,
                    worker_id,
                    config,
                    job_sender,
                    shutdown_token,
                )
                .instrument(info_span!("fetcher", worker_id = %worker_id)),
            )
        };

        // Spawn job processor
        let processor_handle = {
            let pool = self.pool.clone();
            let registry = self.registry.clone();
            let config = self.config.clone();
            let semaphore = semaphore.clone();
            let shutdown_token = self.shutdown_token.clone();
            tokio::spawn(
                WorkerSupervisor::run_processor_static(
                    pool,
                    registry,
                    config,
                    job_receiver,
                    semaphore,
                    shutdown_token,
                )
                .instrument(info_span!("processor", worker_id = %self.worker_id)),
            )
        };

        // Wait for shutdown signal
        self.shutdown_token.cancelled().await;
        info!("Shutdown initiated, waiting for tasks to complete...");

        // Wait for all permits to be available (all jobs completed)
        let _permits = semaphore
            .acquire_many(self.config.concurrency as u32)
            .await?;
        info!("All jobs completed, shutting down");

        // Wait for fetcher and processor to finish
        let _ = tokio::join!(fetcher_handle, processor_handle);

        Ok(())
    }

    /// Job fetching loop
    async fn run_fetcher_static(
        pool: PgPool,
        worker_id: Uuid,
        config: WorkerConfig,
        job_sender: mpsc::Sender<crate::entities::Job>,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        let mut poll_interval = interval(Duration::from_millis(config.poll_interval_ms));

        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => {
                    info!("Fetcher shutting down");
                    break;
                }
                _ = poll_interval.tick() => {
                    match JobRepository::fetch_due_jobs(
                        &pool,
                        config.concurrency as i64,
                        worker_id,
                        config.visibility_timeout_secs,
                    )
                    .await
                    {
                        Ok(jobs) => {
                            debug!("Fetched {} jobs", jobs.len());
                            for job in jobs {
                                if job_sender.send(job).await.is_err() {
                                    warn!("Job receiver dropped, stopping fetcher");
                                    return Ok(());
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to fetch jobs: {}", e);
                            // Brief pause on error to avoid tight loop
                            sleep(Duration::from_millis(1000)).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Job processing loop
    async fn run_processor_static(
        pool: PgPool,
        registry: Arc<JobRegistry>,
        config: WorkerConfig,
        mut job_receiver: mpsc::Receiver<crate::entities::Job>,
        semaphore: Arc<Semaphore>,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        while let Some(job) = tokio::select! {
            _ = shutdown_token.cancelled() => None,
            job = job_receiver.recv() => job,
        } {
            let permit = semaphore.clone().acquire_owned().await?;
            let pool = pool.clone();
            let registry = registry.clone();
            let config = config.clone();

            // Capture fields for tracing before moving job
            let job_id = job.id;
            let job_kind = job.kind.clone();
            let job_attempt = job.attempts;

            tokio::spawn(
                async move {
                    let _permit = permit; // Hold permit until job completes
                    Self::process_job(pool, registry, config, job).await;
                }
                .instrument(
                    info_span!("job", id = %job_id, kind = %job_kind, attempt = job_attempt),
                ),
            );
        }

        info!("Processor shutting down");
        Ok(())
    }

    /// Process a single job
    async fn process_job(
        pool: PgPool,
        registry: Arc<JobRegistry>,
        config: WorkerConfig,
        job: crate::entities::Job,
    ) {
        info!("Processing job {} (attempt {})", job.id, job.attempts + 1);

        let span = info_span!("job_execution", id = %job.id, kind = %job.kind);

        // Create handler for this job
        let handler = match registry.create_handler(&job.kind, job.payload.clone()) {
            Ok(handler) => handler,
            Err(e) => {
                error!("Failed to create handler for job {}: {}", job.id, e);
                let _ = JobRepository::mark_failure(
                    &pool,
                    job.id,
                    &format!("Failed to create handler: {}", e),
                    None,
                    0,
                )
                .await;
                return;
            }
        };

        // Execute the job
        let result = handler.run(job.payload.clone(), &pool, span.clone()).await;

        match result {
            Ok(()) => {
                info!("Job {} completed successfully", job.id);
                if let Err(e) = JobRepository::mark_success(&pool, job.id).await {
                    error!("Failed to mark job {} as successful: {}", job.id, e);
                }
            }
            Err(e) => {
                let attempt = job.attempts + 1;
                error!("Job {} failed (attempt {}): {}", job.id, attempt, e);

                // Determine if we should retry
                if attempt < job.max_attempts {
                    let backoff_delay = calculate_backoff_delay(attempt, config.base_backoff_secs);
                    let next_run_at =
                        Utc::now() + chrono::Duration::from_std(backoff_delay).unwrap();

                    info!(
                        "Job {} will retry in {} seconds (attempt {}/{})",
                        job.id,
                        backoff_delay.as_secs(),
                        attempt + 1,
                        job.max_attempts
                    );

                    if let Err(retry_err) = JobRepository::mark_failure(
                        &pool,
                        job.id,
                        &e.to_string(),
                        Some(next_run_at),
                        backoff_delay.as_secs() as i32,
                    )
                    .await
                    {
                        error!("Failed to schedule retry for job {}: {}", job.id, retry_err);
                    }
                } else {
                    info!(
                        "Job {} permanently failed after {} attempts",
                        job.id, attempt
                    );
                    if let Err(fail_err) =
                        JobRepository::mark_failure(&pool, job.id, &e.to_string(), None, 0).await
                    {
                        error!(
                            "Failed to mark job {} as permanently failed: {}",
                            job.id, fail_err
                        );
                    }
                }
            }
        }
    }
}
