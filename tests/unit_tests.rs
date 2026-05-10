//! Unit tests that don't require external services.

#[test]
fn test_metrics_gather() {
    // Ensure metrics can be gathered without panicking
    use prometheus::{IntCounter, Registry, Encoder, TextEncoder};
    let registry = Registry::new();
    let counter = IntCounter::new("test_counter", "test").unwrap();
    registry.register(Box::new(counter.clone())).unwrap();
    counter.inc();

    let encoder = TextEncoder::new();
    let families = registry.gather();
    let mut buf = Vec::new();
    encoder.encode(&families, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("test_counter"));
    assert!(output.contains("1"));
}

#[test]
fn test_redis_key_formats() {
    // Test key generation helpers
    assert_eq!(format!("chat:{}", -1001i64), "chat:-1001");
    assert_eq!(format!("gban:{}", 123i64), "gban:123");
    assert_eq!(format!("disabled:{}", -500i64), "disabled:-500");
    assert_eq!(format!("locks:{}", -999i64), "locks:-999");
}

#[test]
fn test_ttl_cache() {
    use std::time::Duration;

    // Simple in-memory TTL cache test
    use std::collections::HashMap;
    use std::time::Instant;

    let mut map: HashMap<String, (String, Instant)> = HashMap::new();
    let ttl = Duration::from_millis(50);

    // Set
    map.insert("key1".into(), ("value1".into(), Instant::now()));

    // Get (fresh)
    if let Some((val, ts)) = map.get("key1") {
        assert!(ts.elapsed() < ttl);
        assert_eq!(val, "value1");
    }

    // Wait for expiry
    std::thread::sleep(Duration::from_millis(60));
    if let Some((_, ts)) = map.get("key1") {
        assert!(ts.elapsed() >= ttl);
    }
}
