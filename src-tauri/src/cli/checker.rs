#[tauri::command]
pub fn check_cli_installed() -> bool {
    #[cfg(target_os = "windows")]
    {
        super::installer::get_ollama_cmd_path().exists() && super::installer::get_ollama_ps1_path().exists()
    }
    #[cfg(not(target_os = "windows"))]
    {
        super::installer::get_ollama_sh_path().exists()
    }
}


