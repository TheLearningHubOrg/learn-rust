//! The HTTP layer: routes, shared state and the mapping from domain errors to responses.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::SystemMetric;
use crate::tasks::{Action, NewTask, Priority, Task, TaskError, TaskStore};
use crate::text::{self, TextStats};

/// Shared state. Cloning an `AppState` clones the `Arc`, not the store.
#[derive(Clone, Default)]
pub struct AppState {
    tasks: Arc<Mutex<TaskStore>>,
}

impl AppState {
    /// Locks the store. A panic in another handler poisons the mutex; the store
    /// holds no invariant a half-finished write could break, so keep serving.
    fn store(&self) -> MutexGuard<'_, TaskStore> {
        self.tasks.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub fn router() -> Router {
    router_with_state(AppState::default())
}

pub fn router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/metrics", get(metrics))
        .route("/api/v1/text/stats", post(text_stats))
        .route("/api/v1/tasks", get(list_tasks).post(create_task))
        .route("/api/v1/tasks/{id}", get(get_task).delete(delete_task))
        .route("/api/v1/tasks/{id}/actions", post(apply_action))
        .with_state(state)
}

/// Domain errors become HTTP responses here, in one place.
impl IntoResponse for TaskError {
    fn into_response(self) -> Response {
        let status = match &self {
            TaskError::NotFound(_) => StatusCode::NOT_FOUND,
            TaskError::InvalidTitle { .. } | TaskError::UnknownPriority(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            TaskError::InvalidTransition { .. } => StatusCode::CONFLICT,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

async fn metrics() -> Json<Vec<SystemMetric>> {
    Json(vec![
        SystemMetric::new(1, "Zero-Cost Abstractions", 8),
        SystemMetric::new(2, "Fearless Concurrency with Tokio", 16),
        SystemMetric::new(3, "Memory Safety Without Garbage Collection", 1),
    ])
}

#[derive(Deserialize)]
struct TextRequest {
    text: String,
    #[serde(default = "default_top")]
    top: usize,
}

fn default_top() -> usize {
    5
}

async fn text_stats(Json(req): Json<TextRequest>) -> Json<TextStats> {
    Json(text::stats(&req.text, req.top))
}

#[derive(Deserialize)]
struct ListQuery {
    priority: Option<String>,
}

async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Task>>, TaskError> {
    // Option<Result<_, _>> -> Result<Option<_>, _>, then `?` returns early on a bad value.
    let priority = query
        .priority
        .as_deref()
        .map(str::parse::<Priority>)
        .transpose()?;
    let tasks = state.store().list(priority).into_iter().cloned().collect();
    Ok(Json(tasks))
}

async fn create_task(
    State(state): State<AppState>,
    Json(new): Json<NewTask>,
) -> Result<(StatusCode, Json<Task>), TaskError> {
    let task = state.store().create(new)?.clone();
    Ok((StatusCode::CREATED, Json(task)))
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, TaskError> {
    Ok(Json(state.store().get(id)?.clone()))
}

async fn apply_action(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(action): Json<Action>,
) -> Result<Json<Task>, TaskError> {
    Ok(Json(state.store().apply(id, action)?.clone()))
}

async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, TaskError> {
    state.store().remove(id)?;
    Ok(StatusCode::NO_CONTENT)
}
