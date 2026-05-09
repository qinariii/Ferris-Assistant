use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bot_token: String,
    pub bot_name: String,
    pub owner_id: i64,
    pub sudo_users: Vec<i64>,
    pub database_url: String,
    pub bot_username: String,
    pub bot_id: u64,
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
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://ferris.db".to_string());

        Self {
            bot_token,
            bot_name,
            owner_id,
            sudo_users,
            database_url,
            bot_username: String::new(),
            bot_id: 0,
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