use base64::Engine;
use lofty::prelude::*;
use lofty::probe::Probe;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
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

#[tauri::command]
pub fn read_metadata(path: String) -> Result<TrackMetadata, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File not found: {}", path));
    }

    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let tagged_file = Probe::open(file_path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    let properties = tagged_file.properties();
    let duration = properties.duration().as_secs_f64();

    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    let (title, artist, album, cover, track_number) = if let Some(tag) = tag {
        let title = tag.title().map(|s| s.to_string());
        let artist = tag.artist().map(|s| s.to_string());
        let album = tag.album().map(|s| s.to_string());
        let track_number = tag.track();

        let cover = tag.pictures().first().map(|pic| encode_cover(pic));

        (title, artist, album, cover, track_number)
    } else {
        (None, None, None, None, None)
    };

    Ok(TrackMetadata {
        path,
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
pub fn read_metadata_batch(paths: Vec<String>) -> Vec<TrackMetadata> {
    paths
        .into_iter()
        .filter_map(|p| read_metadata(p).ok())
        .collect()
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