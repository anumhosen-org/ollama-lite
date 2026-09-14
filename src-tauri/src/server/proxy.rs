use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use futures_util::StreamExt;
use serde_json::{json, Value};
use crate::sidecar::state::SidecarState;
use crate::sidecar::process::ensure_model_running;
use super::translator::*;

pub async fn run_ollama_proxy(state: SidecarState, port: u16) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            println!("[Ollama Lite] REST API & Compatibility Server listening on http://{}", addr);
            l
        }
        Err(e) => {
            eprintln!("[Ollama Lite] Failed binding REST server to {}: {}", addr, e);
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, state_clone).await {
                        eprintln!("[Ollama Lite] HTTP Client error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("[Ollama Lite] Accept error: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: TcpStream, state: SidecarState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = vec![0u8; 8192];
    let bytes_read = stream.read(&mut buf).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&buf[..bytes_read]);
    let mut lines = request_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return Ok(()),
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0].to_uppercase();
    let path = parts[1];

    // Find headers and body offset
    let mut content_length: usize = 0;
    for line in request_str.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = line.split(':').nth(1) {
                content_length = val.trim().parse().unwrap_or(0);
            }
        }
    }

    // Extract initial body from first buffer
    let header_end = request_str.find("\r\n\r\n").map(|p| p + 4).unwrap_or(bytes_read);
    let mut body_bytes = buf[header_end..bytes_read].to_vec();

    // If body not fully read, read remainder
    while body_bytes.len() < content_length {
        let mut temp_buf = vec![0u8; 8192];
        let n = stream.read(&mut temp_buf).await?;
        if n == 0 {
            break;
        }
        body_bytes.extend_from_slice(&temp_buf[..n]);
    }

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    // CORS preflight
    if method == "OPTIONS" {
        send_cors_response(&mut stream).await?;
        return Ok(());
    }

    // Route handling
    match (method.as_str(), path) {
        ("GET", "/") | ("HEAD", "/") => {
            send_plain_response(&mut stream, 200, "Ollama is running\n").await?;
        }
        ("GET", "/api/version") | ("HEAD", "/api/version") => {
            let resp = json!({ "version": "0.1.0" });
            send_json_response(&mut stream, 200, &resp).await?;
        }
        ("GET", "/api/tags") | ("HEAD", "/api/tags") => {
            handle_get_tags(&mut stream).await?;
        }
        ("GET", "/api/ps") => {
            handle_get_ps(&mut stream, &state).await?;
        }
        ("POST", "/api/show") => {
            handle_post_show(&mut stream, &body_str).await?;
        }
        ("POST", "/api/chat") => {
            handle_post_chat(&mut stream, &body_str, &state).await?;
        }
        ("POST", "/api/generate") => {
            handle_post_generate(&mut stream, &body_str, &state).await?;
        }
        ("POST", "/api/pull") => {
            handle_post_pull(&mut stream, &body_str).await?;
        }
        ("DELETE", "/api/delete") => {
            handle_delete_model(&mut stream, &body_str, &state).await?;
        }
        ("POST", "/api/copy") => {
            handle_copy_model(&mut stream, &body_str).await?;
        }
        ("POST", "/api/embed") => {
            handle_post_embed(&mut stream, &body_str, &state, false).await?;
        }
        ("POST", "/api/embeddings") => {
            handle_post_embed(&mut stream, &body_str, &state, true).await?;
        }
        ("GET", "/v1/models") => {
            handle_get_v1_models(&mut stream).await?;
        }
        ("POST", p) if p.starts_with("/v1/") => {
            forward_to_internal(&mut stream, "POST", path, &body_bytes, &state).await?;
        }
        ("GET", p) if p.starts_with("/v1/") => {
            forward_to_internal(&mut stream, "GET", path, &[], &state).await?;
        }
        _ => {
            let err = json!({ "error": format!("Route not found: {} {}", method, path) });
            send_json_response(&mut stream, 404, &err).await?;
        }
    }

    Ok(())
}

async fn handle_get_tags(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let installed = crate::downloader::list_installed_models(None);
    let mut models = Vec::new();

    for m in installed {
        let tag_name = m.filename.replace(".gguf", ":latest");
        let family = if m.filename.to_lowercase().contains("qwen") {
            "qwen2".to_string()
        } else if m.filename.to_lowercase().contains("deepseek") {
            "deepseek".to_string()
        } else if m.filename.to_lowercase().contains("mistral") {
            "mistral".to_string()
        } else if m.filename.to_lowercase().contains("gemma") {
            "gemma2".to_string()
        } else {
            "llama".to_string()
        };

        models.push(json!({
            "name": tag_name,
            "model": tag_name,
            "modified_at": "2026-09-14T12:00:00Z",
            "size": m.size_bytes,
            "digest": format!("sha256:{:x}", m.size_bytes),
            "details": {
                "parent_model": "",
                "format": "gguf",
                "family": family.clone(),
                "families": [family],
                "parameter_size": format!("{:.1}B", m.size_gb * 1.5),
                "quantization_level": "Q4_K_M"
            }
        }));
    }

    let resp = json!({ "models": models });
    send_json_response(stream, 200, &resp).await
}

async fn handle_get_v1_models(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let installed = crate::downloader::list_installed_models(None);
    let mut data = Vec::new();

    for m in installed {
        let id = m.filename.replace(".gguf", ":latest");
        data.push(json!({
            "id": id,
            "object": "model",
            "created": 1726300000,
            "owned_by": "library"
        }));
    }

    let resp = json!({
        "object": "list",
        "data": data
    });
    send_json_response(stream, 200, &resp).await
}

async fn handle_get_ps(stream: &mut TcpStream, state: &SidecarState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let current_model = state.current_model.lock().unwrap().clone();
    let is_running = {
        let mut child = state.child.lock().unwrap();
        if let Some(ref mut c) = *child {
            matches!(c.try_wait(), Ok(None))
        } else {
            false
        }
    };

    let mut models = Vec::new();
    if is_running {
        if let Some(name) = current_model {
            let tag = name.replace(".gguf", ":latest");
            models.push(json!({
                "name": tag,
                "model": tag,
                "size": 4920739840u64,
                "digest": "sha256:active",
                "details": {
                    "format": "gguf",
                    "family": "llama",
                    "parameter_size": "7B",
                    "quantization_level": "Q4_K_M"
                },
                "expires_at": "0001-01-01T00:00:00Z",
                "size_vram": 4920739840u64
            }));
        }
    }

    let resp = json!({ "models": models });
    send_json_response(stream, 200, &resp).await
}

async fn handle_post_show(stream: &mut TcpStream, body_str: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let body_json: Value = serde_json::from_str(body_str).unwrap_or(json!({}));
    let model_name = body_json.get("model").and_then(|v| v.as_str()).unwrap_or("model");

    let installed = crate::downloader::list_installed_models(None);
    let matched = installed.iter().find(|m| {
        let fn_clean = m.filename.to_lowercase().replace(".gguf", "");
        let target_clean = model_name.to_lowercase().replace(":latest", "");
        fn_clean.contains(&target_clean) || target_clean.contains(&fn_clean)
    });

    let (family, param_size) = if let Some(m) = matched {
        let fam = if m.filename.to_lowercase().contains("qwen") {
            "qwen2"
        } else if m.filename.to_lowercase().contains("deepseek") {
            "deepseek"
        } else if m.filename.to_lowercase().contains("mistral") {
            "mistral"
        } else if m.filename.to_lowercase().contains("gemma") {
            "gemma2"
        } else {
            "llama"
        };
        (fam.to_string(), format!("{:.1}B", m.size_gb * 1.5))
    } else {
        ("llama".to_string(), "7B".to_string())
    };

    let resp = json!({
        "modelfile": format!("# Modelfile generated by Ollama Lite\nFROM {}\nTEMPLATE \"\"\"{{{{ if .System }}}}<|im_start|>system\n{{{{ .System }}}}<|im_end|>\n{{{{ end }}}}{{{{ if .Prompt }}}}<|im_start|>user\n{{{{ .Prompt }}}}<|im_end|>\n{{{{ end }}}}<|im_start|>assistant\n\"\"\"\nPARAMETER stop \"<|im_end|>\"\nPARAMETER stop \"<|endoftext|>\"\n", model_name),
        "parameters": "stop \"<|im_end|>\"\nstop \"<|endoftext|>\"",
        "template": "{{ if .System }}<|im_start|>system\n{{ .System }}<|im_end|>\n{{ end }}{{ if .Prompt }}<|im_start|>user\n{{ .Prompt }}<|im_end|>\n{{ end }}<|im_start|>assistant\n",
        "system": "",
        "license": "Apache-2.0 / Model License",
        "details": {
            "parent_model": "",
            "format": "gguf",
            "family": family.clone(),
            "families": [family],
            "parameter_size": param_size,
            "quantization_level": "Q4_K_M"
        },
        "capabilities": ["completion", "chat", "thinking"]
    });

    send_json_response(stream, 200, &resp).await
}

async fn handle_post_pull(
    stream: &mut TcpStream,
    body_str: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let body_json: Value = match serde_json::from_str(body_str) {
        Ok(v) => v,
        Err(e) => {
            let err = json!({ "error": format!("Invalid JSON: {}", e) });
            return send_json_response(stream, 400, &err).await;
        }
    };

    let model_name = match body_json.get("model").and_then(|v| v.as_str()) {
        Some(m) if !m.trim().is_empty() => m.trim(),
        _ => {
            let err = json!({ "error": "missing 'model' field" });
            return send_json_response(stream, 400, &err).await;
        }
    };

    let header = "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson; charset=utf-8\r\nTransfer-Encoding: chunked\r\nAccess-Control-Allow-Origin: *\r\n\r\n";

    async fn write_chunk(s: &mut TcpStream, line: &str) -> std::io::Result<()> {
        let hex_len = format!("{:X}\r\n", line.len());
        s.write_all(hex_len.as_bytes()).await?;
        s.write_all(line.as_bytes()).await?;
        s.write_all(b"\r\n").await?;
        s.flush().await
    }

    // Check if model is already installed
    let installed = crate::downloader::list_installed_models(None);
    let target_clean = model_name.to_lowercase().replace(":latest", "");
    let already_installed = installed.iter().find(|m| {
        let fn_clean = m.filename.to_lowercase().replace(".gguf", "");
        fn_clean == target_clean || fn_clean.contains(&target_clean) || target_clean.contains(&fn_clean)
    });

    if let Some(m) = already_installed {
        stream.write_all(header.as_bytes()).await?;
        let _ = write_chunk(stream, &crate::server::translator::format_pull_manifest_chunk()).await;
        let _ = write_chunk(stream, &crate::server::translator::format_pull_download_chunk("sha256:complete", m.size_bytes, m.size_bytes)).await;
        let _ = write_chunk(stream, &crate::server::translator::format_pull_verifying_chunk()).await;
        let _ = write_chunk(stream, &crate::server::translator::format_pull_success_chunk()).await;
        let _ = stream.write_all(b"0\r\n\r\n").await;
        return Ok(());
    }

    let resolved = match crate::server::model_resolver::resolve_model_source(model_name) {
        Some(r) => r,
        None => {
            let err = json!({
                "error": format!("Model '{}' not found in library. Specify a supported model (e.g. llama3.2, deepseek-r1, qwen2.5-coder) or a direct Hugging Face URL", model_name)
            });
            return send_json_response(stream, 404, &err).await;
        }
    };

    let target_dir = crate::downloader::paths::get_models_dir();
    let target_file = target_dir.join(&resolved.filename);

    stream.write_all(header.as_bytes()).await?;
    let _ = write_chunk(stream, &crate::server::translator::format_pull_manifest_chunk()).await;

    if target_file.exists() {
        let size = std::fs::metadata(&target_file).map(|m| m.len()).unwrap_or(0);
        let _ = write_chunk(stream, &crate::server::translator::format_pull_download_chunk("sha256:complete", size, size)).await;
        let _ = write_chunk(stream, &crate::server::translator::format_pull_verifying_chunk()).await;
        let _ = write_chunk(stream, &crate::server::translator::format_pull_success_chunk()).await;
        let _ = stream.write_all(b"0\r\n\r\n").await;
        return Ok(());
    }

    let client = reqwest::Client::new();
    let res = match client.get(&resolved.download_url).header("User-Agent", "ollama-lite").send().await {
        Ok(r) => r,
        Err(e) => {
            let err_line = json!({ "error": format!("Download connection error: {}", e) }).to_string() + "\n";
            let _ = write_chunk(stream, &err_line).await;
            let _ = stream.write_all(b"0\r\n\r\n").await;
            return Ok(());
        }
    };

    let total_bytes = res.content_length().unwrap_or(0);
    let mut file = match std::fs::File::create(&target_file) {
        Ok(f) => f,
        Err(e) => {
            let err_line = json!({ "error": format!("Failed creating file: {}", e) }).to_string() + "\n";
            let _ = write_chunk(stream, &err_line).await;
            let _ = stream.write_all(b"0\r\n\r\n").await;
            return Ok(());
        }
    };

    let mut downloaded: u64 = 0;
    let mut last_reported: u64 = 0;
    let mut stream_bytes = res.bytes_stream();
    let digest = format!("sha256:{:x}", total_bytes);

    while let Some(chunk_res) = stream_bytes.next().await {
        match chunk_res {
            Ok(chunk) => {
                use std::io::Write;
                if let Err(e) = file.write_all(&chunk) {
                    let err_line = json!({ "error": format!("File write error: {}", e) }).to_string() + "\n";
                    let _ = write_chunk(stream, &err_line).await;
                    let _ = stream.write_all(b"0\r\n\r\n").await;
                    return Ok(());
                }
                downloaded += chunk.len() as u64;

                if downloaded - last_reported > 5 * 1024 * 1024 || downloaded == total_bytes {
                    last_reported = downloaded;
                    let chunk_line = crate::server::translator::format_pull_download_chunk(&digest, total_bytes, downloaded);
                    let _ = write_chunk(stream, &chunk_line).await;
                }
            }
            Err(e) => {
                let err_line = json!({ "error": format!("Stream download error: {}", e) }).to_string() + "\n";
                let _ = write_chunk(stream, &err_line).await;
                let _ = stream.write_all(b"0\r\n\r\n").await;
                return Ok(());
            }
        }
    }

    let _ = write_chunk(stream, &crate::server::translator::format_pull_verifying_chunk()).await;
    let _ = write_chunk(stream, &crate::server::translator::format_pull_success_chunk()).await;
    let _ = stream.write_all(b"0\r\n\r\n").await;

    Ok(())
}

async fn handle_delete_model(
    stream: &mut TcpStream,
    body_str: &str,
    state: &SidecarState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let body_json: Value = serde_json::from_str(body_str).unwrap_or(json!({}));
    let model_name = match body_json.get("model").and_then(|v| v.as_str()) {
        Some(m) if !m.trim().is_empty() => m.trim(),
        _ => {
            let err = json!({ "error": "missing 'model' parameter" });
            return send_json_response(stream, 400, &err).await;
        }
    };

    let installed = crate::downloader::list_installed_models(None);
    let target = installed.iter().find(|m| {
        let fn_lower = m.filename.to_lowercase();
        let target_lower = model_name.to_lowercase().replace(":latest", "");
        fn_lower == target_lower || fn_lower == format!("{}.gguf", target_lower) || fn_lower.contains(&target_lower)
    });

    match target {
        Some(m) => {
            let current = state.current_model.lock().unwrap().clone();
            if let Some(ref c) = current {
                if c.contains(&m.filename) {
                    crate::sidecar::stop_sidecar_internal(state);
                }
            }

            if let Err(e) = std::fs::remove_file(&m.full_path) {
                let err = json!({ "error": format!("Failed deleting model: {}", e) });
                return send_json_response(stream, 500, &err).await;
            }

            let resp = json!({ "status": "success" });
            send_json_response(stream, 200, &resp).await
        }
        None => {
            let err = json!({ "error": format!("model '{}' not found", model_name) });
            send_json_response(stream, 404, &err).await
        }
    }
}

async fn handle_copy_model(
    stream: &mut TcpStream,
    body_str: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let body_json: Value = serde_json::from_str(body_str).unwrap_or(json!({}));
    let source = body_json.get("source").and_then(|v| v.as_str()).unwrap_or("");
    let destination = body_json.get("destination").and_then(|v| v.as_str()).unwrap_or("");

    if source.is_empty() || destination.is_empty() {
        let err = json!({ "error": "missing 'source' or 'destination' parameter" });
        return send_json_response(stream, 400, &err).await;
    }

    let installed = crate::downloader::list_installed_models(None);
    let src_match = installed.iter().find(|m| {
        m.filename.to_lowercase().contains(&source.to_lowercase().replace(":latest", ""))
    });

    match src_match {
        Some(src) => {
            let models_dir = crate::downloader::paths::get_models_dir();
            let dest_filename = if destination.ends_with(".gguf") {
                destination.to_string()
            } else {
                format!("{}.gguf", destination.replace(':', "_"))
            };
            let dest_path = models_dir.join(&dest_filename);

            if let Err(e) = std::fs::copy(&src.full_path, &dest_path) {
                let err = json!({ "error": format!("Failed copying model: {}", e) });
                return send_json_response(stream, 500, &err).await;
            }

            let resp = json!({ "status": "success" });
            send_json_response(stream, 200, &resp).await
        }
        None => {
            let err = json!({ "error": format!("source model '{}' not found", source) });
            send_json_response(stream, 404, &err).await
        }
    }
}

async fn handle_post_embed(
    stream: &mut TcpStream,
    body_str: &str,
    state: &SidecarState,
    is_legacy: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let body_json: Value = match serde_json::from_str(body_str) {
        Ok(v) => v,
        Err(e) => {
            let err = json!({ "error": format!("Invalid JSON: {}", e) });
            return send_json_response(stream, 400, &err).await;
        }
    };

    let model = body_json.get("model").and_then(|v| v.as_str()).unwrap_or("model");

    if let Err(e) = ensure_model_running(state, model) {
        let err = json!({ "error": format!("Model error: {}", e) });
        return send_json_response(stream, 500, &err).await;
    }

    let input_val = if is_legacy {
        body_json.get("prompt").cloned().unwrap_or(json!(""))
    } else {
        body_json.get("input").cloned().unwrap_or(json!(""))
    };

    let internal_port = *state.internal_port.lock().unwrap();
    let url = format!("http://127.0.0.1:{}/v1/embeddings", internal_port);

    let openai_body = json!({
        "model": model,
        "input": input_val
    });

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&openai_body).send().await;

    match resp {
        Ok(res) => {
            let res_json: Value = res.json().await.unwrap_or(json!({}));
            if is_legacy {
                let emb = res_json["data"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|item| item["embedding"].as_array())
                    .cloned()
                    .unwrap_or_default();
                let out = json!({ "embedding": emb });
                send_json_response(stream, 200, &out).await
            } else {
                let mut embeddings: Vec<Value> = Vec::new();
                if let Some(arr) = res_json["data"].as_array() {
                    for item in arr {
                        if let Some(emb) = item.get("embedding") {
                            embeddings.push(emb.clone());
                        }
                    }
                }
                let out = json!({
                    "model": model,
                    "embeddings": embeddings
                });
                send_json_response(stream, 200, &out).await
            }
        }
        Err(e) => {
            let err = json!({ "error": format!("Failed generating embeddings: {}", e) });
            send_json_response(stream, 502, &err).await
        }
    }
}

async fn handle_post_chat(
    stream: &mut TcpStream,
    body_str: &str,
    state: &SidecarState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let req: OllamaChatRequest = match serde_json::from_str(body_str) {
        Ok(r) => r,
        Err(e) => {
            let err = json!({ "error": format!("Invalid JSON: {}", e) });
            return send_json_response(stream, 400, &err).await;
        }
    };

    // Auto-boot model if needed
    if let Err(e) = ensure_model_running(state, &req.model) {
        let err = json!({ "error": format!("Model error: {}", e) });
        return send_json_response(stream, 500, &err).await;
    }

    let internal_port = *state.internal_port.lock().unwrap();
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", internal_port);

    let is_stream = req.stream.unwrap_or(true);
    let openai_body = convert_ollama_chat_to_openai(&req);

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&openai_body).send().await;

    let res = match resp {
        Ok(r) => r,
        Err(e) => {
            let err = json!({ "error": format!("Failed connecting to inference engine: {}", e) });
            return send_json_response(stream, 502, &err).await;
        }
    };

    if is_stream {
        // Stream chunked NDJSON
        let header = "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson; charset=utf-8\r\nTransfer-Encoding: chunked\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
        stream.write_all(header.as_bytes()).await?;

        let mut stream_bytes = res.bytes_stream();
        while let Some(chunk_res) = stream_bytes.next().await {
            if let Ok(chunk) = chunk_res {
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("data: ") {
                        let json_part = &trimmed[6..];
                        if json_part == "[DONE]" {
                            let final_nd = format_ollama_chat_chunk(&req.model, "", true);
                            write_chunk(stream, final_nd.as_bytes()).await?;
                            break;
                        }

                        if let Ok(val) = serde_json::from_str::<Value>(json_part) {
                            if let Some(content) = val["choices"][0]["delta"]["content"].as_str() {
                                let nd = format_ollama_chat_chunk(&req.model, content, false);
                                write_chunk(stream, nd.as_bytes()).await?;
                            }
                        }
                    }
                }
            }
        }

        // Write final 0-length chunk to terminate HTTP chunked stream
        stream.write_all(b"0\r\n\r\n").await?;
        stream.flush().await?;
    } else {
        // Non-streaming response
        let json_resp: Value = res.json().await.unwrap_or(json!({}));
        let content = json_resp["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");

        let ollama_res = json!({
            "model": req.model,
            "created_at": chrono_iso_now(),
            "message": {
                "role": "assistant",
                "content": content
            },
            "done": true
        });

        send_json_response(stream, 200, &ollama_res).await?;
    }

    Ok(())
}

async fn handle_post_generate(
    stream: &mut TcpStream,
    body_str: &str,
    state: &SidecarState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let req: OllamaGenerateRequest = match serde_json::from_str(body_str) {
        Ok(r) => r,
        Err(e) => {
            let err = json!({ "error": format!("Invalid JSON: {}", e) });
            return send_json_response(stream, 400, &err).await;
        }
    };

    if req.model.trim().is_empty() {
        crate::sidecar::stop_sidecar_internal(state);
        let resp = json!({ "response": "", "done": true });
        return send_json_response(stream, 200, &resp).await;
    }

    if let Err(e) = ensure_model_running(state, &req.model) {
        let err = json!({ "error": format!("Model error: {}", e) });
        return send_json_response(stream, 500, &err).await;
    }

    let internal_port = *state.internal_port.lock().unwrap();
    let url = format!("http://127.0.0.1:{}/completion", internal_port);

    let is_stream = req.stream.unwrap_or(true);
    let llama_body = convert_ollama_generate_to_llamacpp(&req);

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&llama_body).send().await;

    let res = match resp {
        Ok(r) => r,
        Err(e) => {
            let err = json!({ "error": format!("Failed connecting to inference engine: {}", e) });
            return send_json_response(stream, 502, &err).await;
        }
    };

    if is_stream {
        let header = "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson; charset=utf-8\r\nTransfer-Encoding: chunked\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
        stream.write_all(header.as_bytes()).await?;

        let mut stream_bytes = res.bytes_stream();
        while let Some(chunk_res) = stream_bytes.next().await {
            if let Ok(chunk) = chunk_res {
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("data: ") {
                        let json_part = &trimmed[6..];
                        if let Ok(val) = serde_json::from_str::<Value>(json_part) {
                            let content = val["content"].as_str().unwrap_or("");
                            let stop = val["stop"].as_bool().unwrap_or(false);
                            let nd = format_ollama_generate_chunk(&req.model, content, stop);
                            write_chunk(stream, nd.as_bytes()).await?;
                        }
                    }
                }
            }
        }

        stream.write_all(b"0\r\n\r\n").await?;
        stream.flush().await?;
    } else {
        let json_resp: Value = res.json().await.unwrap_or(json!({}));
        let content = json_resp["content"].as_str().unwrap_or("");

        let ollama_res = json!({
            "model": req.model,
            "created_at": chrono_iso_now(),
            "response": content,
            "done": true
        });

        send_json_response(stream, 200, &ollama_res).await?;
    }

    Ok(())
}

async fn forward_to_internal(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &[u8],
    state: &SidecarState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let internal_port = *state.internal_port.lock().unwrap();
    let url = format!("http://127.0.0.1:{}{}", internal_port, path);

    let client = reqwest::Client::new();
    let mut req_builder = match method {
        "POST" => client.post(&url),
        "GET" => client.get(&url),
        _ => client.post(&url),
    };

    if !body.is_empty() {
        req_builder = req_builder
            .header("Content-Type", "application/json")
            .body(body.to_vec());
    }

    let resp = match req_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            let err = json!({ "error": format!("Internal sidecar forwarding error: {}", e) });
            return send_json_response(stream, 502, &err).await;
        }
    };

    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json")
        .to_string();

    let is_streaming = content_type.contains("text/event-stream")
        || resp.headers().get("transfer-encoding").is_some();

    if is_streaming {
        let header = format!(
            "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nTransfer-Encoding: chunked\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            status, content_type
        );
        stream.write_all(header.as_bytes()).await?;

        let mut byte_stream = resp.bytes_stream();
        while let Some(chunk_res) = byte_stream.next().await {
            if let Ok(chunk) = chunk_res {
                write_chunk(stream, &chunk).await?;
            }
        }
        stream.write_all(b"0\r\n\r\n").await?;
        stream.flush().await?;
    } else {
        let bytes = resp.bytes().await.unwrap_or_default();
        let header = format!(
            "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            status,
            content_type,
            bytes.len()
        );
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(&bytes).await?;
        stream.flush().await?;
    }

    Ok(())
}

async fn write_chunk(stream: &mut TcpStream, data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if data.is_empty() {
        return Ok(());
    }
    let chunk_header = format!("{:x}\r\n", data.len());
    stream.write_all(chunk_header.as_bytes()).await?;
    stream.write_all(data).await?;
    stream.write_all(b"\r\n").await?;
    stream.flush().await?;
    Ok(())
}

async fn send_json_response(
    stream: &mut TcpStream,
    status: u16,
    val: &Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let body = val.to_string();
    let header = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\n\r\n",
        status,
        body.len()
    );
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(body.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn send_plain_response(
    stream: &mut TcpStream,
    status: u16,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let header = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        status,
        text.len()
    );
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(text.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn send_cors_response(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let header = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\nContent-Length: 0\r\n\r\n";
    stream.write_all(header.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
