use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use tauri::Emitter;
use zip::ZipArchive;
use super::paths::get_bin_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub task_id: String,
    pub task_type: String, // "engine" | "model"
    pub filename: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub progress_percent: f64,
    pub speed_mbps: f64,
    pub is_complete: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub is_installed: bool,
    pub exe_path: Option<String>,
    pub version: Option<String>,
}

#[tauri::command]
pub fn check_binary_installed() -> Option<String> {
    let bin_dir = get_bin_dir();
    let candidates = vec![
        bin_dir.join("llama-server.exe"),
        bin_dir.join("llama-server-x86_64-pc-windows-msvc.exe"),
        bin_dir.join("resources").join("llama-server.exe"),
    ];

    for c in candidates {
        if c.exists() {
            return Some(c.to_string_lossy().to_string());
        }
    }
    None
}

#[tauri::command]
pub fn get_installed_engine_info() -> EngineInfo {
    if let Some(exe_path) = check_binary_installed() {
        EngineInfo {
            is_installed: true,
            exe_path: Some(exe_path),
            version: Some("llama.cpp release build".to_string()),
        }
    } else {
        EngineInfo {
            is_installed: false,
            exe_path: None,
            version: None,
        }
    }
}

#[tauri::command]
pub async fn download_llama_engine(
    app: tauri::AppHandle,
    download_url: String,
) -> Result<String, String> {
    let bin_dir = get_bin_dir();
    let zip_path = bin_dir.join("llama-server-engine.zip");

    let client = reqwest::Client::new();
    let res = client
        .get(&download_url)
        .header("User-Agent", "ollama-lite")
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let total_bytes = res.content_length().unwrap_or(0);
    let mut file = File::create(&zip_path).map_err(|e| format!("File creation failed: {}", e))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    let start_time = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Error downloading chunk: {}", e))?;
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
                task_id: "engine_download".to_string(),
                task_type: "engine".to_string(),
                filename: "llama-server-engine.zip".to_string(),
                bytes_downloaded: downloaded,
                total_bytes,
                progress_percent: (percent * 10.0).round() / 10.0,
                speed_mbps: (speed_mbps * 10.0).round() / 10.0,
                is_complete: false,
                error: None,
            },
        );
    }

    // Extract zip
    let file = File::open(&zip_path).map_err(|e| format!("Failed opening downloaded zip: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed parsing zip: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => bin_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            let _ = std::fs::create_dir_all(&outpath);
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    let _ = std::fs::create_dir_all(p);
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    let exe = check_binary_installed().ok_or_else(|| "Binary extracted but not found in destination".to_string())?;

    let _ = app.emit(
        "download-progress",
        DownloadProgressPayload {
            task_id: "engine_download".to_string(),
            task_type: "engine".to_string(),
            filename: "llama-server-engine.zip".to_string(),
            bytes_downloaded: downloaded,
            total_bytes,
            progress_percent: 100.0,
            speed_mbps: 0.0,
            is_complete: true,
            error: None,
        },
    );

    Ok(exe)
}
