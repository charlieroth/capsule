use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::{dtos::ErrorResponse, middleware::AuthenticatedUser},
};

pub async fn list_items(_auth_user: AuthenticatedUser, State(_state): State<AppState>) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented".to_string(),
        }),
    )
        .into_response()
}

pub async fn create_item(
    _auth_user: AuthenticatedUser,
    State(_state): State<AppState>,
    Json(_payload): Json<serde_json::Value>,
) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented".to_string(),
        }),
    )
        .into_response()
}

pub async fn get_item(
    _auth_user: AuthenticatedUser,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented".to_string(),
        }),
    )
        .into_response()
}

pub async fn update_item(
    _auth_user: AuthenticatedUser,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Not implemented".to_string(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::jwt::JwtService, config::Config, repositories::user::MockUserRepositoryTrait,
    };
    use axum::{
        Router,
        body::Body,
        http::{Request, header::AUTHORIZATION},
        routing::{get, patch, post},
    };
    use sqlx::{Pool, Postgres};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn create_test_pool() -> Pool<Postgres> {
        // Create a dummy pool for testing - won't actually be used
        Pool::<Postgres>::connect_lazy("postgresql://dummy").expect("Failed to create test pool")
    }

    fn create_test_app() -> Router {
        let mock_repo = MockUserRepositoryTrait::new();
        let state = AppState {
            user_repo: Arc::new(mock_repo),
            db_pool: create_test_pool(),
        };

        Router::new()
            .route("/items", get(list_items))
            .route("/items", post(create_item))
            .route("/items/{id}", get(get_item))
            .route("/items/{id}", patch(update_item))
            .with_state(state)
    }

    fn create_jwt_token(user_id: Uuid) -> String {
        let config = Config::from_env().expect("Failed to load config");
        let jwt_service = JwtService::new(config.jwt_secret());
        jwt_service
            .generate_token(user_id)
            .expect("Failed to generate token")
    }

    #[tokio::test]
    async fn test_items_routes_require_authentication() {
        let app = create_test_app();
        let user_id = Uuid::new_v4();
        let token = create_jwt_token(user_id);

        // Test GET /items
        let request = Request::builder()
            .method("GET")
            .uri("/items")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);

        // Test POST /items
        let request = Request::builder()
            .method("POST")
            .uri("/items")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn test_items_routes_reject_unauthorized() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/items")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
