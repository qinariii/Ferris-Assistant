mod config;
mod db;
mod handlers;
mod keyboards;
mod modules;
mod utils;

use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting Ferris Bot...");

    let cfg = config::AppConfig::from_env();
    log::info!("Bot name: {}", cfg.bot_name);
    log::info!("Mode: {:?}", cfg.bot_mode);

    // Initialize PostgreSQL
    let pool = db::init_db(&cfg.database_url)
        .await
        .expect("Failed to initialize database");

    // Initialize Redis (optional, graceful fallback)
    utils::redis_cache::init_redis(cfg.redis_url.as_deref()).await;

    // Start HTTP server (health check + metrics)
    utils::http_server::start_http_server(pool.clone(), cfg.http_port).await;

    let bot = Bot::new(&cfg.bot_token);

    let me = bot.get_me().await.expect("Failed to get bot info");
    let cfg = cfg.with_bot_info(me.id.0, me.username().to_string());
    utils::permissions::set_bot_id(me.id);
    log::info!("Bot @{} started!", me.username());

    modules::captcha::cleanup_captcha_on_startup(&bot, &pool).await;

    let flood_tracker = modules::antiflood::new_flood_tracker();
    let _flood_cleanup = modules::antiflood::spawn_flood_tracker_cleanup(
        std::sync::Arc::clone(&flood_tracker),
        30,
        10,
    );

    let handler = handlers::build_handler();

    match cfg.bot_mode {
        config::BotMode::Webhook => {
            let webhook_url = cfg.webhook_url.clone().unwrap_or_else(|| {
                panic!("WEBHOOK_URL must be set when BOT_MODE=webhook")
            });
            log::info!("Starting webhook mode: {}", webhook_url);

            let addr = ([0, 0, 0, 0], 8443).into();
            let url = webhook_url
                .parse()
                .expect("WEBHOOK_URL must be a valid URL");

            let listener = teloxide::update_listeners::webhooks::axum(bot.clone(), teloxide::update_listeners::webhooks::Options::new(addr, url))
                .await
                .expect("Failed to create webhook listener");

            Dispatcher::builder(bot, handler)
                .dependencies(dptree::deps![cfg, pool, flood_tracker])
                .default_handler(|upd| async move {
                    log::warn!("Unhandled update: {:?}", upd.id);
                })
                .error_handler(LoggingErrorHandler::with_custom_text(
                    "Error in dispatcher",
                ))
                .enable_ctrlc_handler()
                .build()
                .dispatch_with_listener(
                    listener,
                    LoggingErrorHandler::with_custom_text("Listener error"),
                )
                .await;
        }
        config::BotMode::Polling => {
            log::info!("Starting polling mode");
            Dispatcher::builder(bot, handler)
                .dependencies(dptree::deps![cfg, pool, flood_tracker])
                .default_handler(|upd| async move {
                    log::warn!("Unhandled update: {:?}", upd.id);
                })
                .error_handler(LoggingErrorHandler::with_custom_text(
                    "Error in dispatcher",
                ))
                .enable_ctrlc_handler()
                .build()
                .dispatch()
                .await;
        }
    }
}