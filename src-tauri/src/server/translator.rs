use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaChatMessage>,
    #[serde(default = "default_stream_true")]
    pub stream: Option<bool>,
    #[serde(default)]
    pub options: Option<Value>,
    #[serde(default)]
    pub keep_alive: Option<Value>,
}

fn default_stream_true() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default = "default_stream_true")]
    pub stream: Option<bool>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub context: Option<Vec<i64>>,
    #[serde(default)]
    pub options: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaEmbedRequest {
    pub model: String,
    pub input: Value,
    #[serde(default)]
    pub truncate: Option<bool>,
    #[serde(default)]
    pub options: Option<Value>,
    #[serde(default)]
    pub keep_alive: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaLegacyEmbeddingsRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub options: Option<Value>,
    #[serde(default)]
    pub keep_alive: Option<Value>,
}

pub fn convert_ollama_chat_to_openai(req: &OllamaChatRequest) -> Value {
    let mut messages: Vec<Value> = Vec::new();

    for m in &req.messages {
        messages.push(json!({
            "role": m.role,
            "content": m.content
        }));
    }

    let mut body = json!({
        "messages": messages,
        "stream": req.stream.unwrap_or(true)
    });

    if let Some(opts) = &req.options {
        if let Some(temp) = opts.get("temperature") {
            body["temperature"] = temp.clone();
        }
        if let Some(top_p) = opts.get("top_p") {
            body["top_p"] = top_p.clone();
        }
        if let Some(top_k) = opts.get("top_k") {
            body["top_k"] = top_k.clone();
        }
        if let Some(max_tokens) = opts.get("num_predict") {
            body["max_tokens"] = max_tokens.clone();
        }
        if let Some(stop) = opts.get("stop") {
            body["stop"] = stop.clone();
        }
    }

    body
}

pub fn convert_ollama_generate_to_llamacpp(req: &OllamaGenerateRequest) -> Value {
    let mut prompt = req.prompt.clone();
    if let Some(sys) = &req.system {
        prompt = format!("System: {}\nUser: {}", sys, prompt);
    }

    let mut body = json!({
        "prompt": prompt,
        "stream": req.stream.unwrap_or(true)
    });

    if let Some(opts) = &req.options {
        if let Some(temp) = opts.get("temperature") {
            body["temperature"] = temp.clone();
        }
        if let Some(top_p) = opts.get("top_p") {
            body["top_p"] = top_p.clone();
        }
        if let Some(max_tokens) = opts.get("num_predict") {
            body["n_predict"] = max_tokens.clone();
        }
        if let Some(stop) = opts.get("stop") {
            body["stop"] = stop.clone();
        }
    }

    body
}

pub fn format_ollama_chat_chunk(model: &str, content: &str, is_done: bool) -> String {
    let now = chrono_iso_now();
    let val = json!({
        "model": model,
        "created_at": now,
        "message": {
            "role": "assistant",
            "content": content
        },
        "done": is_done
    });
    format!("{}\n", val.to_string())
}

pub fn format_ollama_generate_chunk(model: &str, response: &str, is_done: bool) -> String {
    let now = chrono_iso_now();
    let val = json!({
        "model": model,
        "created_at": now,
        "response": response,
        "done": is_done
    });
    format!("{}\n", val.to_string())
}

pub fn chrono_iso_now() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}Z", secs, millis)
}

pub fn format_pull_manifest_chunk() -> String {
    json!({ "status": "pulling manifest" }).to_string() + "\n"
}

pub fn format_pull_download_chunk(digest: &str, total: u64, completed: u64) -> String {
    json!({
        "status": "downloading",
        "digest": digest,
        "total": total,
        "completed": completed
    }).to_string() + "\n"
}

pub fn format_pull_verifying_chunk() -> String {
    json!({ "status": "verifying sha256 digest" }).to_string() + "\n"
}

pub fn format_pull_success_chunk() -> String {
    json!({ "status": "success" }).to_string() + "\n"
}

