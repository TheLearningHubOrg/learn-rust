//! Stage 5 preview: shared ownership (`Arc`) plus interior mutability (`Mutex`).

use std::sync::{Arc, Mutex};

/// A counter that many threads can increment through cloned handles.
#[derive(Debug, Clone)]
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
        // A poisoned lock still holds a valid i64, so recover the guard.
        let mut value = self.counter.lock().unwrap_or_else(|e| e.into_inner());
        *value += 1;
    }

    pub fn get(&self) -> i64 {
        *self.counter.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Returns another owner of the same counter (the strong count goes up by one).
    pub fn clone_handle(&self) -> Arc<Mutex<i64>> {
        Arc::clone(&self.counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn increments_from_many_threads() {
        let counter = SafeCounter::new(0);
        thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    for _ in 0..1_000 {
                        counter.increment();
                    }
                });
            }
        });
        assert_eq!(counter.get(), 8_000);
    }

    #[test]
    fn handles_share_one_value() {
        let counter = SafeCounter::new(5);
        let handle = counter.clone_handle();
        *handle.lock().unwrap() += 10;
        assert_eq!(counter.get(), 15);
        assert_eq!(Arc::strong_count(&handle), 2);
    }
}
