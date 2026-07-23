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

    let url = format!(
        "https://lrclib.net/api/get?track_name={}&artist_name={}&album_name={}&duration={}",
        urlencoded(&title),
        urlencoded(&artist),
        urlencoded(&album),
        duration_secs
    );


    let response = client
        .get(&url)
        .header("User-Agent", "AbelMusicPlayer/0.1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        let lyrics = json
            .get("syncedLyrics")
            .and_then(|v| v.as_str())
            .or_else(|| json.get("plainLyrics").and_then(|v| v.as_str()));

        if let Some(lrc_content) = lyrics {
            let lrc_path = Path::new(&audio_path).with_extension("lrc");
            let _ = fs::write(&lrc_path, lrc_content);
            return Ok(Some(lrc_content.to_string()));
        }
    }

    let search_url = format!(
        "https://lrclib.net/api/search?track_name={}&artist_name={}",
        urlencoded(&title),
        urlencoded(&artist)
    );


    let response = client
        .get(&search_url)
        .header("User-Agent", "AbelMusicPlayer/0.1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        if let Some(results) = json.as_array() {
            if let Some(first) = results.first() {
                let lyrics = first
                    .get("syncedLyrics")
                    .and_then(|v| v.as_str())
                    .or_else(|| first.get("plainLyrics").and_then(|v| v.as_str()));

                if let Some(lrc_content) = lyrics {
                    let lrc_path = Path::new(&audio_path).with_extension("lrc");
                    let _ = fs::write(&lrc_path, lrc_content);
                    return Ok(Some(lrc_content.to_string()));
                }
            }
        }
    }

    if let Some(lrc_content) = try_netease(&client, &title, &artist).await {
        let lrc_path = Path::new(&audio_path).with_extension("lrc");
        let _ = fs::write(&lrc_path, &lrc_content);
        return Ok(Some(lrc_content));
    }

    Ok(None)
}

async fn try_netease(client: &reqwest::Client, title: &str, artist: &str) -> Option<String> {
    let query = if artist.is_empty() {
        title.to_string()
    } else {
        format!("{} {}", title, artist)
    };

    let search_url = format!(
        "https://music.163.com/api/search/get/web?s={}&type=1&limit=5",
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
    let song_id = songs.first()?.get("id")?.as_u64()?;

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