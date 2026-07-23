#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::*;
        use windows_sys::Win32::UI::WindowsAndMessaging::*;

        let mutex_name: Vec<u16> = "AbelMusicPlayer_singleInstance_mutex\0"
            .encode_utf16()
            .collect();

        unsafe {
            let handle = CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr());
            let already_running = windows_sys::Win32::Foundation::GetLastError()
                == windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS;

            if already_running {
                let title: Vec<u16> = "Abel Music Player\0".encode_utf16().collect();
                let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
                if !hwnd.is_null() {
                    if IsIconic(hwnd) != 0 {
                        ShowWindow(hwnd, SW_RESTORE);
                    }
                    ShowWindow(hwnd, SW_SHOW);
                    SetForegroundWindow(hwnd);
                }
                return;
            }

            let _ = handle;
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::net::UnixListener;

        let sock_path = std::env::temp_dir().join("abel-music-player.sock");

        match UnixListener::bind(&sock_path) {
            Ok(listener) => {
                std::mem::forget(listener);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                match std::os::unix::net::UnixStream::connect(&sock_path) {
                    Ok(_) => return,
                    Err(_) => {
                        let _ = std::fs::remove_file(&sock_path);
                        match UnixListener::bind(&sock_path) {
                            Ok(listener) => std::mem::forget(listener),
                            Err(_) => return,
                        }
                    }
                }
            }
            Err(_) => return,
        }
    }

    abelmp_lib::run()
}