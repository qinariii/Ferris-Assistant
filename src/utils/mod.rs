pub mod cache;
pub mod http_server;
pub mod kick;
pub mod metrics;
pub mod permissions;
pub mod extraction;
pub mod formatting;
pub mod i18n;
pub mod redis_cache;

/// Extension trait to log errors instead of silently discarding them with `.ok()`.
pub trait LogErrExt<T> {
    fn log_err(self, context: &str) -> Option<T>;
}

impl<T, E: std::fmt::Display> LogErrExt<T> for Result<T, E> {
    fn log_err(self, context: &str) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("[{}] {}", context, e);
                None
            }
        }
    }
}
