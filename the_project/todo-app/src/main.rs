use axum::{
    Form, Router,
    extract::State,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
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

// Keeps the page from hanging when todo-backend does not answer
const BACKEND_TIMEOUT: Duration = Duration::from_secs(2);

// Place in index.html where the todo list is rendered
const TODOS_MARKER: &str = "<!-- TODOS -->";

// Shared state: where the picture is cached, whether a download is running and where todo-backend is
struct AppState {
    image_path: PathBuf,
    downloading: AtomicBool,
    client: reqwest::Client,
    backend_url: String,
}

// Todo as returned by todo-backend; only the content is shown
#[derive(Deserialize)]
struct Todo {
    content: String,
}

// Received from the form and forwarded to todo-backend as JSON
#[derive(Deserialize, Serialize)]
struct NewTodo {
    content: String,
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
        // todo-backend is reached pod to pod through its Service, it is not exposed by the Ingress
        backend_url: env::var("TODO_BACKEND_URL")
            .unwrap_or_else(|_| "http://todo-backend-service".to_string()),
    });

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET / refreshes the cached picture when needed and answers with the HTML page and the todos
    // GET /images/image.jpg only serves the picture already stored on disk
    // POST /todos forwards the form to todo-backend and goes back to the page
    let app = Router::new()
        .route("/", get(index))
        .route("/images/image.jpg", get(image))
        .route("/todos", post(create_todo))
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

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
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

    // Render the todos into the page; it still loads when todo-backend is down
    let todos_html = match fetch_todos(&state).await {
        Ok(todos) => todos
            .iter()
            .map(|todo| format!("<li>{}</li>", escape_html(&todo.content)))
            .collect::<String>(),
        Err(error) => {
            eprintln!("failed to fetch todos: {error}");
            "<li>Todos unavailable</li>".to_string()
        }
    };

    Html(INDEX_HTML.replace(TODOS_MARKER, &todos_html))
}

async fn create_todo(State(state): State<Arc<AppState>>, Form(new_todo): Form<NewTodo>) -> Redirect {
    let result = state
        .client
        .post(format!("{}/todos", state.backend_url))
        .json(&new_todo)
        .timeout(BACKEND_TIMEOUT)
        .send()
        .await
        .and_then(|response| response.error_for_status());
    if let Err(error) = result {
        eprintln!("failed to create todo: {error}");
    }

    // 303 See Other makes the browser load the page again with GET
    Redirect::to("/")
}

async fn fetch_todos(state: &AppState) -> Result<Vec<Todo>, reqwest::Error> {
    state
        .client
        .get(format!("{}/todos", state.backend_url))
        .timeout(BACKEND_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

// Todos are written by users, so they are escaped before going into the HTML
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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
