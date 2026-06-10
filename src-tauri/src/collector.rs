// data collecting for roblox warpped
// everything stay locally, never send to a server
// opt out anytime in settings

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub game_id: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration: Option<i64>,
}

pub type DbConn = Arc<Mutex<Connection>>;

#[cfg(windows)]
fn derive_key() -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let guid = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Cryptography")
        .and_then(|k| k.get_value::<String, _>("MachineGuid"))
        .unwrap_or_else(|_| "crush-fallback".to_string());
    format!("crush-tracker-v1:{}", guid)
}

#[cfg(not(windows))]
fn derive_key() -> String {
    "crush-tracker-v1:default".to_string()
}

fn run_migrations(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY
        );
        CREATE TABLE IF NOT EXISTS sessions (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id    INTEGER NOT NULL REFERENCES games(id),
            started_at INTEGER NOT NULL,
            ended_at   INTEGER,
            duration   INTEGER
        );",
    )
    .unwrap();
}

pub fn init(app: &AppHandle) -> DbConn {
    let db_path = app.path().app_local_data_dir().unwrap().join("tracker.db");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    let key = derive_key();

    let conn = Connection::open(&db_path).unwrap();
    conn.pragma_update(None, "key", &key).unwrap();

    if conn
        .execute_batch("SELECT count(*) FROM sqlite_master;")
        .is_err()
    {
        drop(conn);
        let bak = db_path.with_extension("db.bak");
        if let Err(e) = std::fs::rename(&db_path, &bak) {
            log::warn!("could not back up old tracker.db: {e}");
        } else {
            log::info!("backed up unencrypted tracker.db → tracker.db.bak");
        }
        let conn = Connection::open(&db_path).unwrap();
        conn.pragma_update(None, "key", &key).unwrap();
        run_migrations(&conn);
        log::info!("initialized encrypted data collection (fresh)");
        return Arc::new(Mutex::new(conn));
    }

    run_migrations(&conn);
    log::info!("initialized encrypted data collection");
    Arc::new(Mutex::new(conn))
}

pub async fn log_game(db: &DbConn, game_id: i64) -> Result<(), rusqlite::Error> {
    let db = Arc::clone(db);
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO games (id) VALUES (?1)",
            params![game_id],
        )?;
        log::info!("logged new game : {game_id}");
        Ok(())
    })
    .await
    .unwrap()
}

pub async fn new_game_session(db: &DbConn, game_id: i64) -> Result<i64, rusqlite::Error> {
    let db = Arc::clone(db);
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();

        conn.execute(
            "UPDATE sessions
             SET ended_at = ?1, duration = ?1 - started_at
             WHERE game_id = ?2 AND ended_at IS NULL",
            params![now, game_id],
        )?;

        conn.execute(
            "INSERT INTO sessions (game_id, started_at) VALUES (?1, ?2)",
            params![game_id, now],
        )?;

        log::info!("logged new game session");
        Ok(conn.last_insert_rowid())
    })
    .await
    .unwrap()
}

pub async fn get_games(db: &DbConn) -> Result<Vec<Game>, rusqlite::Error> {
    let db = Arc::clone(db);
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM games ORDER BY id ASC")?;
        let games = stmt
            .query_map([], |row| Ok(Game { id: row.get(0)? }))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(games)
    })
    .await
    .unwrap()
}

pub async fn get_sessions(db: &DbConn) -> Result<Vec<Session>, rusqlite::Error> {
    let db = Arc::clone(db);
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, game_id, started_at, ended_at, duration
             FROM sessions
             ORDER BY started_at DESC",
        )?;
        let sessions = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    game_id: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    duration: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sessions)
    })
    .await
    .unwrap()
}

pub async fn end_game_session(db: &DbConn, session_id: i64) -> Result<(), rusqlite::Error> {
    let db = Arc::clone(db);
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();

        conn.execute(
            "UPDATE sessions
             SET ended_at = ?1, duration = ?1 - started_at
             WHERE id = ?2",
            params![now, session_id],
        )?;

        log::info!("logged end game session");
        Ok(())
    })
    .await
    .unwrap()
}
