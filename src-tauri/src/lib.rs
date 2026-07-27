mod commands;

use commands::database::{
    init_db, DbState,
    create_playlist, delete_playlist, rename_playlist, get_playlists,
    add_to_playlist, remove_from_playlist, clear_playlist, get_playlist_tracks,
    cache_tracks, get_cached_tracks_for_paths, clear_track_cache, cleanup_stale_cache,
    get_artists, get_albums, get_tracks_by_artist, get_tracks_by_album,
    search_tracks, get_setting, set_setting, get_random_tracks,
};
use commands::lyrics::{download_lyrics, fetch_netease_lyric, read_local_lyrics, search_netease_lyrics};
use commands::metadata::{read_cover, read_metadata, read_metadata_batch};
use commands::portable::get_portable_dir;
use commands::scanner::{scan_folder_files, scan_folder_tree, scan_music_folder};
use commands::theme::get_system_accent_color;
use commands::window::{allow_sleep, prevent_sleep, reset_taskbar_icon, restore_window_state, save_window_state, set_taskbar_icon};

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    webview::WebviewWindowBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_music_folder,
            scan_folder_tree,
            scan_folder_files,
            get_system_accent_color,
            read_metadata,
            read_metadata_batch,
            read_cover,
            read_local_lyrics,
            download_lyrics,
            search_netease_lyrics,
            fetch_netease_lyric,
            set_taskbar_icon,
            reset_taskbar_icon,
            save_window_state,
            restore_window_state,
            create_playlist,
            delete_playlist,
            rename_playlist,
            get_playlists,
            add_to_playlist,
            remove_from_playlist,
            clear_playlist,
            get_playlist_tracks,
            cache_tracks,
            get_cached_tracks_for_paths,
            clear_track_cache,
            cleanup_stale_cache,
            get_artists,
            get_albums,
            get_tracks_by_artist,
            get_tracks_by_album,
            search_tracks,
            get_setting,
            set_setting,
            get_random_tracks,
            prevent_sleep,
            allow_sleep
        ])
        .setup(|app| {
            let conn = init_db();
            app.manage(DbState(Mutex::new(conn)));

            let webview_data = get_portable_dir().join("webview");
            WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                .title("Abel Music Player")
                .inner_size(1200.0, 800.0)
                .min_inner_size(900.0, 600.0)
                .decorations(false)
                .visible(false)
                .data_directory(webview_data)
                .build()?;

            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Abel Music Player")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
