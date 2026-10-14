use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use learn_rust::{concurrency::MessageQueue, memory::SafeCounter, SystemMetric};

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "runtime": "tokio-async",
        "edition": "2021",
        "memory_safety": "guaranteed-by-borrow-checker"
    }))
}

async fn get_metrics() -> Json<Vec<SystemMetric>> {
    let metrics = vec![
        SystemMetric::new(1, "Zero-Cost Abstractions", 8),
        SystemMetric::new(2, "Fearless Concurrency with Tokio", 16),
        SystemMetric::new(3, "Memory Safety Without Garbage Collection", 1),
    ];
    Json(metrics)
}

#[tokio::main]
async fn main() {
    let counter = SafeCounter::new(100);
    counter.increment();

    let (_queue, mut rx) = MessageQueue::new(32);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("Received message: {}", msg);
        }
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/metrics", get(get_metrics));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("⚡ Rust Axum Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
