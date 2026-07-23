use std::path::Path;
use serde::Serialize;
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "aac", "m4a", "wma"];

#[derive(Serialize, Clone)]
pub struct FolderNode {
    pub name: String,
    pub path: String,
    pub children: Vec<FolderNode>,
    pub audio_count: usize,
}

#[tauri::command]
pub fn scan_music_folder(path: String) -> Result<Vec<String>, String> {
    let dir = Path::new(&path);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("Path does not exist or is not a directory: {}", path));
    }

    let mut audio_files: Vec<String> = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if AUDIO_EXTENSIONS.contains(&ext_lower.as_str()) {
                    if let Some(path_str) = path.to_str() {
                        audio_files.push(path_str.to_string());
                    }
                }
            }
        }
    }

    audio_files.sort();
    Ok(audio_files)
}

#[tauri::command]
pub fn scan_folder_tree(path: String) -> Result<FolderNode, String> {
    let dir = Path::new(&path);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("Path does not exist or is not a directory: {}", path));
    }

    Ok(build_folder_node(dir))
}

fn build_folder_node(dir: &Path) -> FolderNode {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.to_string_lossy().to_string());

    let mut children: Vec<FolderNode> = Vec::new();
    let mut audio_count: usize = 0;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let child = build_folder_node(&entry_path);
                if child.audio_count > 0 || !child.children.is_empty() {
                    children.push(child);
                }
            } else if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if AUDIO_EXTENSIONS.contains(&ext_lower.as_str()) {
                        audio_count += 1;
                    }
                }
            }
        }
    }

    children.sort_by(|a, b| a.name.cmp(&b.name));

    FolderNode {
        name,
        path: dir.to_string_lossy().to_string(),
        children,
        audio_count,
    }
}

#[tauri::command]
pub fn scan_folder_files(path: String) -> Result<Vec<String>, String> {
    let dir = Path::new(&path);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("Path does not exist or is not a directory: {}", path));
    }

    let mut audio_files: Vec<String> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if AUDIO_EXTENSIONS.contains(&ext_lower.as_str()) {
                        if let Some(path_str) = entry_path.to_str() {
                            audio_files.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    audio_files.sort();
    Ok(audio_files)
}