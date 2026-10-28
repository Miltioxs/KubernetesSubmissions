use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use std::{env, process, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio_postgres::{Client, NoTls, Row};

// Longest todo accepted, the same limit the input field enforces
const MAX_TODO_LENGTH: usize = 140;

// Time to wait between connection attempts while Postgres is starting
const RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(Serialize)]
struct Todo {
    id: i64,
    content: String,
}

impl From<Row> for Todo {
    fn from(row: Row) -> Self {
        Todo {
            id: row.get("id"),
            content: row.get("content"),
        }
    }
}

// Body of POST /todos
#[derive(Deserialize)]
struct NewTodo {
    content: String,
}

#[tokio::main]
async fn main() {
    // Connection string with the password, so there is no default
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("DATABASE_URL is not set");
        process::exit(1);
    });

    let client = Arc::new(connect(&database_url).await);

    // Create the table on startup, so no manual setup is needed
    // VARCHAR(140) counts characters, so the database enforces the same limit
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS todos (id BIGSERIAL PRIMARY KEY, content VARCHAR(140) NOT NULL);",
        )
        .await
        .expect("failed to create the todos table");

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /todos lists the todos, POST /todos creates a new one
    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
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

async fn list_todos(
    State(client): State<Arc<Client>>,
) -> Result<Json<Vec<Todo>>, (StatusCode, String)> {
    let rows = client
        .query("SELECT id, content FROM todos ORDER BY id", &[])
        .await
        .map_err(database_error)?;
    Ok(Json(rows.into_iter().map(Todo::from).collect()))
}

async fn create_todo(
    State(client): State<Arc<Client>>,
    Json(new_todo): Json<NewTodo>,
) -> Result<(StatusCode, Json<Todo>), (StatusCode, String)> {
    // Validate here too, since the input field limit can be bypassed
    let content = new_todo.content.trim();
    if content.is_empty() || content.chars().count() > MAX_TODO_LENGTH {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("todo must be between 1 and {MAX_TODO_LENGTH} characters"),
        ));
    }

    let row = client
        .query_one(
            "INSERT INTO todos (content) VALUES ($1) RETURNING id, content",
            &[&content],
        )
        .await
        .map_err(database_error)?;
    let todo = Todo::from(row);

    println!("created todo {}: {}", todo.id, todo.content);
    Ok((StatusCode::CREATED, Json(todo)))
}

fn database_error(error: tokio_postgres::Error) -> (StatusCode, String) {
    eprintln!("database error: {error}");
    (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string())
}
