//! Prometheus metrics for observability.
#![allow(dead_code)]

use once_cell::sync::Lazy;
use prometheus::{
    Encoder, IntCounter, IntCounterVec, IntGauge, Opts, Registry, TextEncoder,
};

/// Global metrics registry.
static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// Total messages processed.
pub static MESSAGES_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new("ferris_messages_total", "Total messages processed")
        .expect("metric creation");
    REGISTRY.register(Box::new(counter.clone())).ok();
    counter
});

/// Total commands processed, labeled by command name.
pub static COMMANDS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    let counter = IntCounterVec::new(
        Opts::new("ferris_commands_total", "Total commands processed"),
        &["command"],
    )
    .expect("metric creation");
    REGISTRY.register(Box::new(counter.clone())).ok();
    counter
});

/// Total callback queries processed.
pub static CALLBACKS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new("ferris_callbacks_total", "Total callback queries processed")
        .expect("metric creation");
    REGISTRY.register(Box::new(counter.clone())).ok();
    counter
});

/// Currently tracked chats.
pub static CHATS_GAUGE: Lazy<IntGauge> = Lazy::new(|| {
    let gauge = IntGauge::new("ferris_chats_active", "Number of known chats")
        .expect("metric creation");
    REGISTRY.register(Box::new(gauge.clone())).ok();
    gauge
});

/// Currently tracked users.
pub static USERS_GAUGE: Lazy<IntGauge> = Lazy::new(|| {
    let gauge = IntGauge::new("ferris_users_active", "Number of known users")
        .expect("metric creation");
    REGISTRY.register(Box::new(gauge.clone())).ok();
    gauge
});

/// Antiflood triggers.
pub static FLOOD_TRIGGERS: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new("ferris_flood_triggers_total", "Total antiflood triggers")
        .expect("metric creation");
    REGISTRY.register(Box::new(counter.clone())).ok();
    counter
});

/// Gban enforcements.
pub static GBAN_ENFORCEMENTS: Lazy<IntCounter> = Lazy::new(|| {
    let counter =
        IntCounter::new("ferris_gban_enforcements_total", "Total gban enforcements")
            .expect("metric creation");
    REGISTRY.register(Box::new(counter.clone())).ok();
    counter
});

/// Gather all registered metrics and encode to Prometheus text format.
pub fn gather_metrics() -> Result<String, String> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|e| e.to_string())?;
    String::from_utf8(buffer).map_err(|e| e.to_string())
}
