use std::path::PathBuf;

pub fn get_portable_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let exe = std::env::current_exe().expect("failed to get exe path");
        let dir = exe.parent().unwrap().join("abeldata");
        std::fs::create_dir_all(&dir).ok();
        return dir;
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
        let dir = home.join("Library/Application Support/com.abel.music.player");
        std::fs::create_dir_all(&dir).ok();
        return dir;
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
        let dir = home.join(".local/share/abelmp");
        std::fs::create_dir_all(&dir).ok();
        dir
    }
}