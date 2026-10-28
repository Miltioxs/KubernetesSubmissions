use axum::{Router, extract::State, http::StatusCode, routing::get};
use std::{env, process, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio_postgres::{Client, NoTls};

// Time to wait between connection attempts while Postgres is starting
const RETRY_DELAY: Duration = Duration::from_secs(2);

#[tokio::main]
async fn main() {
    // Connection string with the password, so there is no default
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("DATABASE_URL is not set");
        process::exit(1);
    });

    let client = Arc::new(connect(&database_url).await);

    // Create the table and its single row on startup, so no manual setup is needed
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS pingpong_counter (id INT PRIMARY KEY, count BIGINT NOT NULL);
             INSERT INTO pingpong_counter (id, count) VALUES (1, 0) ON CONFLICT (id) DO NOTHING;",
        )
        .await
        .expect("failed to create the counter table");

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /pingpong answers "pong N" and then increments the counter
    // GET /pings answers the total number of pongs, used by log-output over HTTP
    let app = Router::new()
        .route(
            "/pingpong",
            get(|State(client): State<Arc<Client>>| async move {
                // The increment happens inside Postgres, so concurrent requests never lose a count
                // count - 1 is the value before the increment, so the first request gets "pong 0"
                client
                    .query_one(
                        "UPDATE pingpong_counter SET count = count + 1 WHERE id = 1 RETURNING count - 1",
                        &[],
                    )
                    .await
                    .map(|row| format!("pong {}", row.get::<_, i64>(0)))
                    .map_err(database_error)
            }),
        )
        .route(
            "/pings",
            get(|State(client): State<Arc<Client>>| async move {
                client
                    .query_one("SELECT count FROM pingpong_counter WHERE id = 1", &[])
                    .await
                    .map(|row| row.get::<_, i64>(0).to_string())
                    .map_err(database_error)
            }),
        )
        .with_state(client);

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

async fn connect(database_url: &str) -> Client {
    // Postgres may still be starting, so keep trying instead of crashing
    loop {
        match tokio_postgres::connect(database_url, NoTls).await {
            Ok((client, connection)) => {
                // The connection runs on its own task; if it ends, exit so Kubernetes restarts the container
                tokio::spawn(async move {
                    if let Err(error) = connection.await {
                        eprintln!("database connection lost: {error}");
                    }
                    process::exit(1);
                });
                println!("Connected to the database");
                return client;
            }
            Err(error) => {
                eprintln!("database not available yet ({error}), retrying in {RETRY_DELAY:?}");
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    }
}

fn database_error(error: tokio_postgres::Error) -> (StatusCode, String) {
    eprintln!("database error: {error}");
    (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string())
}
