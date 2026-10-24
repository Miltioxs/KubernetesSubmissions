use axum::{Router, extract::State, routing::get};
use std::{
    env, fs,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

// Shared state: the request counter and the file where it is persisted
struct AppState {
    counter: Mutex<u64>,
    counter_file: String,
}

#[tokio::main]
async fn main() {
    // Read the file path from the PINGPONG_FILE env var, with a default for the container
    let counter_file = env::var("PINGPONG_FILE")
        .unwrap_or_else(|_| "/usr/src/app/files/pingpong.txt".to_string());

    // Make sure the parent directory exists before writing
    if let Some(dir) = Path::new(&counter_file).parent() {
        fs::create_dir_all(dir).expect("failed to create counter directory");
    }

    // Restore the counter from the file so it survives restarts, starting at 0 if missing
    let initial = fs::read_to_string(&counter_file)
        .ok()
        .and_then(|content| content.trim().parse().ok())
        .unwrap_or(0);

    let state = Arc::new(AppState {
        counter: Mutex::new(initial),
        counter_file,
    });

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /pingpong answers "pong N" and then increments the counter
    let app = Router::new()
        .route(
            "/pingpong",
            get(|State(state): State<Arc<AppState>>| async move {
                // The lock keeps increment and write together, so the file never goes backwards
                let mut counter = state.counter.lock().unwrap();
                let count = *counter;
                *counter += 1;

                // Persist the total number of requests so log-output can read it
                fs::write(&state.counter_file, counter.to_string())
                    .expect("failed to write counter file");

                format!("pong {count}")
            }),
        )
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
