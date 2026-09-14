pub mod model_resolver;
pub mod proxy;
pub mod translator;

use crate::sidecar::state::SidecarState;

pub fn start_server_background(state: SidecarState, port: u16) {
    tauri::async_runtime::spawn(async move {
        proxy::run_ollama_proxy(state, port).await;
    });
}
