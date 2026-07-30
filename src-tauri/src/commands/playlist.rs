use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::database::DbState;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    pub track_number: Option<u32>,
    pub position: i64,
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

    for chunk in paths.chunks(super::SQL_BATCH_SIZE) {
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
pub fn get_playlist_tracks(playlist_id: i64, state: tauri::State<'_, DbState>) -> Result<Vec<PlaylistTrack>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT pt.path, pt.position,
                COALESCE(tc.file_name, '') as file_name,
                tc.title, tc.artist, tc.album,
                COALESCE(tc.duration, 0) as duration,
                tc.track_number
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
            cover: None,
            track_number: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}
