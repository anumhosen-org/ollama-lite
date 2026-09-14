use serde::{Deserialize, Serialize};
use std::process::Child;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarStatus {
    pub is_running: bool,
    pub current_model: Option<String>,
    pub host: String,
    pub port: u16,
    pub api_url: String,
    pub n_gpu_layers: i32,
    pub context_size: usize,
    pub threads: usize,
}

#[derive(Clone)]
pub struct SidecarState {
    pub child: Arc<Mutex<Option<Child>>>,
    pub current_model: Arc<Mutex<Option<String>>>,
    pub current_model_path: Arc<Mutex<Option<String>>>,
    pub host: Arc<Mutex<String>>,
    pub port: Arc<Mutex<u16>>,
    pub internal_port: Arc<Mutex<u16>>,
    pub n_gpu_layers: Arc<Mutex<i32>>,
    pub context_size: Arc<Mutex<usize>>,
    pub threads: Arc<Mutex<usize>>,
}

impl SidecarState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for SidecarState {
    fn default() -> Self {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(8);

        Self {
            child: Arc::new(Mutex::new(None)),
            current_model: Arc::new(Mutex::new(None)),
            current_model_path: Arc::new(Mutex::new(None)),
            host: Arc::new(Mutex::new("127.0.0.1".to_string())),
            port: Arc::new(Mutex::new(11434)),
            internal_port: Arc::new(Mutex::new(11435)),
            n_gpu_layers: Arc::new(Mutex::new(99)),
            context_size: Arc::new(Mutex::new(8192)),
            threads: Arc::new(Mutex::new(cores)),
        }
    }
}
