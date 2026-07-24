use serde::Serialize;
use std::fs;
use std::path::Path;

#[tauri::command]
pub fn read_local_lyrics(audio_path: String) -> Result<Option<String>, String> {
    let path = Path::new(&audio_path);
    let lrc_path = path.with_extension("lrc");

    if lrc_path.exists() {
        let content = fs::read_to_string(&lrc_path).map_err(|e| e.to_string())?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn download_lyrics(
    title: String, 
    artist: String,
    album: String,
    duration: f64,
    audio_path: String,
) -> Result<Option<String>, String> {
    let client = reqwest::Client::new();
    let duration_secs = duration.round() as u64;
    
    if let Some(lrc_content) = try_netease(&client, &title, &artist, &album).await {
        let lrc_path = Path::new(&audio_path).with_extension("lrc");
        let _ = fs::write(&lrc_path, &lrc_content);
        return Ok(Some(lrc_content));
    }

    if let Some(lrc_content) = try_lrclib(&client, &title, &artist, &album, duration_secs).await {
        let lrc_path = Path::new(&audio_path).with_extension("lrc");
        let _ = fs::write(&lrc_path, &lrc_content);
        return Ok(Some(lrc_content));
    }
    Ok(None)
}

async fn try_lrclib(client: &reqwest::Client, title: &str, artist: &str, album: &str, duration_secs: u64) -> Option<String> {
    let url = format!(
        "https://lrclib.net/api/get?track_name={}&artist_name={}&album_name={}&duration={}",
        urlencoded(title),
        urlencoded(artist),
        urlencoded(album),
        duration_secs
    );

    let response = client
        .get(&url)
        .header("User-Agent", "AbelMusicPlayer/0.1.0")
        .send()
        .await
        .ok()?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await.ok()?;

        let lyrics = json
            .get("syncedLyrics")
            .and_then(|v| v.as_str());

        if let Some(lrc_content) = lyrics {
            return Some(lrc_content.to_string());
        }
    }

    let search_url = format!(
        "https://lrclib.net/api/search?track_name={}&artist_name={}",
        urlencoded(title),
        urlencoded(artist)
    );

    let response = client
        .get(&search_url)
        .header("User-Agent", "AbelMusicPlayer/0.1.0")
        .send()
        .await
        .ok()?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await.ok()?;

        if let Some(results) = json.as_array() {
            if let Some(first) = results.first() {
                let lyrics = first
                    .get("syncedLyrics")
                    .and_then(|v| v.as_str());

                if let Some(lrc_content) = lyrics {
                    return Some(lrc_content.to_string());
                }
            }
        }
    }

    None
}

async fn try_netease(client: &reqwest::Client, title: &str, artist: &str, album: &str) -> Option<String> {
    let query = if artist.is_empty() {
        title.to_string()
    } else {
        format!("{} {}", title, artist)
    };

    let search_url = format!(
        "https://music.163.com/api/search/get/web?s={}&type=1&limit=50",
        urlencoded(&query)
    );

    let resp = client
        .post(&search_url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Referer", "https://music.163.com/")
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = resp.json().await.ok()?;
    let songs = json.get("result")?.get("songs")?.as_array()?;

    let title_lower = title.to_lowercase();
    let artist_lower = artist.to_lowercase();
    let album_lower = album.to_lowercase();

    let matched = songs.iter().find(|song| {
        let name_match = song.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n.to_lowercase() == title_lower)
            .unwrap_or(false);
        let artist_match = artist.is_empty() || song.get("artists")
            .and_then(|a| a.as_array())
            .map(|arr| arr.iter().any(|a| {
                a.get("name").and_then(|n| n.as_str())
                    .map(|n| n.to_lowercase() == artist_lower)
                    .unwrap_or(false)
            }))
            .unwrap_or(false);
        let album_match = album.is_empty() || song.get("album")
            .and_then(|a| a.get("name"))
            .and_then(|n| n.as_str())
            .map(|n| {
                let n_lower = n.to_lowercase();
                n_lower.contains(&album_lower) || album_lower.contains(&n_lower)
            })
            .unwrap_or(false);
        name_match && artist_match && album_match
    });

    // if matched.is_some() {
    //     let s = matched.unwrap();
    //     eprintln!("try_netease exact matched: name {:?} artist {:?} album {:?}",
    //         s.get("name").and_then(|v| v.as_str()),
    //         s.get("artists").and_then(|a| a.as_array()).and_then(|arr| arr.first()).and_then(|a| a.get("name")).and_then(|v| v.as_str()),
    //         s.get("album").and_then(|a| a.get("name")).and_then(|v| v.as_str()),
    //     );
    // }

    let matched = matched.or_else(|| {
        if album.is_empty() { return None; }
        
        songs.iter().find(|song| {
            song.get("album")
            .and_then(|a| a.get("name"))
            .and_then(|n| n.as_str())
            .filter(|n| !n.is_empty())
            .map(|n| {
                let n_lower = n.to_lowercase();
                n_lower.contains(&album_lower) || album_lower.contains(&n_lower)
            })
            .unwrap_or(false)
        })
    });
    let song = matched.or_else(|| songs.first())?;
    let song_id = song.get("id")?.as_u64()?;

    let lyric_url = format!(
        "https://music.163.com/api/song/lyric?id={}&lv=1",
        song_id
    );

    let resp = client
        .get(&lyric_url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Referer", "https://music.163.com/")
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = resp.json().await.ok()?;
    let lrc = json.get("lrc")?.get("lyric")?.as_str()?;

    if lrc.is_empty() {
        return None;
    }

    Some(lrc.to_string())

}

#[derive(Serialize)]
pub struct NeteaseSearchResult {
    pub id: u64,
    pub name: String,
    pub artist: String,
    pub album: String,
}

#[tauri::command]
pub async fn search_netease_lyrics(query: String, artist_filter: String) -> Result<Vec<NeteaseSearchResult>, String> {
    let client = reqwest::Client::new();
    let search_url = format!(
        "https://music.163.com/api/search/get/web?s={}&type=1&limit=100",
        urlencoded(&query)
    );

    let resp = client
        .post(&search_url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Referer", "https://music.163.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let songs = json
        .get("result")
        .and_then(|r| r.get("songs"))
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();

    let filter_lower = artist_filter.to_lowercase();

    let results = songs.iter().filter_map(|song| {
        let id = song.get("id")?.as_u64()?;
        let name = song.get("name")?.as_str()?.to_string();
        let artist = song.get("artists")
            .and_then(|a| a.as_array())
            .map(|arr| arr.iter()
                .filter_map(|a| a.get("name").and_then(|n| n.as_str())) 
                .collect::<Vec<_>>()
                .join(" / "))
            .unwrap_or_default();
        let album = song.get("album")
            .and_then(|a| a.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();

        if !filter_lower.is_empty() {
            let artist_lower = artist.to_lowercase();
            if !artist_lower.contains(&filter_lower) && !filter_lower.contains(&artist_lower) {
                return None;
            }
        }
        Some(NeteaseSearchResult { id, name, artist, album })
    }).collect();
    
    Ok(results)
}

#[tauri::command]
pub async fn fetch_netease_lyric(song_id: u64, audio_path: String) -> Result<Option<String>, String> {
    let client = reqwest::Client::new();
    let lyric_url = format!(
        "https://music.163.com/api/song/lyric?id={}&lv=1",
        song_id
    );

    let resp = client
        .get(&lyric_url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Referer", "https://music.163.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let lrc = json.get("lrc")
        .and_then(|l| l.get("lyric"))
        .and_then(|l| l.as_str())
        .unwrap_or("");

    if lrc.is_empty() {
        return Ok(None);
    }

    let lrc_path = Path::new(&audio_path).with_extension("lrc");
    let _ = fs::write(&lrc_path, lrc);

    Ok(Some(lrc.to_string()))
}

fn urlencoded(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}