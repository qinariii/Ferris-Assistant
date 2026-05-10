//! Redis-based caching layer for frequently accessed data.
//! Falls back gracefully when Redis is unavailable.

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::OnceCell;

static REDIS: OnceCell<Option<Arc<RedisCache>>> = OnceCell::const_new();

pub struct RedisCache {
    conn: ConnectionManager,
}

impl RedisCache {
    pub async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }
}

/// Initialize global Redis connection (call once at startup).
pub async fn init_redis(url: Option<&str>) {
    REDIS
        .get_or_init(|| async {
            match url {
                Some(u) => match RedisCache::new(u).await {
                    Ok(cache) => {
                        log::info!("Redis connected: {}", u);
                        Some(Arc::new(cache))
                    }
                    Err(e) => {
                        log::warn!("Redis unavailable (falling back to no-cache): {}", e);
                        None
                    }
                },
                None => {
                    log::info!("REDIS_URL not set, caching disabled");
                    None
                }
            }
        })
        .await;
}

/// Get a reference to the global Redis cache (if initialized).
fn get_cache() -> Option<Arc<RedisCache>> {
    REDIS.get().and_then(|o| o.clone())
}

/// Cache a value with a TTL (in seconds). Silently fails if Redis is unavailable.
pub async fn set(key: &str, value: &str, ttl_secs: u64) {
    if let Some(cache) = get_cache() {
        let mut conn = cache.conn.clone();
        let _: Result<(), _> = conn.set_ex(key, value, ttl_secs).await;
    }
}

/// Get a cached value. Returns None if Redis unavailable or key missing.
pub async fn get(key: &str) -> Option<String> {
    let cache = get_cache()?;
    let mut conn = cache.conn.clone();
    conn.get(key).await.ok()
}

/// Delete a cached key.
pub async fn del(key: &str) {
    if let Some(cache) = get_cache() {
        let mut conn = cache.conn.clone();
        let _: Result<(), _> = conn.del(key).await;
    }
}

/// Delete all keys matching a pattern (e.g. "chat:123:*").
#[allow(dead_code)]
pub async fn del_pattern(pattern: &str) {
    if let Some(cache) = get_cache() {
        let mut conn = cache.conn.clone();
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();
        if !keys.is_empty() {
            let _: Result<(), _> = conn.del(keys).await;
        }
    }
}

/// Check if Redis is connected and healthy.
pub async fn is_healthy() -> bool {
    if let Some(cache) = get_cache() {
        let mut conn = cache.conn.clone();
        let result: Result<String, _> = redis::cmd("PING").query_async(&mut conn).await;
        return result.is_ok();
    }
    // No Redis configured is considered "healthy" (feature disabled)
    true
}

// ─── Helper functions for common cache patterns ─────────────────────────────

/// Cache key for chat settings
pub fn chat_key(chat_id: i64) -> String {
    format!("chat:{}", chat_id)
}

/// Cache key for gban check
pub fn gban_key(user_id: i64) -> String {
    format!("gban:{}", user_id)
}

/// Cache key for disabled commands
pub fn disabled_cmds_key(chat_id: i64) -> String {
    format!("disabled:{}", chat_id)
}

/// Cache key for locks
pub fn locks_key(chat_id: i64) -> String {
    format!("locks:{}", chat_id)
}
