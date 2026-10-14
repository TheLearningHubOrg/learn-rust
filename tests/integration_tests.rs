use learn_rust::{memory::SafeCounter, SystemMetric};

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
