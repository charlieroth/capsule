use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// --- PostgreSQL Enums ---
#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[sqlx(type_name = "item_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Pending,
    Fetched,
    Archived,
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

/// --- Tables ---

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub pw_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Item {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub site: Option<String>,
    pub status: ItemStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Content {
    pub item_id: Uuid, // PK and FK -> items.id
    pub html: Option<String>,
    pub text: Option<String>,
    pub lang: Option<String>,
    pub extracted_at: Option<DateTime<Utc>>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ItemTag {
    pub item_id: Uuid, // PK and FK -> items.id
    pub tag_id: Uuid,  // PK and FK -> tags.id
}

#[derive(Debug, Clone, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub kind: String,               // logical job name
    pub payload: serde_json::Value, // job data as JSONB
    pub run_at: DateTime<Utc>,      // next time the job is eligible
    pub attempts: i32,              // execution attempts so far
    pub max_attempts: i32,          // maximum attempts before giving up
    pub backoff_seconds: i32,       // populated when job fails
    pub status: JobStatus,
    pub last_error: Option<String>,
    pub visibility_till: Option<DateTime<Utc>>, // set while "running"
    pub reserved_by: Option<Uuid>,              // worker instance id
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
