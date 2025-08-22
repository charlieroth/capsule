use crate::repositories::UserRepository;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: UserRepository,
}

impl AppState {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            user_repo: UserRepository::new(pool),
        }
    }
}
