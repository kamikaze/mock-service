use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde_json::Value;
use std::{error::Error, sync::Arc};
use tokio::fs;

mod crypto;

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
    // Read private key from file
    let private_key_pem = fs::read("private_key.pem")
        .await
        .expect("Failed to read private key");

    // Read encrypted AES key
    let encrypted_aes_key = fs::read("payloads/00001.json.key")
        .await
        .expect("Failed to read encrypted AES key");

    // Read encrypted JSON file
    let encrypted_payload = fs::read("payloads/00001.json.enc")
        .await
        .expect("Failed to read encrypted JSON file");

    // Decrypt the data using AES key
    let decrypted_payload =
        crypto::decrypt_data_with_aes(&encrypted_payload, &encrypted_aes_key, &private_key_pem)?;

    // Parse JSON
    let json_content = String::from_utf8(decrypted_payload)?;
    let json_value: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON");

    // Wrap in Arc for efficient sharing across requests
    let state = AppState {
        json_data: Arc::new(json_value),
    };

    // Build the router with shared state
    let app = Router::new()
        .route("/api/mock/00001", get(serve_json))
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    println!("Server running on http://0.0.0.0:8000");

    axum::serve(listener, app).await?;

    Ok(())
}
