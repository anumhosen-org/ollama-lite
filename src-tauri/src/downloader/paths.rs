use std::path::PathBuf;

pub fn get_app_dir() -> PathBuf {
    let base_dirs = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = base_dirs.join("ollama-lite");
    if !app_dir.exists() {
        let _ = std::fs::create_dir_all(&app_dir);
    }
    app_dir
}

pub fn get_models_dir() -> PathBuf {
    let models_dir = get_app_dir().join("models");
    if !models_dir.exists() {
        let _ = std::fs::create_dir_all(&models_dir);
    }
    models_dir
}

pub fn get_bin_dir() -> PathBuf {
    let bin_dir = get_app_dir().join("bin");
    if !bin_dir.exists() {
        let _ = std::fs::create_dir_all(&bin_dir);
    }
    bin_dir
}
