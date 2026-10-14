use std::sync::{Arc, Mutex};

/// Demonstrates thread-safe reference counting with Arc and interior mutability with Mutex
pub struct SafeCounter {
    counter: Arc<Mutex<i64>>,
}

impl SafeCounter {
    pub fn new(initial: i64) -> Self {
        Self {
            counter: Arc::new(Mutex::new(initial)),
        }
    }

    pub fn increment(&self) {
        if let Ok(mut lock) = self.counter.lock() {
            *lock += 1;
        }
    }

    pub fn get(&self) -> i64 {
        *self.counter.lock().unwrap()
    }

    pub fn clone_handle(&self) -> Arc<Mutex<i64>> {
        Arc::clone(&self.counter)
    }
}
