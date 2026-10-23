use axum::{Router, extract::State, routing::get};
use std::{
    env,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Request counter kept in memory; it resets when the process restarts
    let counter = Arc::new(AtomicU64::new(0));

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /pingpong answers "pong N" and then increments the counter
    let app = Router::new()
        .route(
            "/pingpong",
            get(|State(counter): State<Arc<AtomicU64>>| async move {
                // fetch_add returns the previous value, so the first request gets "pong 0"
                let count = counter.fetch_add(1, Ordering::SeqCst);
                format!("pong {count}")
            }),
        )
        .with_state(counter);

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
