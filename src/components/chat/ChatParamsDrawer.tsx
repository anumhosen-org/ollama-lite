import React from "react";
import { FiX, FiSliders } from "react-icons/fi";

interface ChatParamsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  temperature: number;
  setTemperature: (v: number) => void;
  contextSize: number;
  setContextSize: (v: number) => void;
  gpuLayers: number;
  setGpuLayers: (v: number) => void;
  cpuThreads: number;
  setCpuThreads: (v: number) => void;
  systemPrompt: string;
  setSystemPrompt: (v: string) => void;
}

export const SYSTEM_PRESETS = [
  {
    name: "Helpful Assistant",
    prompt: "You are a helpful, respectful, and honest assistant. Answer concisely and accurately.",
  },
  {
    name: "Senior Developer",
    prompt: "You are a senior software architect. Provide clean, modular, and idiomatic code with brief explanations.",
  },
  {
    name: "Deep Reasoning",
    prompt: "Think step by step with careful analytical reasoning before giving your final conclusion.",
  },
  {
    name: "Concise Writer",
    prompt: "Provide clear, direct, and concise answers without unnecessary preamble or filler words.",
  },
];

export const ChatParamsDrawer: React.FC<ChatParamsDrawerProps> = ({
  isOpen,
  onClose,
  temperature,
  setTemperature,
  contextSize,
  setContextSize,
  gpuLayers,
  setGpuLayers,
  cpuThreads,
  setCpuThreads,
  systemPrompt,
  setSystemPrompt,
}) => {
  if (!isOpen) return null;

  return (
    <div className="absolute right-0 top-0 bottom-0 w-80 bg-white dark:bg-neutral-900 border-l border-neutral-200 dark:border-neutral-800 shadow-2xl z-30 flex flex-col select-none text-xs transition-colors">
      <div className="flex items-center justify-between p-3.5 border-b border-neutral-200 dark:border-neutral-800">
        <div className="flex items-center gap-2 font-medium text-neutral-900 dark:text-neutral-200">
          <FiSliders className="w-4 h-4 text-neutral-500 dark:text-neutral-400" />
          <span>Model Parameters</span>
        </div>
        <button
          onClick={onClose}
          className="p-1 rounded text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
        >
          <FiX className="w-4 h-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-5 text-neutral-800 dark:text-neutral-300">
        {/* System Preset */}
        <div>
          <label className="block text-[11px] font-semibold text-neutral-500 dark:text-neutral-400 uppercase tracking-wider mb-2">
            Persona Preset
          </label>
          <div className="grid grid-cols-2 gap-1.5 mb-2">
            {SYSTEM_PRESETS.map((preset) => (
              <button
                key={preset.name}
                onClick={() => setSystemPrompt(preset.prompt)}
                className={`px-2 py-1.5 rounded text-left truncate text-xs transition-colors border cursor-pointer ${
                  systemPrompt === preset.prompt
                    ? "bg-neutral-200 dark:bg-neutral-800 border-neutral-400 dark:border-neutral-600 text-neutral-900 dark:text-white font-medium"
                    : "bg-neutral-50 dark:bg-neutral-900/60 border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800"
                }`}
              >
                {preset.name}
              </button>
            ))}
          </div>
          <textarea
            value={systemPrompt}
            onChange={(e) => setSystemPrompt(e.target.value)}
            rows={3}
            className="w-full px-2.5 py-1.5 rounded-md bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-neutral-200 text-xs focus:outline-none focus:border-neutral-500 dark:focus:border-neutral-600 resize-none font-mono"
            placeholder="Custom system instructions..."
          />
        </div>

        {/* Temperature */}
        <div>
          <div className="flex justify-between mb-1.5">
            <label className="text-xs font-medium text-neutral-800 dark:text-neutral-200">Temperature</label>
            <span className="font-mono text-neutral-500 dark:text-neutral-400">{temperature.toFixed(2)}</span>
          </div>
          <input
            type="range"
            min="0"
            max="1.5"
            step="0.05"
            value={temperature}
            onChange={(e) => setTemperature(parseFloat(e.target.value))}
            className="w-full accent-neutral-800 dark:accent-neutral-200"
          />
          <div className="flex justify-between text-[10px] text-neutral-400 dark:text-neutral-500 mt-1">
            <span>Precise (0.0)</span>
            <span>Creative (1.5)</span>
          </div>
        </div>

        {/* Context Window */}
        <div>
          <div className="flex justify-between mb-1.5">
            <label className="text-xs font-medium text-neutral-800 dark:text-neutral-200">Context Size</label>
            <span className="font-mono text-neutral-500 dark:text-neutral-400">{contextSize} tokens</span>
          </div>
          <input
            type="range"
            min="2048"
            max="32768"
            step="1024"
            value={contextSize}
            onChange={(e) => setContextSize(parseInt(e.target.value))}
            className="w-full accent-neutral-800 dark:accent-neutral-200"
          />
          <div className="flex justify-between text-[10px] text-neutral-400 dark:text-neutral-500 mt-1">
            <span>2K</span>
            <span>8K</span>
            <span>16K</span>
            <span>32K</span>
          </div>
        </div>

        {/* GPU Offload Layers */}
        <div>
          <div className="flex justify-between mb-1.5">
            <label className="text-xs font-medium text-neutral-800 dark:text-neutral-200">GPU Offload Layers (-ngl)</label>
            <span className="font-mono text-neutral-500 dark:text-neutral-400">{gpuLayers} layers</span>
          </div>
          <input
            type="range"
            min="0"
            max="128"
            step="1"
            value={gpuLayers}
            onChange={(e) => setGpuLayers(parseInt(e.target.value))}
            className="w-full accent-neutral-800 dark:accent-neutral-200"
          />
          <div className="flex justify-between text-[10px] text-neutral-400 dark:text-neutral-500 mt-1">
            <span>CPU Only (0)</span>
            <span>Max GPU (99+)</span>
          </div>
        </div>

        {/* CPU Threads */}
        <div>
          <div className="flex justify-between mb-1.5">
            <label className="text-xs font-medium text-neutral-800 dark:text-neutral-200">CPU Threads (-t)</label>
            <span className="font-mono text-neutral-500 dark:text-neutral-400">{cpuThreads}</span>
          </div>
          <input
            type="range"
            min="1"
            max="32"
            step="1"
            value={cpuThreads}
            onChange={(e) => setCpuThreads(parseInt(e.target.value))}
            className="w-full accent-neutral-800 dark:accent-neutral-200"
          />
        </div>
      </div>
    </div>
  );
};
