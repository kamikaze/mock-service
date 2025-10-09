use axum::{
    routing::get,
    Router,
    Json,
    response::IntoResponse,
    extract::State,
};
use serde_json::Value;
use std::{error::Error, sync::Arc};
use tokio::fs;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Shared state that holds the preloaded JSON
#[derive(Clone)]
struct AppState {
    json_data: Arc<Value>,
}

async fn serve_json(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.json_data.as_ref().clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Preload JSON once at startup
    let json_content = fs::read_to_string("payloads/00001.json")
        .await
        .expect("Failed to read JSON file");

    let json_value: Value = serde_json::from_str(&json_content)
        .expect("Failed to parse JSON");

    // Wrap in Arc for efficient sharing across requests
    let state = AppState {
        json_data: Arc::new(json_value),
    };

    // Build the router with shared state
    let app = Router::new()
        .route("/api/mock/00001", get(serve_json))
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await?;

    println!("Server running on http://0.0.0.0:8000");

    axum::serve(listener, app).await?;

    Ok(())
}
