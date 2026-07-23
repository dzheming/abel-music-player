#![windows_subsystem = "windows"]

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

    abelmp_lib::run()
}