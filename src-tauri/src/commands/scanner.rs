use std::path::Path;
use walkdir::WalkDir;

use super::database::DbState;
use super::library::{get_excluded_folders_from_db, normalize_path};
use super::{AUDIO_EXTENSIONS, MAX_SCAN_DEPTH};

fn is_excluded(path: &str, excluded: &[String]) -> bool {
    let norm = normalize_path(path);
    excluded.iter().any(|ex| {
        norm == *ex || norm.starts_with(&format!("{}/", ex))
    })
}

#[tauri::command]
pub fn scan_music_folder(path: String, state: tauri::State<'_, DbState>) -> Result<Vec<String>, String> {
    let dir = Path::new(&path);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("Path does not exist or is not a directory: {}", path));
    }

    let excluded = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        get_excluded_folders_from_db(&conn)
    };

    let mut audio_files: Vec<String> = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .max_depth(MAX_SCAN_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                !is_excluded(&e.path().to_string_lossy(), &excluded)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if AUDIO_EXTENSIONS.contains(&ext_lower.as_str()) {
                    audio_files.push(normalize_path(&path.to_string_lossy()));
                }
            }
        }
    }

    audio_files.sort();
    Ok(audio_files)
}
