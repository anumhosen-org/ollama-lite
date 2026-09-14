use serde::{Deserialize, Serialize};
use std::path::Path;
use super::paths::get_models_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFileInfo {
    pub filename: String,
    pub full_path: String,
    pub size_bytes: u64,
    pub size_gb: f64,
    pub modified_time: String,
}

#[tauri::command]
pub fn inspect_gguf_file(filepath: String) -> Result<ModelFileInfo, String> {
    let path = Path::new(&filepath);
    if !path.exists() {
        return Err(format!("File does not exist: {}", filepath));
    }

    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let size_bytes = meta.len();
    let size_gb = (size_bytes as f64) / (1024.0 * 1024.0 * 1024.0);

    // If file is external, copy to models directory for self-contained persistence
    let target_path = get_models_dir().join(&filename);
    if !target_path.exists() && path != target_path {
        let _ = std::fs::copy(path, &target_path);
    }

    let final_path = if target_path.exists() {
        target_path.to_string_lossy().to_string()
    } else {
        filepath
    };

    Ok(ModelFileInfo {
        filename,
        full_path: final_path,
        size_bytes,
        size_gb: (size_gb * 100.0).round() / 100.0,
        modified_time: format!("{:?}", meta.modified().unwrap_or(std::time::SystemTime::now())),
    })
}
