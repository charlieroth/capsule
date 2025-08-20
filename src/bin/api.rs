use axum::{Router, routing::get};
use capsule::config;

#[tokio::main]
async fn main() {
    let config = config::Config::from_env().expect("Failed to load configuration");
    let app = Router::new().route("/", get(|| async { "Hello from capsule!" }));
    let listener = tokio::net::TcpListener::bind(config.bind_addr())
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app).await.unwrap();
}
