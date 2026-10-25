use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};
use std::{
    env,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::net::TcpListener;

// HTML page embedded into the binary at compile time
const INDEX_HTML: &str = include_str!("index.html");

// Source of the random pictures
const IMAGE_URL: &str = "https://picsum.photos/1200";

// How long the same picture is kept before fetching a new one
const IMAGE_MAX_AGE: Duration = Duration::from_secs(10 * 60);

// Shared state: where the picture is cached and whether a download is running
struct AppState {
    image_path: PathBuf,
    downloading: AtomicBool,
    client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    // Read the image path from the IMAGE_PATH env var, with a default inside the persistent volume
    let image_path = PathBuf::from(
        env::var("IMAGE_PATH").unwrap_or_else(|_| "/usr/src/app/images/image.jpg".to_string()),
    );

    // Make sure the parent directory exists before writing
    if let Some(dir) = image_path.parent() {
        tokio::fs::create_dir_all(dir)
            .await
            .expect("failed to create image directory");
    }

    let state = Arc::new(AppState {
        image_path,
        downloading: AtomicBool::new(false),
        client: reqwest::Client::new(),
    });

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET / refreshes the cached picture when needed and answers with the HTML page
    // GET /images/image.jpg only serves the picture already stored on disk
    let app = Router::new()
        .route("/", get(index))
        .route("/images/image.jpg", get(image))
        .with_state(state);

    // Listen on 0.0.0.0 (all interfaces) so the server is reachable inside a container
    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("failed to bind port");

    // Print the port actually opened, only after the bind succeeded
    let bound_port = listener.local_addr().expect("failed to read local address").port();
    println!("Server started in port {bound_port}");

    // Serve requests until the process is stopped
    axum::serve(listener, app).await.expect("server error");
}

async fn index(State(state): State<Arc<AppState>>) -> Html<&'static str> {
    // The file's modification time tells when the picture was downloaded, and it survives restarts
    let modified = tokio::fs::metadata(&state.image_path)
        .await
        .and_then(|metadata| metadata.modified());

    match modified {
        // No picture yet: download it before answering so the page has something to show
        Err(_) => {
            if !state.downloading.swap(true, Ordering::SeqCst) {
                if let Err(error) = download_image(&state).await {
                    eprintln!("failed to download image: {error}");
                }
                state.downloading.store(false, Ordering::SeqCst);
            }
        }
        // Expired picture: serve the old one this time and fetch the new one in the background
        Ok(modified) => {
            let age = modified.elapsed().unwrap_or_default();
            if age > IMAGE_MAX_AGE && !state.downloading.swap(true, Ordering::SeqCst) {
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(error) = download_image(&state).await {
                        eprintln!("failed to download image: {error}");
                    }
                    state.downloading.store(false, Ordering::SeqCst);
                });
            }
        }
    }

    Html(INDEX_HTML)
}

async fn image(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match tokio::fs::read(&state.image_path).await {
        Ok(bytes) => (
            StatusCode::OK,
            // no-cache makes the browser check again, so a new picture shows up after it changes
            [(header::CONTENT_TYPE, "image/jpeg"), (header::CACHE_CONTROL, "no-cache")],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn download_image(state: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Picsum answers with a redirect to the actual picture, which reqwest follows
    let bytes = state
        .client
        .get(IMAGE_URL)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    // Write to a temporary file and rename it, so a half-written picture is never served
    let tmp_path = state.image_path.with_extension("jpg.tmp");
    tokio::fs::write(&tmp_path, &bytes).await?;
    tokio::fs::rename(&tmp_path, &state.image_path).await?;

    println!("downloaded a new image to {}", state.image_path.display());
    Ok(())
}
