import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import {
  VscChromeMinimize,
  VscChromeMaximize,
  VscChromeRestore,
  VscChromeClose,
} from "react-icons/vsc";
import { OllamaLogo } from "../common/OllamaLogo";
import { SidecarStatus } from "../../types";

interface TitlebarProps {
  status: SidecarStatus | null;
}

export const Titlebar: React.FC<TitlebarProps> = ({ status }) => {
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    const updateMaximized = async () => {
      try {
        const max = await appWindow.isMaximized();
        setIsMaximized(max);
      } catch {
        // Fallback for non-tauri contexts
      }
    };

    updateMaximized();
    const unlisten = appWindow.onResized(() => {
      updateMaximized();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleMinimize = async () => {
    try {
      await appWindow.minimize();
    } catch (err) {
      console.error(err);
    }
  };

  const handleToggleMaximize = async () => {
    try {
      await appWindow.toggleMaximize();
      const max = await appWindow.isMaximized();
      setIsMaximized(max);
    } catch (err) {
      console.error(err);
    }
  };

  const handleClose = async () => {
    try {
      await invoke("hide_window");
    } catch {
      try {
        await appWindow.close();
      } catch {
        try {
          await appWindow.hide();
        } catch (err) {
          console.error("Failed to minimize window to tray:", err);
        }
      }
    }
  };

  return (
    <header
      data-tauri-drag-region
      className="h-9 w-full flex items-center justify-between px-3 select-none bg-neutral-100 dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800 text-neutral-700 dark:text-neutral-300 z-50 text-xs shrink-0 transition-colors"
    >
      {/* Left: Branding & Model pill */}
      <div className="flex items-center gap-2 pointer-events-none" data-tauri-drag-region>
        <OllamaLogo className="w-10 h-10 text-neutral-900 dark:text-white" />
        <span className="font-semibold text-neutral-900 dark:text-neutral-100 tracking-tight">Ollama Lite</span>
        <span className="text-neutral-500 font-mono text-[10px]">v0.1.0</span>

        {status?.is_running ? (
          <div className="flex items-center gap-1.5 ml-2 px-2 py-0.5 rounded-full bg-neutral-200 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 text-[11px] text-neutral-800 dark:text-neutral-300">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
            <span className="truncate max-w-[140px] font-mono">
              {status.current_model?.replace(".gguf", "") || "llama-server"}
            </span>
            <span className="text-neutral-500">:11434</span>
          </div>
        ) : (
          <div className="flex items-center gap-1.5 ml-2 px-2 py-0.5 rounded-full bg-neutral-200/60 dark:bg-neutral-800/60 border border-neutral-300 dark:border-neutral-800 text-[10px] text-neutral-500 dark:text-neutral-400">
            <span className="w-1.5 h-1.5 rounded-full bg-neutral-400 dark:bg-neutral-500"></span>
            <span>Idle</span>
          </div>
        )}
      </div>

      {/* Center Drag Region */}
      <div className="flex-1 h-full" data-tauri-drag-region />

      {/* Right: Window Controls */}
      <div data-tauri-drag-region="false" className="flex items-center -mr-3 h-full">
        <button
          type="button"
          data-tauri-drag-region="false"
          onClick={handleMinimize}
          className="h-full px-3 flex items-center justify-center text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
          title="Minimize"
        >
          <VscChromeMinimize className="w-3.5 h-3.5 pointer-events-none" />
        </button>
        <button
          type="button"
          data-tauri-drag-region="false"
          onClick={handleToggleMaximize}
          className="h-full px-3 flex items-center justify-center text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
          title={isMaximized ? "Restore" : "Maximize"}
        >
          {isMaximized ? (
            <VscChromeRestore className="w-4.5 h-4.5 pointer-events-none" />
          ) : (
            <VscChromeMaximize className="w-3.5 h-3.5 pointer-events-none" />
          )}
        </button>
        <button
          type="button"
          data-tauri-drag-region="false"
          onClick={handleClose}
          className="h-full px-3 flex items-center justify-center text-neutral-500 dark:text-neutral-400 hover:text-white hover:bg-red-600 transition-colors"
          title="Minimize to System Tray"
        >
          <VscChromeClose className="w-3.5 h-3.5 pointer-events-none" />
        </button>
      </div>
    </header>
  );
};
