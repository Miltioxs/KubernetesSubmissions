use axum::{Router, extract::State, routing::get};
use std::{env, sync::Arc};
use tokio::net::TcpListener;

// Paths of the files read on every request, both shared through a volume
struct Files {
    log_file: String,
    pingpong_file: String,
}

#[tokio::main]
async fn main() {
    let files = Arc::new(Files {
        // File written by log-writer
        log_file: env::var("LOG_FILE")
            .unwrap_or_else(|_| "/usr/src/app/files/log.txt".to_string()),
        // File written by the pingpong application
        pingpong_file: env::var("PINGPONG_FILE")
            .unwrap_or_else(|_| "/usr/src/app/files/pingpong.txt".to_string()),
    });

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /status answers with the latest log line and the ping-pong count
    let app = Router::new()
        .route(
            "/status",
            get(|State(files): State<Arc<Files>>| async move {
                // Read the files on every request so the latest values are returned
                let log = tokio::fs::read_to_string(&files.log_file)
                    .await
                    .unwrap_or_default();
                let Some(last_line) = log.lines().last() else {
                    return "log file not available yet".to_string();
                };

                // No file yet means pingpong has not received any request
                let pongs = tokio::fs::read_to_string(&files.pingpong_file)
                    .await
                    .ok()
                    .and_then(|content| content.trim().parse::<u64>().ok())
                    .unwrap_or(0);

                format!("{last_line}.\nPing / Pongs: {pongs}")
            }),
        )
        .with_state(files);

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
