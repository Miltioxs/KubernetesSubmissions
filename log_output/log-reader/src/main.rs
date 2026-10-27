use axum::{Router, extract::State, routing::get};
use std::{env, sync::Arc, time::Duration};
use tokio::net::TcpListener;

// Where the data shown by /status comes from
struct AppState {
    // File written by log-writer, shared through a volume
    log_file: String,
    // Endpoint of the pingpong application that answers the number of pongs
    pingpong_url: String,
    // File mounted from the ConfigMap as a volume
    info_file: String,
    client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        log_file: env::var("LOG_FILE")
            .unwrap_or_else(|_| "/usr/src/app/files/log.txt".to_string()),
        pingpong_url: env::var("PINGPONG_URL")
            .unwrap_or_else(|_| "http://pingpong-service/pings".to_string()),
        info_file: env::var("INFO_FILE")
            .unwrap_or_else(|_| "/usr/src/app/config/information.txt".to_string()),
        // The timeout keeps /status from hanging when pingpong does not answer
        client: reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("failed to build HTTP client"),
    });

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /status answers with the ConfigMap values, the latest log line and the ping-pong count
    let app = Router::new()
        .route(
            "/status",
            get(|State(state): State<Arc<AppState>>| async move {
                // Read the ConfigMap file on every request, so changes to the ConfigMap show up without a restart
                let file_content = tokio::fs::read_to_string(&state.info_file)
                    .await
                    .map(|content| content.trim_end().to_string())
                    .unwrap_or_else(|_| "not available".to_string());

                // Env vars are fixed when the container starts, so a new MESSAGE needs a pod restart
                let message = match env::var("MESSAGE") {
                    Ok(message) => format!("MESSAGE={message}"),
                    Err(_) => "MESSAGE not set".to_string(),
                };

                let config = format!("file content: {file_content}\nenv variable: {message}");

                // Read the file on every request so the latest line is returned
                let log = tokio::fs::read_to_string(&state.log_file)
                    .await
                    .unwrap_or_default();
                let Some(last_line) = log.lines().last() else {
                    return format!("{config}\nlog file not available yet");
                };

                // Ask pingpong for the count; show "unavailable" so a failure is not mistaken for 0
                let pongs = match fetch_pongs(&state).await {
                    Ok(pongs) => pongs.to_string(),
                    Err(error) => {
                        eprintln!("failed to fetch pongs from {}: {error}", state.pingpong_url);
                        "unavailable".to_string()
                    }
                };

                format!("{config}\n{last_line}.\nPing / Pongs: {pongs}")
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

async fn fetch_pongs(state: &AppState) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let body = state
        .client
        .get(&state.pingpong_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(body.trim().parse()?)
}
