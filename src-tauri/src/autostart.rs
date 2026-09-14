use winreg::enums::*;
use winreg::RegKey;

const REG_KEY_NAME: &str = "Ollama-Lite";

#[tauri::command]
pub fn set_autostart_enabled(enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let (key, _) = hkcu.create_subkey(path).map_err(|e| format!("Registry open error: {}", e))?;

        if enabled {
            if let Ok(exe_path) = std::env::current_exe() {
                let cmd_str = format!("\"{}\" --autostart", exe_path.to_string_lossy());
                key.set_value(REG_KEY_NAME, &cmd_str)
                    .map_err(|e| format!("Failed to write autostart registry key: {}", e))?;
            }
        } else {
            let _ = key.delete_value(REG_KEY_NAME);
        }
        return Ok(enabled);
    }

    #[cfg(not(target_os = "windows"))]
    Ok(enabled)
}

#[tauri::command]
pub fn is_autostart_enabled() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        if let Ok(key) = hkcu.open_subkey(path) {
            if let Ok(_val) = key.get_value::<String, _>(REG_KEY_NAME) {
                return Ok(true);
            }
        }
        return Ok(false);
    }

    #[cfg(not(target_os = "windows"))]
    Ok(false)
}
