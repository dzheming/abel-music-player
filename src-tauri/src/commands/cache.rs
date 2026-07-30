use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::database::DbState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub path: String,
    pub file_name: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
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
pub fn get_cached_tracks_for_paths(paths: Vec<String>, state: tauri::State<'_, DbState>) -> Result<Vec<TrackMetadata>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    for chunk in paths.chunks(super::SQL_BATCH_SIZE) {
        let placeholders: String = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache WHERE path IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let params: Vec<&dyn rusqlite::types::ToSql> = chunk.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok(TrackMetadata {
                path: row.get(0)?,
                file_name: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
                duration: row.get(5)?,
                track_number: row.get(6)?,
                cover: None,
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
pub fn cache_tracks(tracks: Vec<TrackMetadata>, state: tauri::State<'_, DbState>) -> Result<(), String> {
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
        for chunk in stale.chunks(super::SQL_BATCH_SIZE) {
            let placeholders: String = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!("DELETE FROM track_cache WHERE path IN ({})", placeholders);
            let mut del_stmt = tx.prepare(&sql).map_err(|e| e.to_string())?;
            let params: Vec<&dyn rusqlite::types::ToSql> = chunk.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
            del_stmt.execute(params.as_slice()).map_err(|e| e.to_string())?;

            let sql2 = format!("DELETE FROM playlist_tracks WHERE path IN ({})", placeholders);
            let mut del_stmt2 = tx.prepare(&sql2).map_err(|e| e.to_string())?;
            del_stmt2.execute(params.as_slice()).map_err(|e| e.to_string())?;
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
        FROM track_cache WHERE excluded = 0
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
        FROM track_cache WHERE excluded = 0
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
pub fn get_tracks_by_artist(artist: String, state: tauri::State<'_, DbState>) -> Result<Vec<TrackMetadata>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache
        WHERE COALESCE(artist, '未知歌手') = ?1  AND excluded = 0
        ORDER BY album, track_number, title, file_name"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![artist], |row| {
        Ok(TrackMetadata {
            path: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration: row.get(5)?,
            track_number: row.get(6)?,
            cover: None,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn search_tracks(query: String, state: tauri::State<'_, DbState>) -> Result<Vec<TrackMetadata>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache
        WHERE excluded = 0 AND (title LIKE ?1 COLLATE NOCASE OR (title IS NULL AND file_name LIKE ?1 COLLATE NOCASE))
        ORDER BY title, file_name
        LIMIT 200"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![pattern], |row| {
        Ok(TrackMetadata {
            path: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration: row.get(5)?,
            track_number: row.get(6)?,
            cover: None,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_tracks_by_album(album: String, state: tauri::State<'_, DbState>) -> Result<Vec<TrackMetadata>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache
        WHERE COALESCE(album, '未知专辑') = ?1 AND excluded = 0
        ORDER BY track_number, title, file_name"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![album], |row| {
        Ok(TrackMetadata {
            path: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration: row.get(5)?,
            track_number: row.get(6)?,
            cover: None,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_random_tracks(count: i64, state: tauri::State<'_, DbState>) -> Result<Vec<TrackMetadata>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT path, file_name, title, artist, album, duration, track_number FROM track_cache
        WHERE excluded = 0 ORDER BY RANDOM() LIMIT ?1"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![count], |row| {
        Ok(TrackMetadata {
            path: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration: row.get(5)?,
            track_number: row.get(6)?,
            cover: None,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}
