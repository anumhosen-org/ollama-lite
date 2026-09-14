import React, { useState, useRef, useEffect } from "react";
import { FiChevronDown, FiPlus, FiCheck } from "react-icons/fi";
import { ModelFileInfo, SidecarStatus } from "../../types";

interface ModelSelectorProps {
  installedModels: ModelFileInfo[];
  selectedModel: string;
  onSelectModel: (modelPath: string) => void;
  status: SidecarStatus | null;
  onOpenModelsLibrary: () => void;
}

export const ModelSelector: React.FC<ModelSelectorProps> = ({
  installedModels,
  selectedModel,
  onSelectModel,
  status,
  onOpenModelsLibrary,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const currentModelFile = installedModels.find(
    (m) => m.full_path === selectedModel || m.filename === selectedModel
  );

  const displayName = currentModelFile
    ? currentModelFile.filename.replace(".gguf", "")
    : status?.current_model
    ? status.current_model.replace(".gguf", "")
    : installedModels.length > 0
    ? "Select a model"
    : "No models installed";

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-neutral-100 dark:bg-neutral-800/80 hover:bg-neutral-200 dark:hover:bg-neutral-800 border border-neutral-300 dark:border-neutral-700/60 text-xs text-neutral-800 dark:text-neutral-200 transition-all cursor-pointer shadow-xs"
      >
        <span className="font-medium max-w-[200px] truncate">{displayName}</span>
        {currentModelFile && (
          <span className="text-[10px] text-neutral-500 dark:text-neutral-400 font-mono">
            {currentModelFile.size_gb} GB
          </span>
        )}
        <FiChevronDown
          className={`w-3.5 h-3.5 text-neutral-400 transition-transform ${
            isOpen ? "rotate-180" : ""
          }`}
        />
      </button>

      {isOpen && (
        <div className="absolute left-0 top-full mt-1.5 w-88 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 shadow-2xl py-2 z-50 text-xs">
          <div className="px-3 pb-2 text-[11px] font-semibold text-neutral-500 dark:text-neutral-400 uppercase tracking-wider border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between">
            <span>Installed Models ({installedModels.length})</span>
          </div>

          <div className="max-h-80 overflow-y-auto py-1 divide-y divide-neutral-100 dark:divide-neutral-800/40">
            {installedModels.length === 0 ? (
              <div className="px-3 py-6 text-neutral-500 dark:text-neutral-400 text-center italic">
                No GGUF models installed yet.
              </div>
            ) : (
              installedModels.map((m) => {
                const isSelected =
                  m.full_path === selectedModel || m.filename === selectedModel;
                return (
                  <button
                    key={m.full_path}
                    onClick={() => {
                      onSelectModel(m.full_path);
                      setIsOpen(false);
                    }}
                    className={`w-full flex items-center justify-between px-3.5 py-2.5 text-left hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors cursor-pointer ${
                      isSelected
                        ? "bg-neutral-100 dark:bg-neutral-800/60 text-neutral-900 dark:text-white font-medium"
                        : "text-neutral-700 dark:text-neutral-300"
                    }`}
                  >
                    <div className="truncate pr-3">
                      <div className="truncate font-semibold text-xs text-neutral-900 dark:text-neutral-100">
                        {m.filename.replace(".gguf", "")}
                      </div>
                      <div className="text-[11px] text-neutral-500 dark:text-neutral-400 font-mono mt-0.5">
                        {m.size_gb} GB • {m.full_path}
                      </div>
                    </div>
                    {isSelected && <FiCheck className="w-4 h-4 text-emerald-500 dark:text-emerald-400 shrink-0" />}
                  </button>
                );
              })
            )}
          </div>

          <div className="pt-2 border-t border-neutral-200 dark:border-neutral-800 px-3">
            <button
              onClick={() => {
                setIsOpen(false);
                onOpenModelsLibrary();
              }}
              className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-800 dark:text-neutral-200 text-xs font-medium transition-colors cursor-pointer"
            >
              <FiPlus className="w-3.5 h-3.5" />
              <span>Browse Models Hub ({installedModels.length} installed)</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
