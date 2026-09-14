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
    pub current_os: String,
    pub recommended_url: String,
    pub recommended_label: String,
    pub fallback_url: String,
    pub fallback_label: String,
    pub vulkan_win_url: String,
    pub cpu_win_url: String,
    pub cuda_win_url: String,
    pub cuda_12_4_win_url: String,
    pub cuda_13_3_win_url: String,
    pub hip_win_url: String,
    pub sycl_win_url: String,
    pub assets: Vec<LlamaAsset>,
}

fn get_platform_info() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "windows"
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };

    (os, arch)
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

    let (target_os, target_arch) = get_platform_info();
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

            let mut recommended_url = String::new();
            let mut fallback_url = String::new();

            let mut asset_list = Vec::new();

            if let Some(assets) = rel["assets"].as_array() {
                for asset in assets {
                    let name = asset["name"].as_str().unwrap_or("").to_string();
                    let name_lower = name.to_lowercase();
                    let download_url = asset["browser_download_url"].as_str().unwrap_or("").to_string();
                    let size_bytes = asset["size"].as_u64().unwrap_or(0);

                    if !name_lower.ends_with(".zip") {
                        continue;
                    }

                    // Windows assets
                    if name_lower.contains("win") {
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

                        if target_os == "windows" {
                            asset_list.push(LlamaAsset {
                                name: name.clone(),
                                download_url: download_url.clone(),
                                size_bytes,
                                asset_type,
                            });
                        }
                    }

                    // macOS assets
                    if name_lower.contains("macos") || name_lower.contains("osx") {
                        let is_arm64 = name_lower.contains("arm64") || name_lower.contains("aarch64");
                        let asset_type = if is_arm64 {
                            "macos_arm64_metal".to_string()
                        } else {
                            "macos_x64_intel".to_string()
                        };

                        if (target_arch == "arm64" && is_arm64) || (target_arch != "arm64" && !is_arm64) {
                            if recommended_url.is_empty() {
                                recommended_url = download_url.clone();
                            }
                        } else if fallback_url.is_empty() {
                            fallback_url = download_url.clone();
                        }

                        if target_os == "macos" {
                            asset_list.push(LlamaAsset {
                                name: name.clone(),
                                download_url: download_url.clone(),
                                size_bytes,
                                asset_type,
                            });
                        }
                    }

                    // Linux assets
                    if name_lower.contains("ubuntu") || (name_lower.contains("linux") && !name_lower.contains("win")) {
                        let is_arm64 = name_lower.contains("arm64") || name_lower.contains("aarch64");
                        let asset_type = if is_arm64 {
                            "linux_arm64".to_string()
                        } else {
                            "linux_x64".to_string()
                        };

                        if (target_arch == "arm64" && is_arm64) || (target_arch != "arm64" && !is_arm64) {
                            if recommended_url.is_empty() {
                                recommended_url = download_url.clone();
                            }
                        } else if fallback_url.is_empty() {
                            fallback_url = download_url.clone();
                        }

                        if target_os == "linux" {
                            asset_list.push(LlamaAsset {
                                name: name.clone(),
                                download_url: download_url.clone(),
                                size_bytes,
                                asset_type,
                            });
                        }
                    }
                }
            }

            let (recommended_label, fallback_label) = match target_os {
                "macos" => {
                    let rec_lbl = if target_arch == "arm64" {
                        "Metal (Apple Silicon)"
                    } else {
                        "macOS Intel (x64)"
                    };
                    let fall_lbl = if target_arch == "arm64" {
                        "Intel x64 Fallback"
                    } else {
                        "Apple Silicon (arm64)"
                    };

                    if recommended_url.is_empty() {
                        recommended_url = format!(
                            "https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-macos-{}.zip",
                            tag_name, tag_name, target_arch
                        );
                    }
                    if fallback_url.is_empty() {
                        let other_arch = if target_arch == "arm64" { "x64" } else { "arm64" };
                        fallback_url = format!(
                            "https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-macos-{}.zip",
                            tag_name, tag_name, other_arch
                        );
                    }

                    vulkan_win_url = recommended_url.clone();
                    cpu_win_url = fallback_url.clone();

                    (rec_lbl.to_string(), fall_lbl.to_string())
                }
                "linux" => {
                    let rec_lbl = "Ubuntu / Linux (x64)";
                    let fall_lbl = "CPU Fallback";

                    if recommended_url.is_empty() {
                        recommended_url = format!(
                            "https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-ubuntu-x64.zip",
                            tag_name, tag_name
                        );
                    }
                    if fallback_url.is_empty() {
                        fallback_url = recommended_url.clone();
                    }

                    vulkan_win_url = recommended_url.clone();
                    cpu_win_url = fallback_url.clone();

                    (rec_lbl.to_string(), fall_lbl.to_string())
                }
                _ => {
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

                    recommended_url = vulkan_win_url.clone();
                    fallback_url = cpu_win_url.clone();

                    (
                        "Vulkan (GPU Accelerated)".to_string(),
                        "CPU (AVX2 Fallback)".to_string(),
                    )
                }
            };

            if !tag_name.is_empty() {
                list.push(LlamaBuildRelease {
                    tag_name,
                    release_name,
                    published_at,
                    html_url,
                    body,
                    current_os: target_os.to_string(),
                    recommended_url,
                    recommended_label,
                    fallback_url,
                    fallback_label,
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
