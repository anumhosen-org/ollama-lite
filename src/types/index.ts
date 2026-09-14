export interface SidecarStatus {
  is_running: boolean;
  current_model: string | null;
  host: string;
  port: number;
  api_url: string;
  n_gpu_layers: number;
  context_size: number;
  threads: number;
}

export interface EngineInfo {
  is_installed: boolean;
  exe_path: string | null;
  version: string | null;
}

export interface HardwareInfo {
  cpu_name: string;
  cpu_cores: number;
  total_ram_gb: number;
  free_ram_gb: number;
  gpu_name: string;
  estimated_vram_gb: number;
  supports_vulkan: boolean;
  os_name: string;
}

export interface ModelRecommendation {
  max_params: string;
  recommended_quant: string;
  max_context_length: number;
  recommended_backend: string;
  reason: string;
}

export interface ModelFileInfo {
  filename: string;
  full_path: string;
  size_bytes: number;
  size_gb: number;
  modified_time: string;
}

export interface DownloadProgressPayload {
  task_id: string;
  task_type: string;
  filename: string;
  bytes_downloaded: number;
  total_bytes: number;
  progress_percent: number;
  speed_mbps: number;
  is_complete: boolean;
  error: string | null;
}

export interface LlamaBuildRelease {
  tag_name: string;
  release_name: string;
  published_at: string;
  html_url: string;
  body: string;
  vulkan_win_url: string;
  cpu_win_url: string;
  cuda_win_url: string;
  cuda_12_4_win_url: string;
  cuda_13_3_win_url: string;
  hip_win_url: string;
  sycl_win_url: string;
}

export interface ChatSession {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  thinking?: string;
  isThinking?: boolean;
  thinkingTime?: number;
  timestamp: string;
}

export interface AttachedFile {
  id: string;
  name: string;
  content: string;
  size: number;
}

export interface CuratedModel {
  name: string;
  tag: string;
  description: string;
  parameter_size: string;
  family: string;
  quantization: string;
  file_size_gb: number;
  download_url: string;
  filename: string;
  recommended_vram: string;
}
