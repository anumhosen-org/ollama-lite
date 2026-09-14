use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: OllamaModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelDetails {
    pub format: String,
    pub family: String,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaTagsResponse {
    pub models: Vec<OllamaModel>,
}

#[tauri::command]
pub fn get_ollama_tags() -> Result<OllamaTagsResponse, String> {
    let installed_models = super::downloader::list_installed_models(None);
    let mut models = Vec::new();

    for m in installed_models {
        let tag_name = m.filename.replace(".gguf", ":latest");
        let family = if m.filename.to_lowercase().contains("qwen") {
            "qwen2".to_string()
        } else if m.filename.to_lowercase().contains("deepseek") {
            "deepseek".to_string()
        } else if m.filename.to_lowercase().contains("mistral") {
            "mistral".to_string()
        } else if m.filename.to_lowercase().contains("gemma") {
            "gemma2".to_string()
        } else if m.filename.to_lowercase().contains("phi") {
            "phi3".to_string()
        } else {
            "llama".to_string()
        };

        models.push(OllamaModel {
            name: tag_name,
            modified_at: "2026-09-14T12:00:00Z".to_string(),
            size: m.size_bytes,
            digest: format!("sha256:{:x}", m.size_bytes),
            details: OllamaModelDetails {
                format: "gguf".to_string(),
                family,
                parameter_size: format!("{:.1}B", m.size_gb * 1.5),
                quantization_level: "Q4_K_M".to_string(),
            },
        });
    }

    Ok(OllamaTagsResponse { models })
}
