import React from "react";
import {
  FiMessageSquare,
  FiLayers,
  FiCpu,
  FiSettings,
  FiPlus,
  FiTrash2,
  FiSun,
  FiMoon,
} from "react-icons/fi";
import { ChatSession, SidecarStatus } from "../../types";

export type ActiveTab = "chat" | "models" | "engine" | "settings";

interface SidebarProps {
  activeTab: ActiveTab;
  onSelectTab: (tab: ActiveTab) => void;
  sessions: ChatSession[];
  activeSessionId: string | null;
  onSelectSession: (id: string) => void;
  onNewChat: () => void;
  onDeleteSession: (e: React.MouseEvent, id: string) => void;
  theme: "dark" | "light";
  onToggleTheme: () => void;
  status: SidecarStatus | null;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeTab,
  onSelectTab,
  sessions,
  activeSessionId,
  onSelectSession,
  onNewChat,
  onDeleteSession,
  theme,
  onToggleTheme,
  status,
}) => {
  return (
    <aside className="w-64 h-full flex flex-col bg-neutral-50 dark:bg-neutral-900 border-r border-neutral-200 dark:border-neutral-800 select-none text-neutral-700 dark:text-neutral-300 shrink-0 transition-colors">
      {/* Top Controls: New Chat */}
      <div className="p-3 border-b border-neutral-200 dark:border-neutral-800">
        <button
          onClick={onNewChat}
          className="w-full flex items-center justify-between px-3 py-2 rounded-lg bg-white dark:bg-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-750 text-neutral-900 dark:text-neutral-100 font-medium text-xs border border-neutral-300 dark:border-neutral-700/60 shadow-xs transition-all group"
        >
          <div className="flex items-center gap-2">
            <FiPlus className="w-4 h-4 text-neutral-500 dark:text-neutral-400 group-hover:text-neutral-900 dark:group-hover:text-white transition-colors" />
            <span>New Chat</span>
          </div>
          <span className="text-[10px] text-neutral-400 dark:text-neutral-500 font-mono">⌘N</span>
        </button>
      </div>

      {/* Primary Navigation Tabs */}
      <div className="px-3 py-2 space-y-1 border-b border-neutral-200 dark:border-neutral-800 text-xs">
        <button
          onClick={() => onSelectTab("chat")}
          className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md transition-colors ${
            activeTab === "chat"
              ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium shadow-xs"
              : "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50"
          }`}
        >
          <FiMessageSquare className="w-4 h-4" />
          <span>Chat</span>
        </button>

        <button
          onClick={() => onSelectTab("models")}
          className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md transition-colors ${
            activeTab === "models"
              ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium shadow-xs"
              : "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50"
          }`}
        >
          <FiLayers className="w-4 h-4" />
          <span>Models Library</span>
        </button>

        <button
          onClick={() => onSelectTab("engine")}
          className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-md transition-colors ${
            activeTab === "engine"
              ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium shadow-xs"
              : "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50"
          }`}
        >
          <div className="flex items-center gap-2.5">
            <FiCpu className="w-4 h-4" />
            <span>Engine & Sidecar</span>
          </div>
          {status?.is_running && (
            <span className="w-2 h-2 rounded-full bg-emerald-500"></span>
          )}
        </button>

        <button
          onClick={() => onSelectTab("settings")}
          className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md transition-colors ${
            activeTab === "settings"
              ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium shadow-xs"
              : "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50"
          }`}
        >
          <FiSettings className="w-4 h-4" />
          <span>Settings</span>
        </button>
      </div>

      {/* Recent Chats Section */}
      <div className="flex-1 flex flex-col min-h-0 px-3 py-2">
        <div className="text-[11px] font-semibold text-neutral-400 dark:text-neutral-500 uppercase tracking-wider px-2 mb-1.5">
          History
        </div>

        <div className="flex-1 overflow-y-auto space-y-0.5 pr-1">
          {sessions.length === 0 ? (
            <div className="px-2 py-4 text-xs text-neutral-400 dark:text-neutral-500 text-center italic">
              No conversations yet
            </div>
          ) : (
            sessions.map((sess) => {
              const isActive = activeTab === "chat" && activeSessionId === sess.id;
              return (
                <div
                  key={sess.id}
                  onClick={() => {
                    onSelectTab("chat");
                    onSelectSession(sess.id);
                  }}
                  className={`group flex items-center justify-between px-2.5 py-1.5 rounded-md cursor-pointer text-xs transition-all ${
                    isActive
                      ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 font-medium"
                      : "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-200/40 dark:hover:bg-neutral-800/40"
                  }`}
                >
                  <span className="truncate flex-1 pr-2">{sess.title}</span>
                  <button
                    onClick={(e) => onDeleteSession(e, sess.id)}
                    className="opacity-0 group-hover:opacity-100 p-1 text-neutral-400 hover:text-red-500 dark:hover:text-red-400 transition-opacity"
                    title="Delete Chat"
                  >
                    <FiTrash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Footer: Theme Toggle & Engine Status */}
      <div className="p-3 border-t border-neutral-200 dark:border-neutral-800 flex items-center justify-between text-xs">
        <button
          onClick={onToggleTheme}
          className="flex items-center gap-2 p-1.5 rounded-md text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
          title={`Switch to ${theme === "dark" ? "light" : "dark"} mode`}
        >
          {theme === "dark" ? (
            <FiSun className="w-4 h-4 text-amber-400" />
          ) : (
            <FiMoon className="w-4 h-4 text-neutral-700" />
          )}
          <span className="capitalize">{theme}</span>
        </button>

        <div className="flex items-center gap-1.5 text-[11px] text-neutral-500 dark:text-neutral-400 font-mono">
          <span
            className={`w-2 h-2 rounded-full ${
              status?.is_running ? "bg-emerald-500 animate-pulse" : "bg-neutral-400 dark:bg-neutral-500"
            }`}
          ></span>
          <span>{status?.is_running ? "Running" : "Offline"}</span>
        </div>
      </div>
    </aside>
  );
};
