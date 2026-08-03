use rusqlite::{params, Connection};
use std::sync::Mutex;

use super::portable::get_portable_dir;

pub struct DbState(pub Mutex<Connection>);

/// 当前 schema 版本号。每次加列/改表结构时 +1，并在 apply_migrations 的 match 中新增对应 arm。
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub fn init_db() -> Result<Connection, String> {
    let db_path = get_portable_dir().join("abel-music.db");
    let conn = Connection::open(db_path).map_err(|e| format!("failed to open database: {}", e))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS playlist_tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            playlist_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS track_cache (
            path TEXT PRIMARY KEY,
            file_name TEXT NOT NULL,
            title TEXT,
            artist TEXT,
            album TEXT,
            duration REAL NOT NULL DEFAULT 0,
            track_number INTEGER,
            excluded INTEGER NOT NULL DEFAULT 0,
            scanned_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_playlist_tracks_playlist ON playlist_tracks(playlist_id, position);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_playlist_tracks_unique ON playlist_tracks(playlist_id, path);
        CREATE INDEX IF NOT EXISTS idx_track_cache_artist ON track_cache(artist);
        CREATE INDEX IF NOT EXISTS idx_track_cache_album ON track_cache(album);
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS library_folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            parent_path TEXT,
            is_root INTEGER NOT NULL DEFAULT 0,
            excluded INTEGER NOT NULL DEFAULT 0,
            audio_count INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_library_folders_parent ON library_folders(parent_path);
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;"
    ).map_err(|e| format!("failed to init database tables: {}", e))?;

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_track_cache_excluded_artist ON track_cache(excluded, artist);
            CREATE INDEX IF NOT EXISTS idx_track_cache_excluded_album ON track_cache(excluded, album);"
    ).map_err(|e| format!("failed to create additional indexes: {}", e))?;

    apply_migrations(&conn)?;

    Ok(conn)
}

/// 判断某列是否存在（SQLite 不支持 ADD COLUMN IF NOT EXISTS）
#[allow(dead_code)]
fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let sql = format!("PRAGMA table_info({})", table);
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) {
            for row in rows.flatten() {
                if row == column {
                    return true;
                }
            }
        }
    }
    false
}

fn get_schema_version(conn: &Connection) -> u32 {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse::<u32>().ok())
    .unwrap_or(0)
}

fn set_schema_version(conn: &Connection, version: u32) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('schema_version', ?1)",
        params![version.to_string()],
    )
    .map_err(|e| format!("failed to set schema version: {}", e))?;
    Ok(())
}

/// 顺序应用 schema 迁移，每个版本升级在独立事务内。不支持降级。
fn apply_migrations(conn: &Connection) -> Result<(), String> {
    let current = get_schema_version(conn);

    // 全新安装或老用户首次升级（无 schema_version）：
    // CREATE TABLE 已建最新结构，直接设为当前版本，跳过迁移。
    if current == 0 {
        set_schema_version(conn, CURRENT_SCHEMA_VERSION)?;
        return Ok(());
    }

    // 已有 schema_version，顺序应用迁移
    let mut version = current;
    while version < CURRENT_SCHEMA_VERSION {
        let next = version + 1;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        match version {
            // 示例（未来新增列时取消注释）：
            // 1 => {
            //     if !column_exists(&tx, "track_cache", "cover") {
            //         tx.execute("ALTER TABLE track_cache ADD COLUMN cover TEXT", [])
            //             .map_err(|e| e.to_string())?;
            //     }
            // }
            _ => {}
        }
        set_schema_version(&tx, next)?;
        tx.commit().map_err(|e| e.to_string())?;
        version = next;
    }

    Ok(())
}
