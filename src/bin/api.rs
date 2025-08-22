use axum::{
    Router,
    extract::State,
    middleware::from_fn_with_state,
    routing::{get, post, patch},
};
use capsule::{
    app_state::AppState,
    auth::handlers,
    config,
    items,
    middleware::rate_limit::{RateLimit, rate_limit_middleware},
};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = config::Config::from_env().expect("Failed to load configuration");

    let pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .connect(&config.database_url())
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
        .nest("/v1/auth", auth_routes)
        .nest("/v1/items", item_routes)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr())
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app).await.unwrap();
}

async fn root(State(_state): State<AppState>) -> &'static str {
    "Hello from capsule!"
}
