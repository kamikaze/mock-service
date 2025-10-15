mod payloads;

use crate::payloads::load_payloads;
use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::error::Error;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Shared state that holds the preloaded JSON
#[derive(Clone)]
struct AppState {
    payloads: HashMap<String, HashMap<String, Vec<u8>>>,
}

async fn serve_mock(State(state): State<AppState>, req: Request) -> Response {
    let method = req.method().as_str().to_lowercase();
    let uri = req.uri().path().to_lowercase();

    // Try to find the payload
    let payload = state
        .payloads
        .get(&method)
        .and_then(|method_endpoints| method_endpoints.get(&uri));

    match payload {
        Some(data) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );

            (StatusCode::OK, headers, data.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Mock not found").into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let payloads = load_payloads().await;

    println!("Loaded payloads:");
    for (method, paths) in &payloads {
        println!("  {}: {} endpoints", method.to_uppercase(), paths.len());
        for path in paths.keys() {
            println!("    {path}");
        }
    }

    let state = AppState { payloads };

    // Build the router with a catch-all route
    let app = Router::new().fallback(serve_mock).with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    println!("\nServer running on http://0.0.0.0:8000");

    axum::serve(listener, app).await?;

    Ok(())
}
