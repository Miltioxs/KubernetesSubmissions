use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

// Longest todo accepted, the same limit the input field enforces
const MAX_TODO_LENGTH: usize = 140;

#[derive(Clone, Serialize)]
struct Todo {
    id: u64,
    content: String,
}

// Body of POST /todos
#[derive(Deserialize)]
struct NewTodo {
    content: String,
}

// Todos kept in memory; they are lost when the process restarts
type Todos = Arc<Mutex<Vec<Todo>>>;

#[tokio::main]
async fn main() {
    let todos: Todos = Arc::new(Mutex::new(Vec::new()));

    // Read the port from the PORT env var, falling back to 3000 if missing or invalid
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    // GET /todos lists the todos, POST /todos creates a new one
    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .with_state(todos);

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

async fn list_todos(State(todos): State<Todos>) -> Json<Vec<Todo>> {
    Json(todos.lock().unwrap().clone())
}

async fn create_todo(
    State(todos): State<Todos>,
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

    let mut todos = todos.lock().unwrap();
    let todo = Todo {
        id: todos.len() as u64 + 1,
        content: content.to_string(),
    };
    todos.push(todo.clone());

    println!("created todo {}: {}", todo.id, todo.content);
    Ok((StatusCode::CREATED, Json(todo)))
}
