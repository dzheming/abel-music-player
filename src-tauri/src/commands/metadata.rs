use base64::Engine;
use lofty::prelude::*;
use lofty::probe::Probe;
use rayon::prelude::*;
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
pub struct TrackMetadata {
    pub path: String,
    pub file_name: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
    pub cover: Option<String>,
    pub track_number: Option<u32>,
}

fn mime_type_to_str(mime: Option<&lofty::picture::MimeType>) -> &'static str {
    match mime {
        Some(lofty::picture::MimeType::Png) => "image/png",
        Some(lofty::picture::MimeType::Bmp) => "image/bmp",
        Some(lofty::picture::MimeType::Gif) => "image/gif",
        Some(lofty::picture::MimeType::Tiff) => "image/tiff",
        _ => "image/jpeg",
    }
}

fn encode_cover(pic: &lofty::picture::Picture) -> String {
    let mime = mime_type_to_str(pic.mime_type());
    let b64 = base64::engine::general_purpose::STANDARD.encode(pic.data());
    format!("data:{};base64,{}", mime, b64)
}

fn read_metadata_inner(path: &str, include_cover: bool) -> Option<TrackMetadata> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return None;
    }

    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let tagged_file = match Probe::open(file_path).and_then(|p| p.read()) {
        Ok(f) => Some(f),
        Err(_) => None,
    };

    let (duration, title, artist, album, cover, track_number) = if let Some(ref tf) = tagged_file {
        let duration = tf.properties().duration().as_secs_f64();
        let tag = tf.primary_tag().or_else(|| tf.first_tag());
        if let Some(tag) = tag {
            let title = tag.title().map(|s| s.to_string());
            let artist = tag.artist().map(|s| s.to_string());
            let album = tag.album().map(|s| s.to_string());
            let track_number = tag.track();

            let cover = if include_cover {
                tag.pictures().first().map(|pic| encode_cover(pic))
            } else {
                None
            };
            (duration, title, artist, album, cover, track_number)
        } else {
            (duration, None, None, None, None, None)
        }
    } else {
        (0.0, None, None, None, None, None)
    };

    Some(TrackMetadata {
        path: path.to_string(),
        file_name,
        title,
        artist,
        album,
        duration,
        cover,
        track_number,
    })
}

#[tauri::command]
pub fn read_metadata(path: String) -> Result<TrackMetadata, String> {
    read_metadata_inner(&path, true).ok_or_else(|| format!("Failed to read: {}", path))
}

#[tauri::command]
pub async fn read_metadata_batch(app: AppHandle, paths: Vec<String>) -> Vec<TrackMetadata> {
    const CHUNK_SIZE: usize = 100;
    
    let mut all_results: Vec<TrackMetadata> = Vec::with_capacity(paths.len());

    for chunk in paths.chunks(CHUNK_SIZE) {
        let chunk_owned: Vec<String> = chunk.to_vec();
        let results = match tokio::task::spawn_blocking(move || {
            chunk_owned.par_iter()
                .map(|p| read_metadata_inner(p, false))
                .collect::<Vec<_>>()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
        }).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[metadata] spawn_blocking failed: {}", e);
                continue;
            }
        };

        if !results.is_empty() {
            let _ = app.emit("metadata-batch-chunk", &results);
            all_results.extend(results);
        }
    }

    all_results
}

#[tauri::command]
pub fn read_cover(path: String) -> Result<Option<String>, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Ok(None);
    }

    let tagged_file = Probe::open(file_path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    let cover = tag.and_then(|t| {
        t.pictures().first().map(|pic| encode_cover(pic))
    });

    Ok(cover)
}