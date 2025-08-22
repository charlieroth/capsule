use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::entities::ItemStatus;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateItemRequest {
    pub url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateItemRequest {
    pub title: Option<String>,
    pub status: Option<ItemStatus>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ItemResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub site: Option<String>,
    pub status: ItemStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ItemListResponse {
    pub items: Vec<ItemResponse>,
}

impl CreateItemRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("URL cannot be empty".to_string());
        }
        if self.url.len() > 2048 {
            return Err("URL too long".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_item_request_valid() {
        let request = CreateItemRequest {
            url: "https://example.com".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_item_request_empty_url() {
        let request = CreateItemRequest {
            url: "".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_item_request_url_too_long() {
        let request = CreateItemRequest {
            url: "a".repeat(2049),
        };
        assert!(request.validate().is_err());
    }
}
