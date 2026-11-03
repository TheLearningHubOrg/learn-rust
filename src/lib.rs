//! Library crate behind the `learn-rust` starter service.
//!
//! Each module maps to a stage of the guide:
//!
//! - [`config`]: Stage 1, parsing primitives and returning `Result`
//! - [`text`]: Stage 2, borrowing, slices and lifetimes
//! - [`tasks`]: Stage 3, structs, enums, pattern matching and error types
//! - [`memory`] and [`concurrency`]: previews of Stages 5 and 6
//! - [`app`]: the axum router that exposes the modules over HTTP
//!
//! `main.rs` stays tiny so that everything worth testing lives here.

pub mod app;
pub mod concurrency;
pub mod config;
pub mod memory;
pub mod tasks;
pub mod text;

use serde::{Deserialize, Serialize};

/// A small serialisable record returned by `GET /api/v1/metrics`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemMetric {
    pub id: u64,
    pub title: String,
    pub active_threads: usize,
    pub safe_memory: bool,
}

impl SystemMetric {
    pub fn new(id: u64, title: &str, threads: usize) -> Self {
        Self {
            id,
            title: title.to_string(),
            active_threads: threads,
            safe_memory: true,
        }
    }
}
