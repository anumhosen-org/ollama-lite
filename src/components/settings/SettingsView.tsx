import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  FiSettings,
  FiTerminal,
  FiToggleLeft,
  FiToggleRight,
  FiServer,
  FiCheck,
} from "react-icons/fi";
import { OllamaLogo } from "../common/OllamaLogo";

export const SettingsView: React.FC = () => {
  const [port, setPort] = useState("11434");
  const [autostart, setAutostart] = useState(false);
  const [cliInstalled, setCliInstalled] = useState(false);
  const [localIp, setLocalIp] = useState("127.0.0.1");
  const [savedSuccess, setSavedSuccess] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const isAuto: boolean = await invoke("is_autostart_enabled");
      setAutostart(isAuto);

      const isCli: boolean = await invoke("check_cli_installed");
      setCliInstalled(isCli);

      const ip: string = await invoke("get_local_ip");
      setLocalIp(ip);

      const savedPort: string | null = await invoke("get_db_setting", { key: "server_port" });
      if (savedPort) setPort(savedPort);
    } catch (err) {
      console.error(err);
    }
  };

  const handleToggleAutostart = async () => {
    try {
      const next = !autostart;
      await invoke("set_autostart_enabled", { enabled: next });
      setAutostart(next);
    } catch (err) {
      alert(`Autostart error: ${err}`);
    }
  };

  const handleInstallCli = async () => {
    try {
      await invoke("install_cli_to_path");
      setCliInstalled(true);
      alert("ollama-lite command added to your user PATH. Open a new Command Prompt or PowerShell to use 'ollama-lite list' or 'ollama-lite run'.");
    } catch (err) {
      alert(`CLI installation failed: ${err}`);
    }
  };

  const handleSavePort = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await invoke("set_db_setting", { key: "server_port", value: port });
      setSavedSuccess(true);
      setTimeout(() => setSavedSuccess(false), 2000);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="flex-1 h-full flex flex-col bg-neutral-100 dark:bg-neutral-900 text-neutral-800 dark:text-neutral-200 overflow-y-auto p-6 select-none transition-colors">
      <div className="max-w-3xl mx-auto w-full space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-lg font-semibold text-neutral-900 dark:text-white tracking-tight">Application Settings</h1>
          <p className="text-xs text-neutral-500 dark:text-neutral-400">
            Configure local network ports, CLI integration, and system startup preferences.
          </p>
        </div>

        {/* Server & Network Settings */}
        <div className="p-5 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm space-y-4">
          <div className="flex items-center gap-2 text-xs font-semibold text-neutral-700 dark:text-neutral-200 uppercase tracking-wider">
            <FiServer className="w-4 h-4 text-neutral-500 dark:text-neutral-400" />
            <span>Ollama API & Network Server</span>
          </div>

          <form onSubmit={handleSavePort} className="flex items-center gap-3 text-xs">
            <div className="flex-1">
              <label className="block text-neutral-600 dark:text-neutral-400 mb-1">Server Listen Port</label>
              <input
                type="text"
                value={port}
                onChange={(e) => setPort(e.target.value)}
                placeholder="11434"
                className="w-full px-3 py-1.5 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-neutral-200 font-mono text-xs focus:outline-none focus:border-neutral-400 dark:focus:border-neutral-600"
              />
            </div>

            <button
              type="submit"
              className="mt-5 px-4 py-1.5 rounded-lg bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-800 dark:text-neutral-200 font-medium border border-neutral-300 dark:border-neutral-700 transition-colors cursor-pointer"
            >
              {savedSuccess ? "Saved!" : "Save"}
            </button>
          </form>

          <div className="pt-2 border-t border-neutral-200 dark:border-neutral-800/80 text-[11px] text-neutral-500 dark:text-neutral-400 space-y-1">
            <div className="flex justify-between">
              <span>Localhost Endpoint:</span>
              <span className="font-mono text-neutral-800 dark:text-neutral-200">http://127.0.0.1:{port}</span>
            </div>
            <div className="flex justify-between">
              <span>LAN Network Address:</span>
              <span className="font-mono text-neutral-800 dark:text-neutral-200">http://{localIp}:{port}</span>
            </div>
          </div>
        </div>

        {/* REST API & Third-Party Integration Endpoints */}
        <div className="p-5 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-neutral-700 dark:text-neutral-200 uppercase tracking-wider">
              Ollama & OpenAI Compatible Endpoints
            </span>
            <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-emerald-100 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-400 border border-emerald-300 dark:border-emerald-800">
              Active on Port {port}
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs font-mono">
            <div className="p-2 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800/60">
              <span className="text-[10px] text-emerald-600 dark:text-emerald-400 uppercase">Ollama Chat API</span>
              <div className="text-neutral-800 dark:text-neutral-200 text-[11px]">POST /api/chat</div>
            </div>
            <div className="p-2 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800/60">
              <span className="text-[10px] text-emerald-600 dark:text-emerald-400 uppercase">Ollama Generate API</span>
              <div className="text-neutral-800 dark:text-neutral-200 text-[11px]">POST /api/generate</div>
            </div>
            <div className="p-2 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800/60">
              <span className="text-[10px] text-blue-600 dark:text-blue-400 uppercase">Model Tags / List</span>
              <div className="text-neutral-800 dark:text-neutral-200 text-[11px]">GET /api/tags</div>
            </div>
            <div className="p-2 rounded-lg bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800/60">
              <span className="text-[10px] text-purple-600 dark:text-purple-400 uppercase">OpenAI Chat API</span>
              <div className="text-neutral-800 dark:text-neutral-200 text-[11px]">POST /v1/chat/completions</div>
            </div>
          </div>
          <p className="text-[11px] text-neutral-500 dark:text-neutral-400">
            Fully compatible with Cursor, Continue.dev, Open WebUI, LangChain, LiteLLM, and the official Ollama Python/JS SDKs.
          </p>
        </div>

        {/* Windows Desktop Features */}
        <div className="p-5 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm space-y-4">
          <div className="flex items-center gap-2 text-xs font-semibold text-neutral-700 dark:text-neutral-200 uppercase tracking-wider">
            <FiSettings className="w-4 h-4 text-neutral-500 dark:text-neutral-400" />
            <span>Desktop & System Integration</span>
          </div>

          <div className="divide-y divide-neutral-200 dark:divide-neutral-800/60 text-xs">
            {/* Windows Autostart */}
            <div className="py-3 flex items-center justify-between">
              <div>
                <div className="font-medium text-neutral-800 dark:text-neutral-200">Launch on System Startup</div>
                <div className="text-[11px] text-neutral-500 dark:text-neutral-400">
                  Automatically run Ollama Lite in the background when logging into your desktop session.
                </div>
              </div>
              <button
                onClick={handleToggleAutostart}
                className="p-1 text-2xl text-neutral-400 hover:text-neutral-700 dark:text-neutral-500 dark:hover:text-neutral-300 transition-colors cursor-pointer"
              >
                {autostart ? (
                  <FiToggleRight className="text-emerald-500" />
                ) : (
                  <FiToggleLeft className="text-neutral-400 dark:text-neutral-500" />
                )}
              </button>
            </div>

            {/* CLI Installer */}
            <div className="py-3 flex items-center justify-between">
              <div>
                <div className="font-medium text-neutral-800 dark:text-neutral-200">Drop-In Terminal CLI ('ollama')</div>
                <div className="text-[11px] text-neutral-500 dark:text-neutral-400">
                  Drop-in replacement for the official Ollama CLI. Supports <code className="font-mono text-[10px]">ollama run &lt;model&gt;</code>, <code className="font-mono text-[10px]">ollama pull</code>, <code className="font-mono text-[10px]">ollama list</code>, and <code className="font-mono text-[10px]">ollama ps</code>.
                </div>
              </div>

              {cliInstalled ? (
                <div className="flex items-center gap-1 text-emerald-600 dark:text-emerald-400 text-xs font-medium">
                  <FiCheck className="w-4 h-4" />
                  <span>Installed ('ollama', 'ollama-lite')</span>
                </div>
              ) : (
                <button
                  onClick={handleInstallCli}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-750 text-neutral-800 dark:text-neutral-200 border border-neutral-300 dark:border-neutral-700 text-xs transition-colors cursor-pointer"
                >
                  <FiTerminal className="w-3.5 h-3.5" />
                  <span>Install 'ollama' to PATH</span>
                </button>
              )}
            </div>
          </div>
        </div>

        {/* About Card */}
        <div className="p-5 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-sm flex items-center gap-4 text-xs">
          <OllamaLogo className="w-10 h-10 text-neutral-800 dark:text-neutral-200 shrink-0" />
          <div className="space-y-1">
            <div className="font-semibold text-sm text-neutral-900 dark:text-neutral-100 flex items-center gap-2">
              <span>Ollama Lite</span>
              <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 border border-neutral-200 dark:border-neutral-700">
                v0.1.0
              </span>
            </div>
            <p className="text-neutral-600 dark:text-neutral-400 text-[11px] leading-relaxed">
              Synthesized from Ollama's signature UI & REST API semantics with llama.cpp's embedded
              inference sidecar and Metal, Vulkan, or CUDA GPU hardware acceleration.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
