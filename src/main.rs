#![allow(unused_imports)]

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

    let pool = db::init_db(&cfg.database_url)
        .await
        .expect("Failed to initialize database");

    let bot = Bot::new(&cfg.bot_token);

    let me = bot.get_me().await.expect("Failed to get bot info");
    let cfg = cfg.with_bot_info(me.id.0, me.username().to_string());
    utils::permissions::set_bot_id(me.id);
    log::info!("Bot @{} started!", me.username());

    let flood_tracker = modules::antiflood::new_flood_tracker();
    let _flood_cleanup = modules::antiflood::spawn_flood_tracker_cleanup(
        std::sync::Arc::clone(&flood_tracker),
        30,
        10,
    );

    let handler = handlers::build_handler();

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