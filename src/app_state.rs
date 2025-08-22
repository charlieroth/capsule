use crate::repositories::{UserRepository, UserRepositoryTrait};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub user_repo: Arc<dyn UserRepositoryTrait + Send + Sync>,
}

impl AppState {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            user_repo: Arc::new(UserRepository::new(pool)),
        }
    }
}
