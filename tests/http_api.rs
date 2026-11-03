//! Drives the axum router in-process: no socket, no port, no flakiness.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use learn_rust::app::router;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => builder
            .header("content-type", "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    }
    .expect("valid request");

    // Router clones are cheap and share the same Arc'd state.
    let response = app.clone().oneshot(request).await.expect("infallible");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn health_reports_ok() {
    let app = router();
    let (status, body) = send(&app, "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn metrics_returns_three_items() {
    let (status, body) = send(&router(), "GET", "/api/v1/metrics", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().map(Vec::len), Some(3));
}

#[tokio::test]
async fn text_stats_endpoint() {
    let payload = json!({ "text": "to be or not to be", "top": 2 });
    let (status, body) = send(&router(), "POST", "/api/v1/text/stats", Some(payload)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["words"], 6);
    assert_eq!(body["unique_words"], 4);
    assert_eq!(
        body["top"],
        json!([{"word": "be", "count": 2}, {"word": "to", "count": 2}])
    );
}

#[tokio::test]
async fn task_crud_and_state_machine() {
    let app = router();

    let (status, task) = send(
        &app,
        "POST",
        "/api/v1/tasks",
        Some(json!({ "title": "write tests", "priority": "high" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(task["id"], 1);
    assert_eq!(task["status"], json!({ "state": "todo" }));

    let (status, task) = send(
        &app,
        "POST",
        "/api/v1/tasks/1/actions",
        Some(json!({ "action": "start", "assignee": "ana" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        task["status"],
        json!({ "state": "in_progress", "assignee": "ana" })
    );

    let (status, body) = send(
        &app,
        "POST",
        "/api/v1/tasks/1/actions",
        Some(json!({ "action": "start", "assignee": "bo" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "cannot start a task that is in progress");

    let (status, list) = send(&app, "GET", "/api/v1/tasks?priority=high", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().map(Vec::len), Some(1));

    let (status, _) = send(&app, "DELETE", "/api/v1/tasks/1", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = send(&app, "GET", "/api/v1/tasks/1", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "task 1 not found");
}

#[tokio::test]
async fn validation_errors_are_422() {
    let app = router();
    let (status, body) = send(
        &app,
        "POST",
        "/api/v1/tasks",
        Some(json!({ "title": "  " })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "title must be 1 to 80 characters, got 0");

    let (status, body) = send(&app, "GET", "/api/v1/tasks?priority=urgent", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body["error"],
        "unknown priority \"urgent\"; expected low, medium or high"
    );
}

#[tokio::test]
async fn bad_json_is_rejected_by_the_extractor() {
    let app = router();
    let (status, _) = send(
        &app,
        "POST",
        "/api/v1/tasks",
        Some(json!({ "name": "no title" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = send(&app, "GET", "/api/v1/tasks/not-a-number", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
