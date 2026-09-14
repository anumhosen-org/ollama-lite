import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import {
  FiDownload,
  FiFolder,
  FiCheck,
  FiTrash2,
  FiSearch,
  FiLink,
  FiHardDrive,
  FiCpu,
} from "react-icons/fi";
import {
  CuratedModel,
  DownloadProgressPayload,
  ModelFileInfo,
} from "../../types";
import { CURATED_MODELS } from "../../data/curatedModels";

interface ModelsViewProps {
  installedModels: ModelFileInfo[];
  downloadProgress: DownloadProgressPayload | null;
  onRefreshModels: () => void;
  activeModelPath?: string;
  onSelectModel: (path: string) => void;
}

export const ModelsView: React.FC<ModelsViewProps> = ({
  installedModels,
  downloadProgress,
  onRefreshModels,
  activeModelPath,
  onSelectModel,
}) => {
  const [activeTab, setActiveTab] = useState<"curated" | "installed" | "custom">("curated");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedFamily, setSelectedFamily] = useState("All");
  const [customUrl, setCustomUrl] = useState("");
  const [customFilename, setCustomFilename] = useState("");
  const [isDownloading, setIsDownloading] = useState(false);

  const filteredCurated = CURATED_MODELS.filter((m) => {
    const matchesSearch =
      m.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      m.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      m.family.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesFamily =
      selectedFamily === "All" || m.family.toLowerCase() === selectedFamily.toLowerCase();
    return matchesSearch && matchesFamily;
  });

  const isModelInstalled = (filename: string) => {
    return installedModels.some(
      (m) => m.filename.toLowerCase() === filename.toLowerCase()
    );
  };

  const handleDownloadCurated = async (model: CuratedModel) => {
    if (isDownloading) return;
    setIsDownloading(true);

    try {
      await invoke("download_model_file", {
        downloadUrl: model.download_url,
        filename: model.filename,
      });
      onRefreshModels();
    } catch (err) {
      alert(`Download error: ${err}`);
    } finally {
      setIsDownloading(false);
    }
  };

  const handleDownloadCustom = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!customUrl.trim() || !customFilename.trim() || isDownloading) return;

    let targetFilename = customFilename.trim();
    if (!targetFilename.toLowerCase().endsWith(".gguf")) {
      targetFilename += ".gguf";
    }

    setIsDownloading(true);
    try {
      await invoke("download_model_file", {
        downloadUrl: customUrl.trim(),
        filename: targetFilename,
      });
      setCustomUrl("");
      setCustomFilename("");
      onRefreshModels();
      setActiveTab("installed");
    } catch (err) {
      alert(`Download error: ${err}`);
    } finally {
      setIsDownloading(false);
    }
  };

  const handleImportLocalGguf = async () => {
    try {
      const selected = await openFileDialog({
        multiple: false,
        filters: [{ name: "GGUF Model Files", extensions: ["gguf"] }],
      });

      if (selected && typeof selected === "string") {
        await invoke("inspect_gguf_file", { filepath: selected });
        onRefreshModels();
        setActiveTab("installed");
      }
    } catch (err) {
      console.error("Failed importing local GGUF:", err);
    }
  };

  const handleDeleteModel = async (path: string) => {
    if (confirm("Are you sure you want to remove this model from disk?")) {
      try {
        await invoke("delete_installed_model", { filepath: path });
        onRefreshModels();
      } catch (err) {
        alert(`Delete failed: ${err}`);
      }
    }
  };

  return (
    <div className="flex-1 h-full flex flex-col bg-neutral-100 dark:bg-neutral-900 text-neutral-800 dark:text-neutral-200 overflow-hidden select-none transition-colors">
      {/* Header */}
      <div className="p-6 border-b border-neutral-200 dark:border-neutral-800 bg-white/90 dark:bg-neutral-900/90 backdrop-blur-sm shrink-0">
        <div className="max-w-5xl mx-auto">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h1 className="text-lg font-semibold text-neutral-900 dark:text-white tracking-tight">Models Hub</h1>
              <p className="text-xs text-neutral-500 dark:text-neutral-400">
                Explore curated models, download GGUFs from Hugging Face, or import local files.
              </p>
            </div>

            <button
              onClick={handleImportLocalGguf}
              className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-750 text-neutral-800 dark:text-neutral-200 border border-neutral-300 dark:border-neutral-700 text-xs font-medium transition-all shadow-xs"
            >
              <FiFolder className="w-3.5 h-3.5" />
              <span>Import .gguf</span>
            </button>
          </div>

          {/* Sub Navigation Tabs */}
          <div className="flex gap-2 text-xs">
            <button
              onClick={() => setActiveTab("curated")}
              className={`px-3 py-1.5 rounded-md transition-colors ${
                activeTab === "curated"
                  ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium"
                  : "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200"
              }`}
            >
              Curated Models ({CURATED_MODELS.length})
            </button>
            <button
              onClick={() => setActiveTab("installed")}
              className={`px-3 py-1.5 rounded-md transition-colors ${
                activeTab === "installed"
                  ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium"
                  : "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200"
              }`}
            >
              Installed ({installedModels.length})
            </button>
            <button
              onClick={() => setActiveTab("custom")}
              className={`px-3 py-1.5 rounded-md transition-colors ${
                activeTab === "custom"
                  ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium"
                  : "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200"
              }`}
            >
              HuggingFace Downloader
            </button>
          </div>
        </div>
      </div>

      {/* Global Download Progress Banner */}
      {downloadProgress && !downloadProgress.is_complete && (
        <div className="bg-neutral-50 dark:bg-neutral-950 border-b border-neutral-200 dark:border-neutral-800 px-6 py-2.5 shrink-0">
          <div className="max-w-5xl mx-auto flex items-center justify-between text-xs mb-1.5">
            <div className="flex items-center gap-2 text-neutral-800 dark:text-neutral-200">
              <FiDownload className="w-3.5 h-3.5 text-emerald-500 dark:text-emerald-400 animate-bounce" />
              <span className="font-medium truncate max-w-sm">{downloadProgress.filename}</span>
            </div>
            <div className="flex items-center gap-3 font-mono text-neutral-500 dark:text-neutral-400 text-[11px]">
              <span>{downloadProgress.speed_mbps} MB/s</span>
              <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                {downloadProgress.progress_percent}%
              </span>
            </div>
          </div>
          <div className="max-w-5xl mx-auto w-full bg-neutral-200 dark:bg-neutral-800 h-1.5 rounded-full overflow-hidden">
            <div
              className="bg-emerald-500 h-full transition-all duration-300 rounded-full"
              style={{ width: `${downloadProgress.progress_percent}%` }}
            />
          </div>
        </div>
      )}

      {/* Content Area */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-6xl mx-auto">
          {/* CURATED TAB */}
          {activeTab === "curated" && (
            <div className="space-y-4">
              <div className="flex flex-col sm:flex-row gap-3 items-stretch sm:items-center justify-between">
                <div className="relative flex-1">
                  <FiSearch className="w-4 h-4 text-neutral-400 dark:text-neutral-500 absolute left-3 top-2.5" />
                  <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Filter models by name, family, or purpose..."
                    className="w-full pl-9 pr-4 py-2 rounded-lg bg-white dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-neutral-200 text-xs focus:outline-none focus:border-neutral-400 dark:focus:border-neutral-700 placeholder-neutral-400 dark:placeholder-neutral-500"
                  />
                </div>

                {/* Family Category Pills */}
                <div className="flex items-center gap-1.5 overflow-x-auto pb-1 sm:pb-0 text-xs shrink-0">
                  {["All", "DeepSeek", "Llama", "Qwen", "Mistral", "Phi", "Gemma", "Hermes", "SmolLM"].map((fam) => (
                    <button
                      key={fam}
                      onClick={() => setSelectedFamily(fam)}
                      className={`px-2.5 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                        selectedFamily === fam
                          ? "bg-neutral-900 text-white dark:bg-white dark:text-black shadow-xs"
                          : "bg-neutral-200/60 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300"
                      }`}
                    >
                      {fam}
                    </button>
                  ))}
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {filteredCurated.map((model) => {
                  const installed = isModelInstalled(model.filename);
                  return (
                    <div
                      key={model.name}
                      className="p-4 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700/80 transition-all flex flex-col justify-between shadow-xs"
                    >
                      <div>
                        <div className="flex items-start justify-between gap-2 mb-1.5">
                          <h3 className="font-semibold text-sm text-neutral-900 dark:text-neutral-100">{model.name}</h3>
                          <span className="px-2 py-0.5 rounded text-[10px] font-mono bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 border border-neutral-200 dark:border-neutral-700">
                            {model.parameter_size}
                          </span>
                        </div>

                        <p className="text-xs text-neutral-600 dark:text-neutral-400 line-clamp-2 mb-3 leading-relaxed">
                          {model.description}
                        </p>

                        <div className="flex flex-wrap gap-1.5 mb-4 text-[10px] font-mono text-neutral-500 dark:text-neutral-400">
                          <span className="px-1.5 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800/80">
                            {model.quantization}
                          </span>
                          <span className="px-1.5 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800/80">
                            {model.file_size_gb} GB
                          </span>
                          <span className="px-1.5 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800/80">
                            VRAM: {model.recommended_vram}
                          </span>
                        </div>
                      </div>

                      {installed ? (
                        <div className="flex items-center justify-between pt-2 border-t border-neutral-200 dark:border-neutral-800 text-xs text-emerald-600 dark:text-emerald-400">
                          <div className="flex items-center gap-1 font-medium">
                            <FiCheck className="w-3.5 h-3.5" />
                            <span>Installed</span>
                          </div>
                          <button
                            onClick={() => {
                              const found = installedModels.find(
                                (m) => m.filename.toLowerCase() === model.filename.toLowerCase()
                              );
                              if (found) onSelectModel(found.full_path);
                            }}
                            className="text-[11px] text-neutral-600 hover:text-neutral-900 dark:text-neutral-300 dark:hover:text-white underline cursor-pointer"
                          >
                            Load in Chat
                          </button>
                        </div>
                      ) : (
                        <button
                          onClick={() => handleDownloadCurated(model)}
                          disabled={isDownloading}
                          className="w-full flex items-center justify-center gap-1.5 py-2 rounded-lg bg-neutral-900 hover:bg-neutral-800 text-white dark:bg-neutral-800 dark:hover:bg-neutral-700 dark:text-neutral-100 text-xs font-medium transition-colors border border-neutral-800 dark:border-neutral-700 cursor-pointer disabled:opacity-50"
                        >
                          <FiDownload className="w-3.5 h-3.5" />
                          <span>Download ({model.file_size_gb} GB)</span>
                        </button>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* INSTALLED TAB */}
          {activeTab === "installed" && (
            <div className="space-y-3">
              {installedModels.length === 0 ? (
                <div className="py-12 text-center text-neutral-400 dark:text-neutral-500">
                  <FiHardDrive className="w-8 h-8 mx-auto mb-2 opacity-50" />
                  <p className="text-xs">No models installed in your models directory.</p>
                  <button
                    onClick={() => setActiveTab("curated")}
                    className="mt-3 px-3 py-1.5 rounded-lg bg-neutral-200 dark:bg-neutral-800 text-xs text-neutral-800 dark:text-neutral-200 hover:bg-neutral-300 dark:hover:bg-neutral-700 font-medium"
                  >
                    Browse Curated Models
                  </button>
                </div>
              ) : (
                installedModels.map((model) => {
                  const isActive =
                    activeModelPath === model.full_path || activeModelPath === model.filename;
                  return (
                    <div
                      key={model.full_path}
                      className={`p-3.5 rounded-xl border flex items-center justify-between transition-all ${
                        isActive
                          ? "bg-neutral-200/70 dark:bg-neutral-800/80 border-neutral-300 dark:border-neutral-600"
                          : "bg-white dark:bg-neutral-900 border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700"
                      }`}
                    >
                      <div className="flex items-center gap-3 truncate pr-4">
                        <div className="w-8 h-8 rounded-lg bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center shrink-0 border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300">
                          <FiCpu className="w-4 h-4" />
                        </div>
                        <div className="truncate">
                          <div className="flex items-center gap-2">
                            <span className="font-semibold text-xs text-neutral-900 dark:text-neutral-100 truncate">
                              {model.filename.replace(".gguf", "")}
                            </span>
                            {isActive && (
                              <span className="px-1.5 py-0.2 rounded text-[10px] bg-emerald-100 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-400 border border-emerald-300 dark:border-emerald-800 font-medium">
                                Active
                              </span>
                            )}
                          </div>
                          <span className="text-[11px] text-neutral-500 dark:text-neutral-400 font-mono">
                            {model.size_gb} GB • {model.full_path}
                          </span>
                        </div>
                      </div>

                      <div className="flex items-center gap-2 shrink-0">
                        <button
                          onClick={() => onSelectModel(model.full_path)}
                          className="px-3 py-1.5 rounded-lg bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-800 dark:text-neutral-200 text-xs font-medium transition-colors border border-neutral-300 dark:border-neutral-700 cursor-pointer"
                        >
                          {isActive ? "Reload" : "Launch"}
                        </button>
                        <button
                          onClick={() => handleDeleteModel(model.full_path)}
                          className="p-1.5 rounded-lg text-neutral-400 hover:text-red-500 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                          title="Delete file"
                        >
                          <FiTrash2 className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          )}

          {/* CUSTOM HUGGINGFACE DOWNLOADER TAB */}
          {activeTab === "custom" && (
            <div className="max-w-2xl mx-auto p-6 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm">
              <div className="flex items-center gap-2 mb-4 text-neutral-900 dark:text-neutral-100 font-semibold text-sm">
                <FiLink className="w-4 h-4 text-neutral-500 dark:text-neutral-400" />
                <span>Download GGUF from Hugging Face or Direct URL</span>
              </div>

              <form onSubmit={handleDownloadCustom} className="space-y-4 text-xs">
                <div>
                  <label className="block text-neutral-600 dark:text-neutral-400 mb-1 font-medium">Download URL (.gguf)</label>
                  <input
                    type="url"
                    required
                    value={customUrl}
                    onChange={(e) => {
                      setCustomUrl(e.target.value);
                      if (!customFilename) {
                        const extracted = e.target.value.split("/").pop() || "";
                        if (extracted.endsWith(".gguf")) {
                          setCustomFilename(extracted);
                        }
                      }
                    }}
                    placeholder="https://huggingface.co/.../resolve/main/model-q4_k_m.gguf"
                    className="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-neutral-200 focus:outline-none focus:border-neutral-400 dark:focus:border-neutral-600 font-mono text-[11px]"
                  />
                </div>

                <div>
                  <label className="block text-neutral-600 dark:text-neutral-400 mb-1 font-medium">Local Filename</label>
                  <input
                    type="text"
                    required
                    value={customFilename}
                    onChange={(e) => setCustomFilename(e.target.value)}
                    placeholder="model-name-q4_k_m.gguf"
                    className="w-full px-3 py-2 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-neutral-200 focus:outline-none focus:border-neutral-400 dark:focus:border-neutral-600 font-mono text-[11px]"
                  />
                </div>

                <button
                  type="submit"
                  disabled={isDownloading || !customUrl.trim() || !customFilename.trim()}
                  className="w-full flex items-center justify-center gap-2 py-2.5 rounded-lg bg-neutral-900 text-white dark:bg-white dark:text-black font-semibold text-xs hover:bg-neutral-800 dark:hover:bg-neutral-200 transition-colors cursor-pointer disabled:opacity-50"
                >
                  <FiDownload className="w-4 h-4" />
                  <span>Start Model Download</span>
                </button>
              </form>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
