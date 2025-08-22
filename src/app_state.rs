use crate::repositories::{UserRepository, UserRepositoryTrait};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub user_repo: Arc<dyn UserRepositoryTrait + Send + Sync>,
    pub db_pool: Pool<Postgres>,
}

impl AppState {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            user_repo: Arc::new(UserRepository::new(pool.clone())),
            db_pool: pool,
        }
    }
}
