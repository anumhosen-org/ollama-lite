use std::path::Path;
use std::process::Command;
use super::state::{SidecarState, SidecarStatus};

pub fn stop_sidecar_internal(state: &SidecarState) {
    let mut child_guard = state.child.lock().unwrap();
    if let Some(mut child) = child_guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let mut model_guard = state.current_model.lock().unwrap();
    *model_guard = None;
    let mut path_guard = state.current_model_path.lock().unwrap();
    *path_guard = None;
}

#[tauri::command]
pub fn stop_sidecar(state: tauri::State<'_, SidecarState>) -> Result<(), String> {
    stop_sidecar_internal(&state);
    Ok(())
}

#[tauri::command]
pub fn get_sidecar_status(state: tauri::State<'_, SidecarState>) -> SidecarStatus {
    let mut child_guard = state.child.lock().unwrap();

    let is_running = if let Some(ref mut child) = *child_guard {
        match child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    } else {
        false
    };

    let model = state.current_model.lock().unwrap().clone();
    let host = state.host.lock().unwrap().clone();
    let public_port = *state.port.lock().unwrap();
    let n_gpu_layers = *state.n_gpu_layers.lock().unwrap();
    let context_size = *state.context_size.lock().unwrap();
    let threads = *state.threads.lock().unwrap();

    let api_url = format!("http://{}:{}", host, public_port);

    SidecarStatus {
        is_running,
        current_model: model,
        host,
        port: public_port,
        api_url,
        n_gpu_layers,
        context_size,
        threads,
    }
}

pub fn start_sidecar_internal(
    state: &SidecarState,
    model_path: &str,
    host: Option<String>,
    public_port: Option<u16>,
    n_gpu_layers: Option<i32>,
    context_size: Option<usize>,
    threads: Option<usize>,
) -> Result<SidecarStatus, String> {
    stop_sidecar_internal(state);

    let exe_path = super::super::downloader::engine_downloader::check_binary_installed()
        .ok_or_else(|| "llama-server binary is not installed. Please download engine in the Engine tab first.".to_string())?;

    if !Path::new(model_path).exists() {
        return Err(format!("Model file not found: {}", model_path));
    }

    let default_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8);

    let target_host = host.unwrap_or_else(|| "127.0.0.1".to_string());
    let target_pub_port = public_port.unwrap_or(11434);
    let target_internal_port = *state.internal_port.lock().unwrap();
    let target_gpu_layers = n_gpu_layers.unwrap_or(99);
    let target_ctx = context_size.unwrap_or(8192);
    let target_threads = threads.unwrap_or(default_threads);

    let filename = Path::new(model_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut cmd = Command::new(&exe_path);
    cmd.arg("-m")
        .arg(model_path)
        .arg("--host")
        .arg(&target_host)
        .arg("--port")
        .arg(target_internal_port.to_string())
        .arg("-ngl")
        .arg(target_gpu_layers.to_string())
        .arg("-c")
        .arg(target_ctx.to_string())
        .arg("-t")
        .arg(target_threads.to_string());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed spawning llama-server process: {}", e))?;

    *state.child.lock().unwrap() = Some(child);
    *state.current_model.lock().unwrap() = Some(filename);
    *state.current_model_path.lock().unwrap() = Some(model_path.to_string());
    *state.host.lock().unwrap() = target_host.clone();
    *state.port.lock().unwrap() = target_pub_port;
    *state.n_gpu_layers.lock().unwrap() = target_gpu_layers;
    *state.context_size.lock().unwrap() = target_ctx;
    *state.threads.lock().unwrap() = target_threads;

    Ok(SidecarStatus {
        is_running: true,
        current_model: Some(model_path.to_string()),
        host: target_host.clone(),
        port: target_pub_port,
        api_url: format!("http://{}:{}", target_host, target_pub_port),
        n_gpu_layers: target_gpu_layers,
        context_size: target_ctx,
        threads: target_threads,
    })
}

#[tauri::command]
pub fn start_sidecar(
    state: tauri::State<'_, SidecarState>,
    model_path: String,
    host: Option<String>,
    port: Option<u16>,
    n_gpu_layers: Option<i32>,
    context_size: Option<usize>,
    threads: Option<usize>,
) -> Result<SidecarStatus, String> {
    start_sidecar_internal(&state, &model_path, host, port, n_gpu_layers, context_size, threads)
}

pub fn ensure_model_running(state: &SidecarState, model_hint: &str) -> Result<String, String> {
    // Check if child is running
    let is_running = {
        let mut child_guard = state.child.lock().unwrap();
        if let Some(ref mut child) = *child_guard {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    };

    let current_model_name = state.current_model.lock().unwrap().clone();

    // Clean model hint (strip tags like ":latest" or paths)
    let clean_hint = model_hint
        .replace(":latest", "")
        .to_lowercase();

    if is_running {
        if let Some(ref cur) = current_model_name {
            if clean_hint.is_empty() || cur.to_lowercase().contains(&clean_hint) {
                return Ok(cur.clone());
            }
        }
    }

    // Find installed model matching hint
    let installed = super::super::downloader::list_installed_models(None);
    if installed.is_empty() {
        return Err("No installed models found. Download or import a model in the Models tab first.".to_string());
    }

    let target_model = if clean_hint.is_empty() {
        &installed[0]
    } else {
        installed
            .iter()
            .find(|m| m.filename.to_lowercase().contains(&clean_hint))
            .unwrap_or(&installed[0])
    };

    start_sidecar_internal(state, &target_model.full_path, None, None, None, None, None)?;

    // Give llama-server a brief moment to initialize its internal socket
    std::thread::sleep(std::time::Duration::from_millis(800));

    Ok(target_model.filename.clone())
}
