use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tauri::Emitter;
use super::engine_downloader::DownloadProgressPayload;
use super::gguf_inspector::ModelFileInfo;
use super::paths::get_models_dir;

#[tauri::command]
pub fn list_installed_models(custom_dir: Option<String>) -> Vec<ModelFileInfo> {
    let models_dir = if let Some(ref d) = custom_dir {
        let p = Path::new(d);
        if p.exists() {
            p.to_path_buf()
        } else {
            get_models_dir()
        }
    } else {
        get_models_dir()
    };

    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "gguf" {
                        let filename = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let size_gb = (size_bytes as f64) / (1024.0 * 1024.0 * 1024.0);

                        list.push(ModelFileInfo {
                            filename,
                            full_path: path.to_string_lossy().to_string(),
                            size_bytes,
                            size_gb: (size_gb * 100.0).round() / 100.0,
                            modified_time: format!(
                                "{:?}",
                                entry.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::now())
                            ),
                        });
                    }
                }
            }
        }
    }
    list
}

#[tauri::command]
pub fn delete_installed_model(filepath: String) -> Result<(), String> {
    let path = Path::new(&filepath);
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("Failed deleting model file: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn download_model_file(
    app: tauri::AppHandle,
    download_url: String,
    filename: String,
    custom_dir: Option<String>,
) -> Result<String, String> {
    let target_dir = if let Some(ref d) = custom_dir {
        let p = Path::new(d);
        if p.exists() {
            p.to_path_buf()
        } else {
            get_models_dir()
        }
    } else {
        get_models_dir()
    };

    let model_path = target_dir.join(&filename);

    let client = reqwest::Client::new();
    let res = client
        .get(&download_url)
        .header("User-Agent", "ollama-lite")
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let total_bytes = res.content_length().unwrap_or(0);
    let mut file = File::create(&model_path).map_err(|e| format!("Model file creation failed: {}", e))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    let start_time = std::time::Instant::now();
    let task_id = format!("model_download_{}", filename);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Error downloading model chunk: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Write failed: {}", e))?;
        downloaded += chunk.len() as u64;

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed_mbps = if elapsed > 0.0 {
            (downloaded as f64 / (1024.0 * 1024.0)) / elapsed
        } else {
            0.0
        };

        let percent = if total_bytes > 0 {
            (downloaded as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        let _ = app.emit(
            "download-progress",
            DownloadProgressPayload {
                task_id: task_id.clone(),
                task_type: "model".to_string(),
                filename: filename.clone(),
                bytes_downloaded: downloaded,
                total_bytes,
                progress_percent: (percent * 10.0).round() / 10.0,
                speed_mbps: (speed_mbps * 10.0).round() / 10.0,
                is_complete: false,
                error: None,
            },
        );
    }

    let _ = app.emit(
        "download-progress",
        DownloadProgressPayload {
            task_id,
            task_type: "model".to_string(),
            filename: filename.clone(),
            bytes_downloaded: downloaded,
            total_bytes,
            progress_percent: 100.0,
            speed_mbps: 0.0,
            is_complete: true,
            error: None,
        },
    );

    Ok(model_path.to_string_lossy().to_string())
}
