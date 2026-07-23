#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD,
};

#[tauri::command]
pub fn get_system_accent_color() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let sub_key: Vec<u16> = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent\0"
            .encode_utf16()
            .collect();
        let value_name: Vec<u16> = "AccentColorMenu\0".encode_utf16().collect();

        let mut data: u32 = 0;
        let mut data_size: u32 = std::mem::size_of::<u32>() as u32;

        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                sub_key.as_ptr(),
                value_name.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                &mut data as *mut u32 as *mut std::ffi::c_void,
                &mut data_size,
            )
        };

        if result == 0 {
            let b = (data >> 16) & 0xff;
            let g = (data >> 8) & 0xff;
            let r = data & 0xff;
            return Ok(format!("#{:02x}{:02x}{:02x}", r, g, b));
        }
    }
    
    Ok("#007aff".to_string())
}