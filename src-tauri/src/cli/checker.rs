use super::installer::{get_ollama_cmd_path, get_ollama_ps1_path};

#[tauri::command]
pub fn check_cli_installed() -> bool {
    get_ollama_cmd_path().exists() && get_ollama_ps1_path().exists()
}

