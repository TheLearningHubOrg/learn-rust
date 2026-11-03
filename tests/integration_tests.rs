//! Black-box tests: they see only the crate's public API, like any other user.

use learn_rust::tasks::{Action, NewTask, Priority, Status, TaskError, TaskStore};
use learn_rust::text::{first_word, stats};
use learn_rust::{SystemMetric, memory::SafeCounter};

#[test]
fn test_safe_counter_increment() {
    let counter = SafeCounter::new(10);
    counter.increment();
    assert_eq!(counter.get(), 11);
}

#[test]
fn test_system_metric_creation() {
    let metric = SystemMetric::new(1, "Ownership Model", 4);
    assert_eq!(metric.title, "Ownership Model");
    assert!(metric.safe_memory);
}

#[test]
fn task_lifecycle_through_public_api() {
    let mut store = TaskStore::new();
    let id = store
        .create(NewTask {
            title: "review PR".into(),
            priority: Priority::High,
        })
        .unwrap()
        .id;

    store
        .apply(
            id,
            Action::Start {
                assignee: "kim".into(),
            },
        )
        .unwrap();
    assert_eq!(
        store.apply(id, Action::Complete).unwrap().status,
        Status::Done
    );

    let removed = store.remove(id).unwrap();
    assert_eq!(removed.title, "review PR");
    assert_eq!(store.get(id), Err(TaskError::NotFound(id)));
}

#[test]
fn text_helpers_work_on_borrowed_input() {
    let owned = String::from("ownership borrowing lifetimes ownership");
    assert_eq!(first_word(&owned), "ownership");
    let summary = stats(&owned, 1);
    assert_eq!(summary.top[0].word, "ownership");
    assert_eq!(summary.top[0].count, 2);
    // `owned` is still usable: the helpers only borrowed it.
    assert_eq!(owned.len(), 39);
}
