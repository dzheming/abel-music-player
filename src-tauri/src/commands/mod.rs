pub mod cache;
pub mod database;
pub mod library;
pub mod lyrics;
pub mod metadata;
pub mod playlist;
pub mod portable;
pub mod scanner;
pub mod settings;
pub mod theme;
pub mod window;

pub const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "aac", "m4a", "wma"];
pub const SQL_BATCH_SIZE: usize = 500;
pub const READ_METADATA_BATCH: usize = 100;
pub const MAX_SCAN_DEPTH: usize = 32;
