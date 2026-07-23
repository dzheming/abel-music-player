use std::path::PathBuf;

pub fn get_portable_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("failed to get exe path");
    let dir = exe.parent().unwrap().join("abeldata");
    std::fs::create_dir_all(&dir).ok();
    dir
}