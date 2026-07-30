use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AUDIO_EXTENSIONS;
use super::database::DbState;

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn is_sub_path(path: &str, parent: &str) -> bool {
    let np = normalize_path(path);
    let pp = normalize_path(parent);
    np == pp || np.starts_with(&format!("{}/", pp))
}

fn is_audio_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        AUDIO_EXTENSIONS.contains(&ext.to_string_lossy().to_lowercase().as_str())
    } else {
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryFolder {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub parent_path: Option<String>,
    pub is_root: bool,
    pub excluded: bool,
    pub audio_count: i64,
}

#[derive(Debug, Serialize)]
pub struct LibraryFolderNode {
    pub path: String,
    pub name: String,
    pub audio_count: i64,
    pub children: Vec<LibraryFolderNode>,
}

#[tauri::command]
pub fn get_library_folders(state: tauri::State<'_, DbState>) -> Result<Vec<LibraryFolder>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, path, name, parent_path, is_root, excluded, audio_count FROM library_folders WHERE is_root = 1"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(LibraryFolder {
            id: row.get(0)?,
            path: row.get(1)?,
            name: row.get(2)?,
            parent_path: row.get(3)?,
            is_root: row.get::<_, i64>(4)? == 1,
            excluded: row.get::<_, i64>(5)? == 1,
            audio_count: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_folder_tree(root_path: String, state: tauri::State<'_, DbState>) -> Result<LibraryFolderNode, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let norm = normalize_path(&root_path);
    let mut stmt = conn.prepare(
        "SELECT path, name, parent_path, audio_count FROM library_folders WHERE (path = ?1 OR path LIKE ?2) AND excluded = 0"
    ).map_err(|e| e.to_string())?;
    let pattern = format!("{}/%", norm);

    let all_rows: Vec<(String, String, Option<String>, i64)> = stmt.query_map(
        params![norm, pattern],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    build_tree_from_rows(&norm, &all_rows)
}

fn build_tree_from_rows(root_path: &str, rows: &[(String, String, Option<String>, i64)]) -> Result<LibraryFolderNode, String> {
    let mut children_map: HashMap<String, Vec<&(String, String, Option<String>, i64)>> = HashMap::new();
    let mut root_row: Option<&(String, String, Option<String>, i64)> = None;

    for row in rows {
        if row.0 == root_path {
            root_row = Some(row);
        }
        if let Some(ref parent) = row.2 {
            children_map.entry(parent.clone()).or_default().push(row);
        }
    }

    let (name, audio_count) = root_row
        .map(|r| (r.1.clone(), r.3))
        .unwrap_or_else(|| {
            let name = std::path::Path::new(root_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| root_path.to_string());
            (name, 0)
        });

    fn build_node(path: &str, name: &str, audio_count: i64, children_map: &HashMap<String, Vec<&(String, String, Option<String>, i64)>>) -> LibraryFolderNode {
        let children = children_map.get(path)
            .map(|kids| {
                let mut nodes: Vec<LibraryFolderNode> = kids.iter()
                    .map(|k| build_node(&k.0, &k.1, k.3, children_map))
                    .collect();
                nodes.sort_by(|a, b| a.name.cmp(&b.name));
                nodes
            })
            .unwrap_or_default();

        LibraryFolderNode { path: path.to_string(), name: name.to_string(), audio_count, children }
    }

    Ok(build_node(root_path, &name, audio_count, &children_map))
}

#[tauri::command]
pub fn add_library_folder(path: String, state: tauri::State<'_, DbState>) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let norm = normalize_path(&path);

    let dir = std::path::Path::new(&path);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("Path does not exist or is not a directory: {}", path));
    }

    let mut stmt = conn.prepare("SELECT path FROM library_folders WHERE is_root = 1").map_err(|e| e.to_string())?;
    let roots: Vec<String> = stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    for root in &roots {
        if is_sub_path(&norm, root) && norm != *root {
            let pattern = format!("{}/%", norm);
            conn.execute(
                "UPDATE library_folders SET excluded = 0 WHERE path = ?1 OR path LIKE ?2",
                params![norm, pattern],
            ).map_err(|e| e.to_string())?;
            let track_pattern = format!("{}%", norm);
            conn.execute("UPDATE track_cache SET excluded = 0 WHERE path LIKE ?1", params![track_pattern])
                .map_err(|e| e.to_string())?;
            sync_folder_tree_inner(&conn, root)?;
            return Ok(root.clone());
        }
    }

    let name = dir.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| norm.clone());

    conn.execute(
        "INSERT OR IGNORE INTO library_folders (path, name, parent_path, is_root, excluded, audio_count) VALUES (?1, ?2, NULL, 1, 0, 0)",
        params![norm, name],
    ).map_err(|e| e.to_string())?;

    let track_pattern = format!("{}%", norm);
    conn.execute("UPDATE track_cache SET excluded = 0 WHERE path LIKE ?1", params![track_pattern])
        .map_err(|e| e.to_string())?;

    sync_folder_tree_inner(&conn, &norm)?;
    Ok(norm)
}

#[tauri::command]
pub fn remove_library_folder(path: String, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let norm = normalize_path(&path);
    let pattern = format!("{}/%", norm);
    conn.execute("DELETE FROM library_folders WHERE path = ?1 OR path LIKE ?2", params![norm, pattern])
        .map_err(|e| e.to_string())?;
    let track_pattern = format!("{}%", norm);
    conn.execute("DELETE FROM playlist_tracks WHERE path LIKE ?1", params![track_pattern])
        .map_err(|e| e.to_string())?;
    conn.execute("UPDATE track_cache SET excluded = 1 WHERE path LIKE ?1", params![track_pattern])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn exclude_folder(path: String, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let norm = normalize_path(&path);
    let pattern = format!("{}/%", norm);
    conn.execute("UPDATE library_folders SET excluded = 1 WHERE path = ?1 OR path LIKE ?2", params![norm, pattern])
        .map_err(|e| e.to_string())?;
    let track_pattern = format!("{}%", norm);
    conn.execute("DELETE FROM playlist_tracks WHERE path LIKE ?1", params![track_pattern])
        .map_err(|e| e.to_string())?;
    conn.execute("UPDATE track_cache SET excluded = 1 WHERE path LIKE ?1", params![track_pattern])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn restore_folder(path: String, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let norm = normalize_path(&path);
    let pattern = format!("{}/%", norm);
    conn.execute("UPDATE library_folders SET excluded = 0 WHERE path = ?1 OR path LIKE ?2", params![norm, pattern])
        .map_err(|e| e.to_string())?;
    let track_pattern = format!("{}%", norm);
    conn.execute("UPDATE track_cache SET excluded = 0 WHERE path LIKE ?1", params![track_pattern])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn sync_library_folder(root_path: String, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let norm = normalize_path(&root_path);
    sync_folder_tree_inner(&conn, &norm)
}

fn sync_folder_tree_inner(conn: &Connection, root_path: &str) -> Result<(), String> {
    let pattern = format!("{}/%", root_path);
    let mut stmt = conn.prepare(
        "SELECT path FROM library_folders WHERE (path LIKE ?1 OR path = ?2) AND excluded = 1"
    ).map_err(|e| e.to_string())?;
    let excluded: Vec<String> = stmt.query_map(params![pattern, root_path], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    let mut disk_folders: Vec<(String, String, Option<String>, i64)> = Vec::new();
    scan_dirs_recursive(std::path::Path::new(root_path), None, &excluded, &mut disk_folders);

    let mut stmt = conn.prepare(
        "SELECT path, excluded FROM library_folders WHERE path LIKE ?1 OR path = ?2"
    ).map_err(|e| e.to_string())?;
    let db_entries: HashMap<String, bool> = stmt.query_map(
        params![pattern, root_path],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? == 1))
    ).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    let disk_paths: std::collections::HashSet<&str> = disk_folders.iter().map(|f| f.0.as_str()).collect();

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    let to_delete: Vec<&str> = db_entries.iter()
        .filter(|(path, excluded)| !disk_paths.contains(path.as_str()) && !*excluded)
        .map(|(path, _)| path.as_str())
        .collect();

    if !to_delete.is_empty() {
        for chunk in to_delete.chunks(super::SQL_BATCH_SIZE) {
            let placeholders: String = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!("DELETE FROM library_folders WHERE path IN ({})", placeholders);
            let mut del_stmt = tx.prepare(&sql).map_err(|e| e.to_string())?;
            let params_list: Vec<&dyn rusqlite::types::ToSql> = chunk.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
            del_stmt.execute(params_list.as_slice()).map_err(|e| e.to_string())?;
        }
    }

    for (fpath, fname, fparent, faudio) in &disk_folders {
        if db_entries.contains_key(fpath) {
            tx.execute(
                "UPDATE library_folders SET audio_count = ?1 WHERE path = ?2 AND excluded = 0",
                params![faudio, fpath],
            ).map_err(|e| e.to_string())?;
        } else {
            let is_root = if fpath == root_path { 1i64 } else { 0i64 };
            tx.execute(
                "INSERT OR IGNORE INTO library_folders (path, name, parent_path, is_root, excluded, audio_count) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                params![fpath, fname, fparent, is_root, faudio],
            ).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn scan_dirs_recursive(
    dir: &std::path::Path,
    parent: Option<&str>,
    excluded: &[String],
    results: &mut Vec<(String, String, Option<String>, i64)>,
) -> i64 {
    if !dir.exists() || !dir.is_dir() {
        return 0;
    }

    let path_str = normalize_path(&dir.to_string_lossy());

    if excluded.iter().any(|ex| is_sub_path(&path_str, ex)) {
        return 0;
    }

    let name = dir.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.clone());

    let mut audio_count = 0i64;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                audio_count += scan_dirs_recursive(&entry_path, Some(&path_str), excluded, results);
            } else if entry_path.is_file() && is_audio_file(&entry_path) {
                audio_count += 1;
            }
        }
    }

    results.push((path_str.clone(), name, parent.map(|s| s.to_string()), audio_count));
    audio_count
}

pub fn get_excluded_folders_from_db(conn: &Connection) -> Vec<String> {
    conn.prepare("SELECT path FROM library_folders WHERE excluded = 1")
        .and_then(|mut stmt| {
            let rows: Vec<String> = stmt.query_map([], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        })
        .unwrap_or_default()
}
