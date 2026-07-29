use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use super::portable::get_portable_dir;

pub struct DbState(pub Mutex<Connection>);

pub fn init_db() -> Connection {
    let db_path = get_portable_dir().join("abel-music.db");
    let conn = Connection::open(db_path).expect("failed to open database");

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
            cover_hash TEXT,
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
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;"
    ).expect("failed to init database tables");
    
    conn
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub track_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub path: String,
    pub file_name: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedTrack {
    pub path: String,
    pub file_name: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub track_number: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistGroup {
    pub artist: String,
    pub track_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumGroup {
    pub album: String,
    pub artist: Option<String>,
    pub track_count: i64,
}

#[tauri::command]
pub fn create_playlist(name: String, state: tauri::State<'_, DbState>) -> Result<Playlist, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO playlists (name) VALUES (?1)", params![name])
        .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let created_at: String = conn.query_row(
        "SELECT created_at FROM playlists WHERE id = ?1", params![id],
        |row| row.get(0)    
    ).map_err(|e| e.to_string())?;
    Ok(Playlist { id, name, track_count: 0, created_at })
}

#[tauri::command]
pub fn delete_playlist(id: i64, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlist_tracks WHERE playlist_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn rename_playlist(id: i64, name: String, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE playlists SET name = ?1 WHERE ID = ?2", params![name, id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_playlists(state: tauri::State<'_, DbState>) -> Result<Vec<Playlist>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.created_at, COUNT(pt.id) as track_count
        FROM playlists p
        LEFT JOIN playlist_tracks pt ON pt.playlist_id = p.id
        GROUP BY p.id
        ORDER BY p.created_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(Playlist {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            track_count: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn add_to_playlist(playlist_id: i64, paths: Vec<String>, state: tauri::State<'_, DbState>) -> Result<u64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let max_pos: i64 = tx.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM playlist_tracks WHERE playlist_id = ?1",
        &[&playlist_id], |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    let mut pos = max_pos + 1;
    let mut added: u64 = 0;
    for path in &paths {
        let n= tx.execute(
            "INSERT OR IGNORE INTO playlist_tracks (playlist_id, path, position) VALUES (?1, ?2, ?3)",
            params![playlist_id, path, pos]
        ).map_err(|e| e.to_string())?;
        if n > 0 {
            pos += 1;
            added += 1;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(added)
}

#[tauri::command]
pub fn remove_from_playlist(playlist_id: i64, paths: Vec<String>, state: tauri::State<'_, DbState>) -> Result<u64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let mut removed: u64 = 0;

    for chunk in paths.chunks(500) {
        let placeholders: String = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "DELETE FROM playlist_tracks WHERE playlist_id = ? AND path IN ({})",
            placeholders
        );
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params_vec.push(Box::new(playlist_id));
        for path in chunk {
            params_vec.push(Box::new(path.clone()));
        }
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let n = tx.execute(&sql, params_refs.as_slice()).map_err(|e| e.to_string())?;
        removed += n as u64;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(removed)
}

#[tauri::command]
pub fn clear_playlist(playlist_id: i64, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlist_tracks WHERE playlist_id = ?1", params![playlist_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_tracks_by_folder(folder_path: String, state: tauri::State<'_, DbState>) -> Result<u64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let pattern = format!("{}%", folder_path);
    let removed = conn.execute(
        "DELETE FROM playlist_tracks WHERE path LIKE ?1",
        params![pattern]
    ).map_err(|e| e.to_string())?;
    Ok(removed as u64)
}

#[tauri::command]
pub fn get_playlist_tracks(playlist_id: i64, state: tauri::State<'_, DbState>) -> Result<Vec<PlaylistTrack>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT pt.path, pt.position,
                COALESCE(tc.file_name, '') as file_name,
                tc.title, tc.artist, tc.album,
                COALESCE(tc.duration, 0) as duration
        FROM playlist_tracks pt
        LEFT JOIN track_cache tc ON tc.path = pt.path
        WHERE pt.playlist_id = ?1
        ORDER BY pt.position"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![playlist_id], |row| {
        Ok(PlaylistTrack {
            path: row.get(0)?,
            position: row.get(1)?,
            file_name: row.get(2)?,
            title: row.get(3)?,
            artist: row.get(4)?,
            album: row.get(5)?,
            duration: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_cached_tracks_for_paths(paths: Vec<String>, state: tauri::State<'_, DbState>) -> Result<Vec<CachedTrack>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    for chunk in paths.chunks(500) {
        let placeholders: String = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache WHERE path IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let params: Vec<&dyn rusqlite::types::ToSql> = chunk.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok(CachedTrack {
                path: row.get(0)?,
                file_name: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
                duration: row.get(5)?,
                track_number: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?;
        for row in rows {
            if let Ok(track) = row {
                result.push(track);
            }
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn cache_tracks(tracks: Vec<CachedTrack>, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    for track in &tracks {
        tx.execute(
            "INSERT OR REPLACE INTO track_cache (path, file_name, title, artist, album, duration, track_number)
            VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![track.path, track.file_name, track.title, track.artist, track.album, track.duration, track.track_number]
        ).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn clear_track_cache(state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM track_cache", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cleanup_stale_cache(state: tauri::State<'_, DbState>) -> Result<u64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT path FROM track_cache").map_err(|e| e.to_string())?;
    let paths: Vec<String> = stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let stale: Vec<String> = paths.into_par_iter()
        .filter(|p| !std::path::Path::new(p.as_str()).exists())
        .collect();
    let count = stale.len() as u64;

    if count > 0 {
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        for chunk in stale.chunks(500) {
            let placeholders: String = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!("DELETE FROM track_cache WHERE path IN ({})", placeholders);
            let mut del_stmt = tx.prepare(&sql).map_err(|e| e.to_string())?;
            let params: Vec<&dyn rusqlite::types::ToSql> = chunk.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
            del_stmt.execute(params.as_slice()).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
    }

    Ok(count)
}

#[tauri::command]
pub fn get_artists(state: tauri::State<'_, DbState>) -> Result<Vec<ArtistGroup>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT COALESCE(artist, '未知歌手') as artist, COUNT(*) as track_count 
        FROM track_cache 
        GROUP BY COALESCE(artist, '未知歌手') 
        ORDER BY artist COLLATE NOCASE"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(ArtistGroup {
            artist: row.get(0)?,
            track_count: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_albums(state: tauri::State<'_, DbState>) -> Result<Vec<AlbumGroup>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT COALESCE(album, '未知专辑') as album, artist, COUNT(*) as track_count 
        FROM track_cache 
        GROUP BY COALESCE(album, '未知专辑') 
        ORDER BY album COLLATE NOCASE"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(AlbumGroup {
            album: row.get(0)?,
            artist: row.get(1)?,
            track_count: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_tracks_by_artist(artist: String, state: tauri::State<'_, DbState>) -> Result<Vec<CachedTrack>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache 
        WHERE COALESCE(artist, '未知歌手') = ?1 
        ORDER BY album, track_number, title, file_name"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![artist], |row| {
        Ok(CachedTrack {
            path: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration: row.get(5)?,
            track_number: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn search_tracks(query: String, state: tauri::State<'_, DbState>) -> Result<Vec<CachedTrack>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache 
        WHERE title LIKE ?1 COLLATE NOCASE OR (title IS NULL AND file_name LIKE ?1 COLLATE NOCASE) 
        ORDER BY title, file_name 
        LIMIT 200"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![pattern], |row| {
        Ok(CachedTrack {
            path: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration: row.get(5)?,
            track_number: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_tracks_by_album(album: String, state: tauri::State<'_, DbState>) -> Result<Vec<CachedTrack>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache 
        WHERE COALESCE(album, '未知专辑') = ?1 
        ORDER BY track_number, title, file_name"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![album], |row| {
        Ok(CachedTrack {
            path: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration: row.get(5)?,
            track_number: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_setting(key: String, state: tauri::State<'_, DbState>) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        &[&key],
        |row| row.get(0),
    );
    match result {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_setting(key: String, value: String, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_random_tracks(count: i64, state: tauri::State<'_, DbState>) -> Result<Vec<CachedTrack>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache 
        ORDER BY RANDOM() 
        LIMIT ?1"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![count], |row| {
        Ok(CachedTrack {
            path: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration: row.get(5)?,
            track_number: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}