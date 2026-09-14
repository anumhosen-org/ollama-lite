use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedModelSource {
    pub tag: String,
    pub filename: String,
    pub download_url: String,
    pub parameter_size: String,
    pub family: String,
}

/// Resolves standard Ollama tags, Hugging Face repo paths, or direct URLs into a downloadable GGUF spec.
pub fn resolve_model_source(identifier: &str) -> Option<ResolvedModelSource> {
    let clean = identifier.trim().to_lowercase();
    let norm = clean.strip_prefix("ollama.com/library/").unwrap_or(&clean);

    // 1. Direct HTTP/HTTPS URL
    if norm.starts_with("http://") || norm.starts_with("https://") {
        let url = norm.to_string();
        let filename = url
            .split('/')
            .last()
            .unwrap_or("downloaded_model.gguf")
            .split('?')
            .next()
            .unwrap_or("downloaded_model.gguf")
            .to_string();
        let final_filename = if filename.ends_with(".gguf") {
            filename
        } else {
            format!("{}.gguf", filename)
        };
        return Some(ResolvedModelSource {
            tag: identifier.to_string(),
            filename: final_filename,
            download_url: url,
            parameter_size: "Unknown".to_string(),
            family: "custom".to_string(),
        });
    }

    // 2. Hugging Face shortcut: "hf.co/user/repo/model.gguf" or "user/repo/model.gguf"
    if norm.starts_with("hf.co/") || (norm.contains('/') && norm.ends_with(".gguf")) {
        let hf_path = norm.strip_prefix("hf.co/").unwrap_or(norm);
        let parts: Vec<&str> = hf_path.split('/').collect();
        if parts.len() >= 3 {
            let user = parts[0];
            let repo = parts[1];
            let file = parts[2..].join("/");
            let url = format!("https://huggingface.co/{}/{}/resolve/main/{}", user, repo, file);
            let filename = parts.last().unwrap_or(&"model.gguf").to_string();
            return Some(ResolvedModelSource {
                tag: identifier.to_string(),
                filename,
                download_url: url,
                parameter_size: "Custom".to_string(),
                family: "custom".to_string(),
            });
        }
    }

    // 3. Known Ollama library catalog mappings to high-speed GGUF weights
    let (tag, filename, url, param, family) = match norm {
        // DeepSeek R1 Series
        "deepseek-r1" | "deepseek-r1:latest" | "deepseek-r1:7b" => (
            "deepseek-r1:7b",
            "DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf",
            "https://huggingface.co/unsloth/DeepSeek-R1-Distill-Qwen-7B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf",
            "7B",
            "deepseek",
        ),
        "deepseek-r1:1.5b" => (
            "deepseek-r1:1.5b",
            "DeepSeek-R1-Distill-Qwen-1.5B-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q4_K_M.gguf",
            "1.5B",
            "deepseek",
        ),
        "deepseek-r1:8b" => (
            "deepseek-r1:8b",
            "DeepSeek-R1-Distill-Llama-8B-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/DeepSeek-R1-Distill-Llama-8B-GGUF/resolve/main/DeepSeek-R1-Distill-Llama-8B-Q4_K_M.gguf",
            "8B",
            "deepseek",
        ),
        "deepseek-coder-v2" | "deepseek-coder-v2:latest" | "deepseek-coder-v2:16b" => (
            "deepseek-coder-v2:16b",
            "DeepSeek-Coder-V2-Lite-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/DeepSeek-Coder-V2-Lite-Instruct-GGUF/resolve/main/DeepSeek-Coder-V2-Lite-Instruct-Q4_K_M.gguf",
            "16B",
            "deepseek",
        ),

        // Llama 3.2 Series
        "llama3.2" | "llama3.2:latest" | "llama3.2:3b" => (
            "llama3.2:3b",
            "Llama-3.2-3B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf",
            "3B",
            "llama",
        ),
        "llama3.2:1b" => (
            "llama3.2:1b",
            "Llama-3.2-1B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf",
            "1B",
            "llama",
        ),

        // Llama 3.1 & Llama 2
        "llama3.1" | "llama3.1:latest" | "llama3.1:8b" => (
            "llama3.1:8b",
            "Llama-3.1-8B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/Llama-3.1-8B-Instruct-GGUF/resolve/main/Llama-3.1-8B-Instruct-Q4_K_M.gguf",
            "8B",
            "llama",
        ),
        "llama2" | "llama2:latest" | "llama2:7b" => (
            "llama2:7b",
            "llama-2-7b-chat.Q4_K_M.gguf",
            "https://huggingface.co/TheBloke/Llama-2-7B-Chat-GGUF/resolve/main/llama-2-7b-chat.Q4_K_M.gguf",
            "7B",
            "llama",
        ),

        // Qwen 2.5 & Qwen 2.5 Coder
        "qwen2.5-coder" | "qwen2.5-coder:latest" | "qwen2.5-coder:7b" => (
            "qwen2.5-coder:7b",
            "Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/main/qwen2.5-coder-7b-instruct-q4_k_m.gguf",
            "7B",
            "qwen2",
        ),
        "qwen2.5-coder:1.5b" => (
            "qwen2.5-coder:1.5b",
            "Qwen2.5-Coder-1.5B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
            "1.5B",
            "qwen2",
        ),
        "qwen2.5-coder:14b" => (
            "qwen2.5-coder:14b",
            "Qwen2.5-Coder-14B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/Qwen/Qwen2.5-Coder-14B-Instruct-GGUF/resolve/main/qwen2.5-coder-14b-instruct-q4_k_m.gguf",
            "14B",
            "qwen2",
        ),
        "qwen2.5" | "qwen2.5:latest" | "qwen2.5:7b" => (
            "qwen2.5:7b",
            "Qwen2.5-7B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf",
            "7B",
            "qwen2",
        ),
        "qwen2.5:3b" => (
            "qwen2.5:3b",
            "Qwen2.5-3B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf",
            "3B",
            "qwen2",
        ),
        "qwen2.5:0.5b" | "qwen2.5-0.5b" | "qwen2.5-0.5b-instruct-q4_k_m" | "qwen2.5-0.5b-instruct-q4_k_m:latest" => (
            "qwen2.5:0.5b",
            "Qwen2.5-0.5B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf",
            "0.5B",
            "qwen2",
        ),

        // Mistral Series
        "mistral" | "mistral:latest" | "mistral:7b" => (
            "mistral:7b",
            "Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
            "7B",
            "mistral",
        ),
        "mistral-nemo" | "mistral-nemo:latest" | "mistral-nemo:12b" => (
            "mistral-nemo:12b",
            "Mistral-Nemo-Instruct-2407-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/Mistral-Nemo-Instruct-2407-GGUF/resolve/main/Mistral-Nemo-Instruct-2407-Q4_K_M.gguf",
            "12B",
            "mistral",
        ),

        // Phi Series
        "phi4" | "phi4:latest" | "phi4:14b" => (
            "phi4:14b",
            "phi-4-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/phi-4-GGUF/resolve/main/phi-4-Q4_K_M.gguf",
            "14B",
            "phi3",
        ),
        "phi3.5" | "phi3.5:latest" | "phi3.5:3.8b" => (
            "phi3.5:3.8b",
            "Phi-3.5-mini-instruct-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/Phi-3.5-mini-instruct-GGUF/resolve/main/Phi-3.5-mini-instruct-Q4_K_M.gguf",
            "3.8B",
            "phi3",
        ),

        // Gemma Series
        "gemma2" | "gemma2:latest" | "gemma2:2b" => (
            "gemma2:2b",
            "gemma-2-2b-it-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf",
            "2B",
            "gemma2",
        ),
        "gemma2:9b" => (
            "gemma2:9b",
            "gemma-2-9b-it-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/gemma-2-9b-it-GGUF/resolve/main/gemma-2-9b-it-Q4_K_M.gguf",
            "9B",
            "gemma2",
        ),

        // Embeddings Model
        "nomic-embed-text" | "nomic-embed-text:latest" | "nomic-embed" => (
            "nomic-embed-text:latest",
            "nomic-embed-text-v1.5.Q4_K_M.gguf",
            "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf",
            "137M",
            "nomic-bert",
        ),

        // SmolLM & Hermes
        "smollm2" | "smollm2:latest" | "smollm2:1.7b" => (
            "smollm2:1.7b",
            "SmolLM2-1.7B-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/SmolLM2-1.7B-Instruct-GGUF/resolve/main/SmolLM2-1.7B-Instruct-Q4_K_M.gguf",
            "1.7B",
            "smollm",
        ),
        "smollm2:360m" => (
            "smollm2:360m",
            "SmolLM2-360M-Instruct-Q4_K_M.gguf",
            "https://huggingface.co/bartowski/SmolLM2-360M-Instruct-GGUF/resolve/main/SmolLM2-360M-Instruct-Q4_K_M.gguf",
            "360M",
            "smollm",
        ),
        "hermes3" | "hermes3:latest" | "hermes3:8b" => (
            "hermes3:8b",
            "Hermes-3-Llama-3.1-8B-Q4_K_M.gguf",
            "https://huggingface.co/NousResearch/Hermes-3-Llama-3.1-8B-GGUF/resolve/main/Hermes-3-Llama-3.1-8B-Q4_K_M.gguf",
            "8B",
            "llama",
        ),

        _ => return None,
    };

    Some(ResolvedModelSource {
        tag: tag.to_string(),
        filename: filename.to_string(),
        download_url: url.to_string(),
        parameter_size: param.to_string(),
        family: family.to_string(),
    })
}
