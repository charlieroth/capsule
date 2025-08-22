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

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "job_kind", rename_all = "snake_case")]
pub enum JobKind {
    FetchAndExtract,
    ReindexItem,
    DeleteItem,
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Done,
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
    pub kind: JobKind,
    pub item_id: Option<Uuid>, // FK -> items.id
    pub status: JobStatus,
    pub run_at: DateTime<Utc>,
    pub attempts: i32,
    pub last_error: Option<String>,
}
