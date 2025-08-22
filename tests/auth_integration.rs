mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use sqlx::{Pool, Postgres};
use tower::ServiceExt;

use capsule::auth::{
    dtos::{ErrorResponse, LoginResponse},
    jwt::JwtService,
};

#[sqlx::test]
async fn test_signup_success(pool: Pool<Postgres>) {
    let app = helpers::test_app(pool);

    let signup_body = json!({
        "email": "alice@example.com",
        "password": "CorrectHorseBatteryStaple123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(signup_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test]
async fn test_signup_duplicate_email(pool: Pool<Postgres>) {
    let app = helpers::test_app(pool);

    let signup_body = json!({
        "email": "alice@example.com",
        "password": "CorrectHorseBatteryStaple123"
    });

    // First signup should succeed
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(signup_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Second signup with same email should fail
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(signup_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(error_response.error, "User already exists");
}

#[sqlx::test]
async fn test_login_success(pool: Pool<Postgres>) {
    let app = helpers::test_app(pool);

    // First create a user
    let signup_body = json!({
        "email": "alice@example.com",
        "password": "CorrectHorseBatteryStaple123"
    });

    let signup_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(signup_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(signup_response.status(), StatusCode::CREATED);

    // Now login
    let login_body = json!({
        "email": "alice@example.com",
        "password": "CorrectHorseBatteryStaple123"
    });

    let login_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_response: LoginResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Verify JWT token is valid
    let jwt_service = JwtService::new("dev-secret-change-me");
    let claims = jwt_service.verify_token(&login_response.token).unwrap();
    assert!(!claims.sub.is_empty());
}

#[sqlx::test]
async fn test_login_invalid_credentials(pool: Pool<Postgres>) {
    let app = helpers::test_app(pool);

    let login_body = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(error_response.error, "Invalid credentials");
}
