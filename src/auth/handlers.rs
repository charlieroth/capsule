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
