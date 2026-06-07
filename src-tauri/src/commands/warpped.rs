use tauri::Manager;

use crate::collector::{get_games, get_sessions, DbConn, Game, Session};

#[tauri::command]
pub async fn get_played_games(app: tauri::AppHandle) -> Result<Vec<Game>, String> {
    let pool = app.state::<DbConn>();

    get_games(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_played_sessions(app: tauri::AppHandle) -> Result<Vec<Session>, String> {
    let pool = app.state::<DbConn>();

    get_sessions(&pool).await.map_err(|e| e.to_string())
}
