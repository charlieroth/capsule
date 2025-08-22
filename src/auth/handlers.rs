use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    app_state::AppState,
    auth::{
        dtos::{ErrorResponse, LoginRequest, LoginResponse, SignupRequest},
        jwt::JwtService,
    },
    config::Config,
    passwords::Passwords,
};

pub async fn signup(State(state): State<AppState>, Json(payload): Json<SignupRequest>) -> Response {
    if let Err(error) = payload.validate() {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })).into_response();
    }

    // Check if user already exists
    match state.user_repo.find_by_email(&payload.email).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "User already exists".to_string(),
                }),
            )
                .into_response();
        }
        Ok(None) => {} // User doesn't exist, continue
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
                .into_response();
        }
    }

    // Hash password
    let passwords = Passwords::new(65536, 2, 1);
    let pw_hash = match passwords.hash(&payload.password) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to hash password".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Create user
    match state.user_repo.create(&payload.email, &pw_hash).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create user".to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    if let Err(error) = payload.validate() {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })).into_response();
    }

    // Find user by email
    let user = match state.user_repo.find_by_email(&payload.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid credentials".to_string(),
                }),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Verify password
    let passwords = Passwords::new(65536, 2, 1);
    let (is_valid, _needs_rehash) = match passwords.verify(&payload.password, &user.pw_hash) {
        Ok(result) => result,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Password verification failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    if !is_valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        )
            .into_response();
    }

    // Generate JWT token
    let config = Config::from_env().expect("Failed to load config");
    let jwt_service = JwtService::new(config.jwt_secret());
    let token = match jwt_service.generate_token(user.id) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to generate token".to_string(),
                }),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(LoginResponse { token })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::user::MockUserRepositoryTrait;
    use axum::{body::Body, http::Request};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_signup_database_error_on_find() {
        let mut mock_repo = MockUserRepositoryTrait::new();
        mock_repo
            .expect_find_by_email()
            .returning(|_| Err(anyhow::anyhow!("Database connection failed")));

        let state = AppState {
            user_repo: Arc::new(mock_repo),
        };

        let app = axum::Router::new()
            .route("/signup", axum::routing::post(signup))
            .with_state(state);

        let request = Request::builder()
            .method("POST")
            .uri("/signup")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "email": "test@example.com",
                    "password": "validpassword123"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_signup_database_error_on_create() {
        let mut mock_repo = MockUserRepositoryTrait::new();
        mock_repo.expect_find_by_email().returning(|_| Ok(None));
        mock_repo
            .expect_create()
            .returning(|_, _| Err(anyhow::anyhow!("Database insert failed")));

        let state = AppState {
            user_repo: Arc::new(mock_repo),
        };

        let app = axum::Router::new()
            .route("/signup", axum::routing::post(signup))
            .with_state(state);

        let request = Request::builder()
            .method("POST")
            .uri("/signup")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "email": "test@example.com",
                    "password": "validpassword123"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_login_database_error() {
        let mut mock_repo = MockUserRepositoryTrait::new();
        mock_repo
            .expect_find_by_email()
            .returning(|_| Err(anyhow::anyhow!("Database connection failed")));

        let state = AppState {
            user_repo: Arc::new(mock_repo),
        };

        let app = axum::Router::new()
            .route("/login", axum::routing::post(login))
            .with_state(state);

        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "email": "test@example.com",
                    "password": "anypassword"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
