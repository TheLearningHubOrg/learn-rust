pub mod concurrency;
pub mod memory;

use serde::{Deserialize, Serialize};

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
