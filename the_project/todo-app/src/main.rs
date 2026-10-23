use axum::{Router, response::Html, routing::get};
use std::env;
use tokio::net::TcpListener;

// HTML page embedded into the binary at compile time
const INDEX_HTML: &str = include_str!("index.html");

#[tokio::main]
async fn main() {
    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // Single route: GET / answers with the HTML page
    let app = Router::new().route("/", get(|| async { Html(INDEX_HTML) }));

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
