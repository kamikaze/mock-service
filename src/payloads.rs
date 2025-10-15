use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::fs;

pub async fn load_payloads() -> HashMap<String, HashMap<String, Vec<u8>>> {
    let dir_path = "payloads/";
    let mut result = HashMap::new();

    // Read the top-level directory (get, post, put, etc.)
    let Ok(mut entries) = fs::read_dir(dir_path).await else {
        return result;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();

        if path.is_dir() {
            let method_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Load all files under this method directory
            let method_payloads = load_method_directory(path, String::new()).await;
            result.insert(method_name, method_payloads);
        }
    }

    result
}

fn load_method_directory(
    dir_path: PathBuf,
    current_path: String,
) -> Pin<Box<dyn Future<Output = HashMap<String, Vec<u8>>> + Send>> {
    Box::pin(async move {
        let mut result = HashMap::new();

        let Ok(mut entries) = fs::read_dir(&dir_path).await else {
            return result;
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            if path.is_dir() {
                // Build the new path segment
                let new_path = if current_path.is_empty() {
                    format!("/{name}")
                } else {
                    format!("{current_path}/{name}")
                };

                // Recursively load subdirectory
                let subdir_payloads = load_method_directory(path, new_path).await;
                result.extend(subdir_payloads);
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Get filename without extension
                let filename = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(name)
                    .to_string();

                // Build the full key path
                let key = format!("{current_path}/{filename}");

                // Read file as binary data
                match fs::read(&path).await {
                    Ok(bytes) => {
                        result.insert(key, bytes);
                    }
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", path.display(), e);
                    }
                }
            }
        }

        result
    })
}

// Example usage:
// let payloads = load_payloads().await;
// let get_payload: &Vec<u8> = &payloads["get"]["/v1/users/endpoint/00001"];
// let post_payload: &Vec<u8> = &payloads["post"]["/endpoint/00001"];
