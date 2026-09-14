import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  FiCpu,
  FiSquare,
  FiRotateCw,
  FiDownload,
  FiCheckCircle,
  FiAlertCircle,
  FiActivity,
} from "react-icons/fi";
import {
  EngineInfo,
  HardwareInfo,
  LlamaBuildRelease,
  ModelRecommendation,
  SidecarStatus,
} from "../../types";

interface EngineViewProps {
  status: SidecarStatus | null;
  onRefreshStatus: () => void;
  installedModelsCount: number;
}

export const EngineView: React.FC<EngineViewProps> = ({
  status,
  onRefreshStatus,
  installedModelsCount,
}) => {
  const [engineInfo, setEngineInfo] = useState<EngineInfo | null>(null);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [recommendation, setRecommendation] = useState<ModelRecommendation | null>(null);
  const [releases, setReleases] = useState<LlamaBuildRelease[]>([]);
  const [selectedBuildType, setSelectedBuildType] = useState<"vulkan" | "cpu" | "cuda">("vulkan");
  const [isDownloadingEngine, setIsDownloadingEngine] = useState(false);
  const [isLoadingReleases, setIsLoadingReleases] = useState(false);

  useEffect(() => {
    loadEngineAndHardware();
  }, []);

  const loadEngineAndHardware = async () => {
    try {
      const eng: EngineInfo = await invoke("get_installed_engine_info");
      setEngineInfo(eng);

      const hw: HardwareInfo = await invoke("get_hardware_info");
      setHardware(hw);

      const rec: ModelRecommendation = await invoke("get_hardware_recommendation", { info: hw });
      setRecommendation(rec);
    } catch (err) {
      console.error(err);
    }
  };

  const handleFetchReleases = async () => {
    setIsLoadingReleases(true);
    try {
      const rels: LlamaBuildRelease[] = await invoke("fetch_llama_releases");
      setReleases(rels);
    } catch (err) {
      alert(`Failed fetching GitHub releases: ${err}`);
    } finally {
      setIsLoadingReleases(false);
    }
  };

  const handleDownloadEngine = async (downloadUrl: string) => {
    if (isDownloadingEngine) return;
    setIsDownloadingEngine(true);

    try {
      await invoke("download_llama_engine", { downloadUrl });
      await loadEngineAndHardware();
      alert("llama-server engine downloaded and installed successfully!");
    } catch (err) {
      alert(`Download failed: ${err}`);
    } finally {
      setIsDownloadingEngine(false);
    }
  };

  const handleStopSidecar = async () => {
    try {
      await invoke("stop_sidecar");
      onRefreshStatus();
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="flex-1 h-full flex flex-col bg-neutral-100 dark:bg-neutral-900 text-neutral-800 dark:text-neutral-200 overflow-y-auto p-6 select-none transition-colors">
      <div className="max-w-5xl mx-auto w-full space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-lg font-semibold text-neutral-900 dark:text-white tracking-tight">Engine & Sidecar Manager</h1>
          <p className="text-xs text-neutral-500 dark:text-neutral-400">
            Supervise local llama-server inference process, GPU acceleration, and hardware telemetry.
          </p>
        </div>

        {/* Status Card & Process Controls */}
        <div className="p-5 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className={`w-10 h-10 rounded-xl flex items-center justify-center ${
                  status?.is_running
                    ? "bg-emerald-100 dark:bg-emerald-950/80 text-emerald-600 dark:text-emerald-400 border border-emerald-300 dark:border-emerald-800"
                    : "bg-neutral-100 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400 border border-neutral-200 dark:border-neutral-700"
                }`}
              >
                <FiActivity className="w-5 h-5" />
              </div>

              <div>
                <div className="flex items-center gap-2">
                  <span className="font-semibold text-sm text-neutral-900 dark:text-neutral-100">
                    Sidecar Runtime Status:
                  </span>
                  <span
                    className={`font-semibold text-xs ${
                      status?.is_running ? "text-emerald-600 dark:text-emerald-400" : "text-neutral-500 dark:text-neutral-400"
                    }`}
                  >
                    {status?.is_running ? "ONLINE (Active)" : "OFFLINE (Idle)"}
                  </span>
                </div>
                <div className="text-[11px] text-neutral-500 dark:text-neutral-400 font-mono flex items-center gap-2">
                  <span>Listening on {status?.api_url || "http://127.0.0.1:11434"}</span>
                  <span>•</span>
                  <span>{installedModelsCount} models installed</span>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-2">
              {status?.is_running ? (
                <button
                  onClick={handleStopSidecar}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-red-50 dark:bg-red-900/30 hover:bg-red-100 dark:hover:bg-red-900/50 text-red-600 dark:text-red-300 border border-red-200 dark:border-red-800 text-xs font-medium transition-colors cursor-pointer"
                >
                  <FiSquare className="w-3.5 h-3.5 fill-current" />
                  <span>Stop Engine</span>
                </button>
              ) : (
                <div className="text-xs text-neutral-400 dark:text-neutral-500 italic">
                  Engine starts automatically when a prompt is sent.
                </div>
              )}
            </div>
          </div>

          {/* Active sidecar config metrics */}
          {status?.is_running && (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3 pt-3 border-t border-neutral-200 dark:border-neutral-800 text-xs font-mono">
              <div className="p-2.5 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800">
                <div className="text-[10px] text-neutral-400 dark:text-neutral-500 uppercase">Current Model</div>
                <div className="text-neutral-800 dark:text-neutral-200 font-medium truncate">
                  {status.current_model?.replace(".gguf", "")}
                </div>
              </div>
              <div className="p-2.5 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800">
                <div className="text-[10px] text-neutral-400 dark:text-neutral-500 uppercase">GPU Offload</div>
                <div className="text-neutral-800 dark:text-neutral-200 font-medium">{status.n_gpu_layers} layers</div>
              </div>
              <div className="p-2.5 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800">
                <div className="text-[10px] text-neutral-400 dark:text-neutral-500 uppercase">Context Size</div>
                <div className="text-neutral-800 dark:text-neutral-200 font-medium">{status.context_size} tokens</div>
              </div>
              <div className="p-2.5 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800">
                <div className="text-[10px] text-neutral-400 dark:text-neutral-500 uppercase">CPU Threads</div>
                <div className="text-neutral-800 dark:text-neutral-200 font-medium">{status.threads} threads</div>
              </div>
            </div>
          )}
        </div>

        {/* Hardware & Hardware Advisor */}
        {hardware && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Hardware Telemetry */}
            <div className="p-5 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm space-y-3">
              <div className="flex items-center gap-2 text-xs font-semibold text-neutral-700 dark:text-neutral-300 uppercase tracking-wider">
                <FiCpu className="w-4 h-4 text-neutral-500 dark:text-neutral-400" />
                <span>Detected Host Hardware</span>
              </div>

              <div className="space-y-2 text-xs">
                <div className="flex justify-between py-1 border-b border-neutral-100 dark:border-neutral-800/60">
                  <span className="text-neutral-500 dark:text-neutral-400">Processor:</span>
                  <span className="text-neutral-800 dark:text-neutral-200 font-medium text-right max-w-[240px] truncate">
                    {hardware.cpu_name} ({hardware.cpu_cores} threads)
                  </span>
                </div>

                <div className="flex justify-between py-1 border-b border-neutral-100 dark:border-neutral-800/60">
                  <span className="text-neutral-500 dark:text-neutral-400">System RAM:</span>
                  <span className="text-neutral-800 dark:text-neutral-200 font-mono">
                    {hardware.free_ram_gb} GB free / {hardware.total_ram_gb} GB total
                  </span>
                </div>

                <div className="flex justify-between py-1 border-b border-neutral-100 dark:border-neutral-800/60">
                  <span className="text-neutral-500 dark:text-neutral-400">Discrete GPU:</span>
                  <span className="text-neutral-800 dark:text-neutral-200 font-medium text-right max-w-[240px] truncate">
                    {hardware.gpu_name}
                  </span>
                </div>

                <div className="flex justify-between py-1">
                  <span className="text-neutral-500 dark:text-neutral-400">Estimated VRAM:</span>
                  <span className="text-neutral-800 dark:text-neutral-200 font-mono">~{hardware.estimated_vram_gb} GB</span>
                </div>
              </div>
            </div>

            {/* Smart Hardware Recommendation */}
            {recommendation && (
              <div className="p-5 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm space-y-3">
                <div className="flex items-center gap-2 text-xs font-semibold text-emerald-600 dark:text-emerald-400 uppercase tracking-wider">
                  <FiCheckCircle className="w-4 h-4" />
                  <span>Hardware Tuned Recommendation</span>
                </div>

                <div className="space-y-2 text-xs">
                  <div className="flex justify-between py-1 border-b border-neutral-100 dark:border-neutral-800/60">
                    <span className="text-neutral-500 dark:text-neutral-400">Optimal Model Size:</span>
                    <span className="text-neutral-900 dark:text-neutral-100 font-semibold">{recommendation.max_params}</span>
                  </div>

                  <div className="flex justify-between py-1 border-b border-neutral-100 dark:border-neutral-800/60">
                    <span className="text-neutral-500 dark:text-neutral-400">Quantization Preset:</span>
                    <span className="text-neutral-800 dark:text-neutral-200 font-mono">{recommendation.recommended_quant}</span>
                  </div>

                  <div className="flex justify-between py-1 border-b border-neutral-100 dark:border-neutral-800/60">
                    <span className="text-neutral-500 dark:text-neutral-400">Recommended Backend:</span>
                    <span className="text-neutral-800 dark:text-neutral-200 font-medium">{recommendation.recommended_backend}</span>
                  </div>

                  <div className="pt-1 text-[11px] text-neutral-500 dark:text-neutral-400 leading-relaxed italic">
                    "{recommendation.reason}"
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Engine Binary Installation & Releases */}
        <div className="p-5 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="font-semibold text-sm text-neutral-900 dark:text-neutral-100">llama.cpp Engine Binary</h2>
              <p className="text-xs text-neutral-500 dark:text-neutral-400">
                {engineInfo?.is_installed
                  ? `Installed at: ${engineInfo.exe_path}`
                  : "Engine binary not found. Download an official build below to enable inference."}
              </p>
            </div>

            <button
              onClick={handleFetchReleases}
              disabled={isLoadingReleases}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-750 text-neutral-800 dark:text-neutral-200 text-xs font-medium border border-neutral-300 dark:border-neutral-700 transition-colors cursor-pointer"
            >
              <FiRotateCw className={`w-3.5 h-3.5 ${isLoadingReleases ? "animate-spin" : ""}`} />
              <span>Check GitHub Releases</span>
            </button>
          </div>

          {/* If releases loaded, show release cards */}
          {releases.length > 0 ? (
            <div className="space-y-3 pt-2">
              <div className="flex gap-2 text-xs mb-2">
                <button
                  onClick={() => setSelectedBuildType("vulkan")}
                  className={`px-3 py-1 rounded-md transition-colors cursor-pointer ${
                    selectedBuildType === "vulkan"
                      ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium"
                      : "text-neutral-500 dark:text-neutral-400 hover:text-neutral-800 dark:hover:text-neutral-200"
                  }`}
                >
                  {releases[0]?.recommended_label || "GPU / Recommended"}
                </button>
                <button
                  onClick={() => setSelectedBuildType("cpu")}
                  className={`px-3 py-1 rounded-md transition-colors cursor-pointer ${
                    selectedBuildType === "cpu"
                      ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium"
                      : "text-neutral-500 dark:text-neutral-400 hover:text-neutral-800 dark:hover:text-neutral-200"
                  }`}
                >
                  {releases[0]?.fallback_label || "CPU (Fallback)"}
                </button>
              </div>

              {releases.slice(0, 3).map((rel) => {
                const targetUrl =
                  selectedBuildType === "vulkan"
                    ? rel.recommended_url || rel.vulkan_win_url
                    : rel.fallback_url || rel.cpu_win_url;
                return (
                  <div
                    key={rel.tag_name}
                    className="p-3.5 rounded-xl bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 flex items-center justify-between text-xs"
                  >
                    <div>
                      <div className="font-semibold text-neutral-900 dark:text-neutral-100">{rel.release_name}</div>
                      <div className="text-[11px] text-neutral-500 font-mono">
                        Published: {rel.published_at.slice(0, 10)} • Tag: {rel.tag_name}
                      </div>
                    </div>

                    <button
                      onClick={() => handleDownloadEngine(targetUrl)}
                      disabled={isDownloadingEngine}
                      className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-800 dark:text-neutral-200 font-medium transition-colors border border-neutral-300 dark:border-neutral-700 disabled:opacity-50 cursor-pointer"
                    >
                      <FiDownload className="w-3.5 h-3.5" />
                      <span>Install Engine</span>
                    </button>
                  </div>
                );
              })}
            </div>
          ) : !engineInfo?.is_installed ? (
            <div className="p-4 rounded-xl bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 flex items-center justify-between text-xs">
              <div className="flex items-center gap-2 text-neutral-700 dark:text-neutral-300">
                <FiAlertCircle className="w-4 h-4 text-amber-500" />
                <span>Quick Setup: Download latest stable llama.cpp engine binary</span>
              </div>
              <button
                onClick={() => {
                  const ua = typeof navigator !== "undefined" ? navigator.userAgent.toLowerCase() : "";
                  const url = ua.includes("mac")
                    ? "https://github.com/ggml-org/llama.cpp/releases/download/b4800/llama-b4800-bin-macos-arm64.zip"
                    : ua.includes("linux")
                    ? "https://github.com/ggml-org/llama.cpp/releases/download/b4800/llama-b4800-bin-ubuntu-x64.zip"
                    : "https://github.com/ggml-org/llama.cpp/releases/download/b4800/llama-b4800-bin-win-vulkan-x64.zip";
                  handleDownloadEngine(url);
                }}
                disabled={isDownloadingEngine}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-neutral-900 text-white dark:bg-white dark:text-black font-semibold hover:bg-neutral-800 dark:hover:bg-neutral-200 transition-colors cursor-pointer"
              >
                <FiDownload className="w-3.5 h-3.5" />
                <span>1-Click Download Engine</span>
              </button>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
};
