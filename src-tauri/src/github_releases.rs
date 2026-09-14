use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaAsset {
    pub name: String,
    pub download_url: String,
    pub size_bytes: u64,
    pub asset_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaBuildRelease {
    pub tag_name: String,
    pub release_name: String,
    pub published_at: String,
    pub html_url: String,
    pub body: String,
    pub vulkan_win_url: String,
    pub cpu_win_url: String,
    pub cuda_win_url: String,
    pub cuda_12_4_win_url: String,
    pub cuda_13_3_win_url: String,
    pub hip_win_url: String,
    pub sycl_win_url: String,
    pub assets: Vec<LlamaAsset>,
}

#[tauri::command]
pub async fn fetch_llama_releases() -> Result<Vec<LlamaBuildRelease>, String> {
    let client = reqwest::Client::builder()
        .user_agent("ollama-lite-desktop-app")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/ggml-org/llama.cpp/releases?per_page=15")
        .send()
        .await
        .map_err(|e| format!("Failed to reach GitHub API: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned HTTP status: {}", resp.status()));
    }

    let releases_json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub JSON: {}", e))?;

    let mut list = Vec::new();

    if let Some(arr) = releases_json.as_array() {
        for rel in arr {
            let tag_name = rel["tag_name"].as_str().unwrap_or("").to_string();
            let release_name = rel["name"].as_str().unwrap_or(&tag_name).to_string();
            let published_at = rel["published_at"].as_str().unwrap_or("").to_string();
            let html_url = rel["html_url"].as_str().unwrap_or("").to_string();
            let body = rel["body"].as_str().unwrap_or("").to_string();

            let mut vulkan_win_url = String::new();
            let mut cpu_win_url = String::new();
            let mut cuda_win_url = String::new();
            let mut cuda_12_4_win_url = String::new();
            let mut cuda_13_3_win_url = String::new();
            let mut hip_win_url = String::new();
            let mut sycl_win_url = String::new();

            let mut asset_list = Vec::new();

            if let Some(assets) = rel["assets"].as_array() {
                for asset in assets {
                    let name = asset["name"].as_str().unwrap_or("").to_string();
                    let name_lower = name.to_lowercase();
                    let download_url = asset["browser_download_url"].as_str().unwrap_or("").to_string();
                    let size_bytes = asset["size"].as_u64().unwrap_or(0);

                    if name_lower.contains("win") && name_lower.ends_with(".zip") {
                        let mut asset_type = "win_other".to_string();

                        if name_lower.contains("vulkan") {
                            vulkan_win_url = download_url.clone();
                            asset_type = "vulkan".to_string();
                        } else if name_lower.contains("cpu") || name_lower.contains("avx2") {
                            cpu_win_url = download_url.clone();
                            asset_type = "cpu".to_string();
                        } else if name_lower.contains("cuda-12.4") {
                            cuda_12_4_win_url = download_url.clone();
                            cuda_win_url = download_url.clone();
                            asset_type = "cuda_12_4".to_string();
                        } else if name_lower.contains("cuda-13.3") || name_lower.contains("cuda-13") {
                            cuda_13_3_win_url = download_url.clone();
                            if cuda_win_url.is_empty() {
                                cuda_win_url = download_url.clone();
                            }
                            asset_type = "cuda_13_3".to_string();
                        } else if name_lower.contains("hip") || name_lower.contains("radeon") {
                            hip_win_url = download_url.clone();
                            asset_type = "hip_radeon".to_string();
                        } else if name_lower.contains("sycl") {
                            sycl_win_url = download_url.clone();
                            asset_type = "sycl".to_string();
                        }

                        asset_list.push(LlamaAsset {
                            name,
                            download_url,
                            size_bytes,
                            asset_type,
                        });
                    }
                }
            }

            if vulkan_win_url.is_empty() {
                vulkan_win_url = format!(
                    "https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-win-vulkan-x64.zip",
                    tag_name, tag_name
                );
            }
            if cpu_win_url.is_empty() {
                cpu_win_url = format!(
                    "https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-win-cpu-x64.zip",
                    tag_name, tag_name
                );
            }

            if !tag_name.is_empty() {
                list.push(LlamaBuildRelease {
                    tag_name,
                    release_name,
                    published_at,
                    html_url,
                    body,
                    vulkan_win_url,
                    cpu_win_url,
                    cuda_win_url,
                    cuda_12_4_win_url,
                    cuda_13_3_win_url,
                    hip_win_url,
                    sycl_win_url,
                    assets: asset_list,
                });
            }
        }
    }

    Ok(list)
}

#[tauri::command]
pub async fn check_latest_build_number() -> Result<LlamaBuildRelease, String> {
    let releases = fetch_llama_releases().await?;
    releases
        .into_iter()
        .next()
        .ok_or_else(|| "No releases found on ggml-org/llama.cpp".to_string())
}
