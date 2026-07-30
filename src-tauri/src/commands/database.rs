use rusqlite::Connection;
use std::sync::Mutex;

use super::portable::get_portable_dir;

pub struct DbState(pub Mutex<Connection>);

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

    Ok(conn)
}
