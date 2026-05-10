//! Integration tests for database queries.
//! Requires a running PostgreSQL instance (use docker-compose for testing).
//!
//! Run with: DATABASE_URL=postgres://ferris:ferris@localhost/ferris_test cargo test

use sqlx::postgres::PgPoolOptions;

type Pool = sqlx::PgPool;

async fn setup_test_pool() -> Pool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ferris:ferris@localhost/ferris_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_upsert_and_get_chat() {
    let pool = setup_test_pool().await;

    // Clean up
    sqlx::query("DELETE FROM chats WHERE chat_id = -1001")
        .execute(&pool)
        .await
        .ok();

    // Upsert
    sqlx::query(
        "INSERT INTO chats (chat_id, chat_name) VALUES ($1, $2)
         ON CONFLICT(chat_id) DO UPDATE SET chat_name = excluded.chat_name",
    )
    .bind(-1001i64)
    .bind("Test Chat")
    .execute(&pool)
    .await
    .expect("upsert_chat failed");

    // Get
    let row: Option<(i64, String)> =
        sqlx::query_as("SELECT chat_id, chat_name FROM chats WHERE chat_id = $1")
            .bind(-1001i64)
            .fetch_optional(&pool)
            .await
            .expect("get_chat failed");

    assert!(row.is_some());
    let (id, name) = row.unwrap();
    assert_eq!(id, -1001);
    assert_eq!(name, "Test Chat");

    // Cleanup
    sqlx::query("DELETE FROM chats WHERE chat_id = -1001")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_upsert_and_get_user() {
    let pool = setup_test_pool().await;

    sqlx::query("DELETE FROM users WHERE user_id = 999")
        .execute(&pool)
        .await
        .ok();

    sqlx::query(
        "INSERT INTO users (user_id, username, first_name, last_name) VALUES ($1, $2, $3, $4)
         ON CONFLICT(user_id) DO UPDATE SET username = excluded.username",
    )
    .bind(999i64)
    .bind("testuser")
    .bind("Test")
    .bind("User")
    .execute(&pool)
    .await
    .expect("upsert_user failed");

    let row: Option<(i64, String)> =
        sqlx::query_as("SELECT user_id, username FROM users WHERE user_id = $1")
            .bind(999i64)
            .fetch_optional(&pool)
            .await
            .expect("get_user failed");

    assert!(row.is_some());
    let (id, username) = row.unwrap();
    assert_eq!(id, 999);
    assert_eq!(username, "testuser");

    sqlx::query("DELETE FROM users WHERE user_id = 999")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_notes_crud() {
    let pool = setup_test_pool().await;

    // Setup chat
    sqlx::query(
        "INSERT INTO chats (chat_id, chat_name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(-2001i64)
    .bind("Notes Test")
    .execute(&pool)
    .await
    .ok();

    // Clean
    sqlx::query("DELETE FROM notes WHERE chat_id = -2001")
        .execute(&pool)
        .await
        .ok();

    // Save note
    sqlx::query(
        "INSERT INTO notes (chat_id, name, content) VALUES ($1, $2, $3)
         ON CONFLICT(chat_id, name) DO UPDATE SET content = excluded.content",
    )
    .bind(-2001i64)
    .bind("test")
    .bind("Hello world")
    .execute(&pool)
    .await
    .expect("save_note failed");

    // Get note
    let row: Option<(i64, String, String)> =
        sqlx::query_as("SELECT chat_id, name, content FROM notes WHERE chat_id = $1 AND name = $2")
            .bind(-2001i64)
            .bind("test")
            .fetch_optional(&pool)
            .await
            .expect("get_note failed");

    assert!(row.is_some());
    let (_, name, content) = row.unwrap();
    assert_eq!(name, "test");
    assert_eq!(content, "Hello world");

    // Delete note
    let result = sqlx::query("DELETE FROM notes WHERE chat_id = $1 AND name = $2")
        .bind(-2001i64)
        .bind("test")
        .execute(&pool)
        .await
        .expect("delete_note failed");
    assert!(result.rows_affected() > 0);

    // Cleanup
    sqlx::query("DELETE FROM chats WHERE chat_id = -2001")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_warns_flow() {
    let pool = setup_test_pool().await;

    // Setup
    sqlx::query(
        "INSERT INTO chats (chat_id, chat_name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(-3001i64)
    .bind("Warns Test")
    .execute(&pool)
    .await
    .ok();

    sqlx::query("DELETE FROM warns WHERE chat_id = -3001")
        .execute(&pool)
        .await
        .ok();

    // Add warn
    sqlx::query("INSERT INTO warns (chat_id, user_id, reason, warned_by) VALUES ($1, $2, $3, $4)")
        .bind(-3001i64)
        .bind(100i64)
        .bind("spam")
        .bind(1i64)
        .execute(&pool)
        .await
        .expect("add_warn failed");

    // Count
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM warns WHERE chat_id = $1 AND user_id = $2")
            .bind(-3001i64)
            .bind(100i64)
            .fetch_one(&pool)
            .await
            .expect("count warns failed");
    assert_eq!(count.0, 1);

    // Reset
    sqlx::query("DELETE FROM warns WHERE chat_id = $1 AND user_id = $2")
        .bind(-3001i64)
        .bind(100i64)
        .execute(&pool)
        .await
        .ok();

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM warns WHERE chat_id = $1 AND user_id = $2")
            .bind(-3001i64)
            .bind(100i64)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count.0, 0);

    // Cleanup
    sqlx::query("DELETE FROM chats WHERE chat_id = -3001")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_blacklist_operations() {
    let pool = setup_test_pool().await;

    sqlx::query(
        "INSERT INTO chats (chat_id, chat_name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(-4001i64)
    .bind("Blacklist Test")
    .execute(&pool)
    .await
    .ok();

    sqlx::query("DELETE FROM blacklist WHERE chat_id = -4001")
        .execute(&pool)
        .await
        .ok();

    // Add
    sqlx::query(
        "INSERT INTO blacklist (chat_id, trigger_word, mode) VALUES ($1, $2, $3)
         ON CONFLICT(chat_id, trigger_word) DO UPDATE SET mode = excluded.mode",
    )
    .bind(-4001i64)
    .bind("badword")
    .bind("delete")
    .execute(&pool)
    .await
    .expect("add_blacklist failed");

    // Check
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT trigger_word FROM blacklist WHERE chat_id = $1")
            .bind(-4001i64)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "badword");

    // Remove
    let result = sqlx::query("DELETE FROM blacklist WHERE chat_id = $1 AND trigger_word = $2")
        .bind(-4001i64)
        .bind("badword")
        .execute(&pool)
        .await
        .unwrap();
    assert!(result.rows_affected() > 0);

    // Cleanup
    sqlx::query("DELETE FROM chats WHERE chat_id = -4001")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_gban_operations() {
    let pool = setup_test_pool().await;

    sqlx::query("DELETE FROM gbans WHERE user_id = 777")
        .execute(&pool)
        .await
        .ok();

    // Gban
    sqlx::query(
        "INSERT INTO gbans (user_id, reason, banned_by) VALUES ($1, $2, $3)
         ON CONFLICT(user_id) DO UPDATE SET reason = excluded.reason",
    )
    .bind(777i64)
    .bind("spammer")
    .bind(1i64)
    .execute(&pool)
    .await
    .expect("gban failed");

    // Check
    let row: Option<(i64,)> = sqlx::query_as("SELECT user_id FROM gbans WHERE user_id = $1")
        .bind(777i64)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(row.is_some());

    // Ungban
    let result = sqlx::query("DELETE FROM gbans WHERE user_id = $1")
        .bind(777i64)
        .execute(&pool)
        .await
        .unwrap();
    assert!(result.rows_affected() > 0);

    let row: Option<(i64,)> = sqlx::query_as("SELECT user_id FROM gbans WHERE user_id = $1")
        .bind(777i64)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(row.is_none());
}
