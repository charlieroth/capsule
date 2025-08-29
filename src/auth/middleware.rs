use axum::{
    Json,
    extract::{FromRequestParts, Request},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    auth::{dtos::ErrorResponse, jwt::JwtService},
    config::Config,
};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl AuthenticatedUser {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        async move {
            let auth_header = auth_header.ok_or(AuthError::MissingToken)?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or(AuthError::InvalidTokenFormat)?;

            let config = Config::from_env().map_err(|_| AuthError::InternalError)?;
            let jwt_service = JwtService::new(config.jwt_secret());

            let claims = jwt_service
                .verify_token(token)
                .map_err(|_| AuthError::InvalidToken)?;

            let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

            Ok(AuthenticatedUser::new(user_id))
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidTokenFormat,
    InvalidToken,
    InternalError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidTokenFormat => (StatusCode::UNAUTHORIZED, "Invalid token format"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            AuthError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (
            status,
            Json(ErrorResponse {
                error: message.to_string(),
            }),
        )
            .into_response()
    }
}

pub async fn auth_middleware(req: Request, next: Next) -> Response {
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app_state::AppState, config::Config, repositories::user::MockUserRepositoryTrait};
    use axum::{
        Json, Router,
        body::to_bytes,
        http::{Request, StatusCode, header::AUTHORIZATION},
        response::Json as ResponseJson,
        routing::get,
    };
    use serde_json::{Value, json};
    use sqlx::{Pool, Postgres};
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    fn create_test_pool() -> Pool<Postgres> {
        // Create a dummy pool for testing - won't actually be used
        Pool::<Postgres>::connect_lazy("postgresql://dummy").expect("Failed to create test pool")
    }

    async fn protected_handler(auth_user: AuthenticatedUser) -> ResponseJson<Value> {
        Json(json!({
            "user_id": auth_user.user_id,
            "message": "Access granted"
        }))
    }

    fn create_test_app() -> Router {
        let mock_repo = MockUserRepositoryTrait::new();
        let state = AppState {
            user_repo: Arc::new(mock_repo),
            db_pool: create_test_pool(),
        };

        Router::new()
            .route("/protected", get(protected_handler))
            .with_state(state)
    }

    fn create_jwt_token(user_id: Uuid) -> String {
        // Use the same config loading logic as the middleware
        let config = Config::from_env().expect("Failed to load config");
        let jwt_service = JwtService::new(config.jwt_secret());
        jwt_service
            .generate_token(user_id)
            .expect("Failed to generate token")
    }

    fn create_expired_jwt_token(user_id: Uuid) -> String {
        use crate::auth::jwt::Claims;
        use chrono::{Duration, Utc};
        use jsonwebtoken::{EncodingKey, Header, encode};

        // Use the same config loading logic as the middleware
        let config = Config::from_env().expect("Failed to load config");
        let encoding_key = EncodingKey::from_secret(config.jwt_secret().as_bytes());

        let now = Utc::now();
        let expired_time = now - Duration::hours(1);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expired_time.timestamp() as usize,
            iat: (expired_time - Duration::hours(24)).timestamp() as usize,
        };

        encode(&Header::default(), &claims, &encoding_key).expect("Failed to create expired token")
    }

    #[tokio::test]
    async fn test_missing_authorization_header() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/protected")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_malformed_authorization_header_no_bearer() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/protected")
            .header(AUTHORIZATION, "Basic dXNlcjpwYXNz")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_malformed_authorization_header_invalid_format() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/protected")
            .header(AUTHORIZATION, "Bearer")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_jwt_token() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/protected")
            .header(AUTHORIZATION, "Bearer invalid.jwt.token")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_expired_jwt_token() {
        let app = create_test_app();
        let user_id = Uuid::new_v4();
        let expired_token = create_expired_jwt_token(user_id);

        let request = Request::builder()
            .method("GET")
            .uri("/protected")
            .header(AUTHORIZATION, format!("Bearer {}", expired_token))
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_valid_jwt_token_success() {
        let app = create_test_app();
        let user_id = Uuid::new_v4();
        let token = create_jwt_token(user_id);

        let request = Request::builder()
            .method("GET")
            .uri("/protected")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_extractor_returns_correct_user_id() {
        let app = create_test_app();
        let user_id = Uuid::new_v4();
        let token = create_jwt_token(user_id);

        let request = Request::builder()
            .method("GET")
            .uri("/protected")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["user_id"], user_id.to_string());
        assert_eq!(json["message"], "Access granted");
    }
}
