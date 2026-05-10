use std::env;

/// Bot run mode: polling (default) or webhook
#[derive(Clone, Debug, PartialEq)]
pub enum BotMode {
    Polling,
    Webhook,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bot_token: String,
    pub bot_name: String,
    pub owner_id: i64,
    pub sudo_users: Vec<i64>,
    pub database_url: String,
    pub bot_username: String,
    pub bot_id: u64,
    // Redis
    pub redis_url: Option<String>,
    // Webhook & HTTP
    pub bot_mode: BotMode,
    pub webhook_url: Option<String>,
    pub http_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let bot_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN not set");
        let bot_name = env::var("BOT_NAME").unwrap_or_else(|_| "FerrisBot".to_string());
        let owner_id: i64 = env::var("OWNER_ID")
            .expect("OWNER_ID not set")
            .parse()
            .expect("OWNER_ID must be a valid i64");
        let sudo_users: Vec<i64> = env::var("SUDO_USERS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://ferris:ferris@localhost/ferris".to_string());

        let redis_url = env::var("REDIS_URL").ok();

        let bot_mode = match env::var("BOT_MODE").unwrap_or_default().to_lowercase().as_str() {
            "webhook" => BotMode::Webhook,
            _ => BotMode::Polling,
        };
        let webhook_url = env::var("WEBHOOK_URL").ok();
        let http_port: u16 = env::var("HTTP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        Self {
            bot_token,
            bot_name,
            owner_id,
            sudo_users,
            database_url,
            bot_username: String::new(),
            bot_id: 0,
            redis_url,
            bot_mode,
            webhook_url,
            http_port,
        }
    }

    pub fn with_bot_info(mut self, bot_id: u64, username: impl Into<String>) -> Self {
        self.bot_id = bot_id;
        self.bot_username = username.into();
        self
    }

    pub fn is_owner(&self, user_id: i64) -> bool {
        self.owner_id == user_id
    }

    pub fn is_sudo(&self, user_id: i64) -> bool {
        self.owner_id == user_id || self.sudo_users.contains(&user_id)
    }
}