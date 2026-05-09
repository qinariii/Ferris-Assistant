//! TTL cache utility for reducing repeated API/DB queries.
#![allow(dead_code)]

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

/// A simple thread-safe TTL cache.
/// Generic over both key (`K`) and value (`V`).
pub struct TtlCache<K, V> {
    inner: Mutex<HashMap<K, (V, Instant)>>,
    ttl: Duration,
}

impl<K: Eq + Hash + Clone, V: Clone> TtlCache<K, V> {
    pub fn new(ttl: Duration) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(HashMap::new()),
            ttl,
        })
    }

    /// Get a cached value if it exists and hasn't expired.
    pub fn get(&self, key: &K) -> Option<V> {
        let map = self.inner.lock();
        if let Some((val, ts)) = map.get(key) {
            if ts.elapsed() < self.ttl {
                return Some(val.clone());
            }
        }
        None
    }

    /// Insert or update a cached value.
    pub fn set(&self, key: K, value: V) {
        let mut map = self.inner.lock();
        // Periodic cleanup when map grows large
        if map.len() > 2000 {
            let ttl = self.ttl;
            map.retain(|_, (_, ts)| ts.elapsed() < ttl);
        }
        map.insert(key, (value, Instant::now()));
    }

    /// Invalidate a specific key.
    pub fn invalidate(&self, key: &K) {
        self.inner.lock().remove(key);
    }

    /// Invalidate all entries matching a predicate on the key.
    pub fn invalidate_by<F: Fn(&K) -> bool>(&self, predicate: F) {
        self.inner.lock().retain(|k, _| !predicate(k));
    }
}