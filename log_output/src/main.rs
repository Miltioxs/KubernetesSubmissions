use axum::{Router, extract::State, routing::get};
use chrono::{SecondsFormat, Utc};
use std::{env, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use uuid::Uuid;

// Builds the status line: current timestamp plus the random string
fn status_line(random_string: &str) -> String {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    format!("{timestamp}: {random_string}")
}

#[tokio::main]
async fn main() {
    // Random string generated once at startup and kept in memory, shared with the handler
    let random_string = Arc::new(Uuid::new_v4().to_string());

    // Background task: print the status every 5 seconds
    let logger_string = Arc::clone(&random_string);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            println!("{}", status_line(&logger_string));
        }
    });

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /status answers with the current timestamp and the random string
    let app = Router::new()
        .route(
            "/status",
            get(|State(random_string): State<Arc<String>>| async move {
                status_line(&random_string)
            }),
        )
        .with_state(random_string);

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
