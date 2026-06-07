// data collecting for roblox warpped
// everything stay locally, never send to a server
// opt out anytime in settings

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Game {
    pub id: i64,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: i64,
    pub game_id: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration: Option<i64>,
}

pub async fn init(app: &AppHandle) -> SqlitePool {
    let db_path = app.path().app_local_data_dir().unwrap().join("tracker.db");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.to_str().unwrap()))
        .unwrap()
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts).await.unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    log::info!("initialized data collection");

    pool
}

pub async fn log_game(pool: &SqlitePool, game_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("INSERT OR IGNORE INTO games (id) VALUES (?)", game_id)
        .execute(pool)
        .await?;

    log::info!("logged new game : {game_id}");

    Ok(())
}

pub async fn new_game_session(pool: &SqlitePool, game_id: i64) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query!(
        "UPDATE sessions
         SET ended_at = ?,
             duration = ? - started_at
         WHERE game_id = ? AND ended_at IS NULL",
        now,
        now,
        game_id
    )
    .execute(pool)
    .await?;

    let result = sqlx::query!(
        "INSERT INTO sessions (game_id, started_at) VALUES (?, ?)",
        game_id,
        now
    )
    .execute(pool)
    .await?;

    log::info!("logged new game session");

    Ok(result.last_insert_rowid())
}

pub async fn end_game_session(
    pool: &SqlitePool,
    session_id: i64,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query!(
        "UPDATE sessions
         SET ended_at = ?,
             duration = ? - started_at
         WHERE id = ?",
        now,
        now,
        session_id
    )
    .execute(pool)
    .await?;

    log::info!("logged end game session");

    Ok(())
}
