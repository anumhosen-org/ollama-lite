#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

const REG_KEY_NAME: &str = "Ollama-Lite";

#[cfg(target_os = "linux")]
fn get_linux_autostart_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|p| p.join("autostart").join("ollama-lite.desktop"))
}

#[cfg(target_os = "macos")]
fn get_macos_launchagent_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|p| p.join("Library").join("LaunchAgents").join("com.anumhosen.ollamalite.plist"))
}

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

    #[cfg(target_os = "linux")]
    {
        if let Some(path) = get_linux_autostart_path() {
            if enabled {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
                let content = format!(
                    "[Desktop Entry]\nType=Application\nName=Ollama Lite\nComment=Ollama Lite Local LLM\nExec=\"{}\" --autostart\nTerminal=false\nStartupNotify=false\n",
                    exe_path.to_string_lossy()
                );
                std::fs::write(&path, content).map_err(|e| e.to_string())?;
            } else if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }
        return Ok(enabled);
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(path) = get_macos_launchagent_path() {
            if enabled {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
                let content = format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n    <key>Label</key>\n    <string>com.anumhosen.ollamalite</string>\n    <key>ProgramArguments</key>\n    <array>\n        <string>{}</string>\n        <string>--autostart</string>\n    </array>\n    <key>RunAtLoad</key>\n    <true/>\n</dict>\n</plist>\n",
                    exe_path.to_string_lossy()
                );
                std::fs::write(&path, content).map_err(|e| e.to_string())?;
            } else if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }
        return Ok(enabled);
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
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

    #[cfg(target_os = "linux")]
    {
        if let Some(path) = get_linux_autostart_path() {
            return Ok(path.exists());
        }
        return Ok(false);
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(path) = get_macos_launchagent_path() {
            return Ok(path.exists());
        }
        return Ok(false);
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    Ok(false)
}
