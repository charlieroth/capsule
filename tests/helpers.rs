use axum::{Router, routing::post};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use capsule::{
    app_state::AppState,
    auth::handlers::{login, signup},
    repositories::{UserRepository, UserRepositoryTrait},
};

pub fn test_app(pool: Pool<Postgres>) -> Router {
    let user_repo: Arc<dyn UserRepositoryTrait + Send + Sync> =
        Arc::new(UserRepository::new(pool.clone()));
    let state = AppState {
        user_repo,
        db_pool: pool,
    };

    Router::new()
        .route("/v1/auth/signup", post(signup))
        .route("/v1/auth/login", post(login))
        .with_state(state)
}
