use axum::{
    Router,
    extract::State,
    middleware::from_fn_with_state,
    routing::{get, patch, post},
};
use capsule::{
    app_state::AppState,
    auth::{
        dtos::{ErrorResponse, LoginRequest, LoginResponse, SignupRequest},
        handlers,
    },
    config,
    entities::ItemStatus,
    health, items,
    items::dtos::{CreateItemRequest, ItemResponse, UpdateItemRequest},
    middleware::rate_limit::{RateLimit, rate_limit_middleware},
};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        handlers::signup,
        handlers::login,
        items::handlers::list_items,
        items::handlers::create_item,
        items::handlers::get_item,
        items::handlers::update_item,
    ),
    components(
        schemas(
            health::HealthResponse,
            SignupRequest,
            LoginRequest,
            LoginResponse,
            ErrorResponse,
            CreateItemRequest,
            UpdateItemRequest,
            ItemResponse,
            ItemStatus,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "items", description = "Item management endpoints")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "capsule=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = config::Config::from_env().expect("Failed to load configuration");

    let pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .connect(config.database_url())
        .await
        .unwrap();

    let app_state = AppState::new(pool);
    let rate_limit = RateLimit::new(10, 60); // 10 requests per minute

    let auth_routes = Router::new()
        .route("/signup", post(handlers::signup))
        .route("/login", post(handlers::login))
        .layer(from_fn_with_state(rate_limit, rate_limit_middleware));

    let item_routes = Router::new()
        .route("/", get(items::handlers::list_items))
        .route("/", post(items::handlers::create_item))
        .route("/{id}", get(items::handlers::get_item))
        .route("/{id}", patch(items::handlers::update_item));

    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(health::health_check))
        .nest("/v1/auth", auth_routes)
        .nest("/v1/items", item_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr())
        .await
        .expect("Failed to bind to address");

    info!("Server starting on {}", config.bind_addr());
    axum::serve(listener, app).await.unwrap();
}

async fn root(State(_state): State<AppState>) -> &'static str {
    "Hello from capsule!"
}
