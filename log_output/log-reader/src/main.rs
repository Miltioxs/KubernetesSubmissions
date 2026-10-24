use axum::{Router, extract::State, routing::get};
use std::{env, sync::Arc};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Path of the file written by log-writer, shared through a volume
    let log_file = Arc::new(
        env::var("LOG_FILE").unwrap_or_else(|_| "/usr/src/app/files/log.txt".to_string()),
    );

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /status answers with the current content of the file
    let app = Router::new()
        .route(
            "/status",
            get(|State(log_file): State<Arc<String>>| async move {
                // Read the file on every request so the latest lines are returned
                tokio::fs::read_to_string(log_file.as_str())
                    .await
                    .unwrap_or_else(|_| "log file not available yet".to_string())
            }),
        )
        .with_state(log_file);

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
