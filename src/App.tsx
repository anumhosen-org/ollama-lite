import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Titlebar } from "./components/titlebar/Titlebar";
import { Sidebar, ActiveTab } from "./components/navigation/Sidebar";
import { ChatWorkspace } from "./components/chat/ChatWorkspace";
import { ModelsView } from "./components/models/ModelsView";
import { EngineView } from "./components/engine/EngineView";
import { SettingsView } from "./components/settings/SettingsView";
import {
  ChatSession,
  DownloadProgressPayload,
  ModelFileInfo,
  SidecarStatus,
} from "./types";

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<ActiveTab>("chat");
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [installedModels, setInstalledModels] = useState<ModelFileInfo[]>([]);
  const [status, setStatus] = useState<SidecarStatus | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgressPayload | null>(null);
  const [theme, setTheme] = useState<"dark" | "light">(() => {
    const cached = localStorage.getItem("app_theme");
    return cached === "light" ? "light" : "dark";
  });

  useEffect(() => {
    applyTheme(theme);
    initApp();

    // Listen for real-time download progress events from Rust
    const unlistenProgress = listen<DownloadProgressPayload>(
      "download-progress",
      (event) => {
        setDownloadProgress(event.payload);
        if (event.payload.is_complete) {
          refreshModels();
          setTimeout(() => setDownloadProgress(null), 3000);
        }
      }
    );

    // Periodic heartbeat to refresh sidecar runtime status
    const timer = setInterval(() => {
      refreshStatus();
    }, 3000);

    return () => {
      unlistenProgress.then((fn) => fn());
      clearInterval(timer);
    };
  }, []);

  const initApp = async () => {
    await refreshModels();
    await refreshStatus();
    await loadSessions();
    await loadTheme();
  };

  const loadTheme = async () => {
    try {
      const savedTheme = await invoke<string | null>("get_db_setting", { key: "app_theme" });
      if (savedTheme === "light" || savedTheme === "dark") {
        setTheme(savedTheme);
        applyTheme(savedTheme);
      }
    } catch {
      // Keep cached or default theme
    }
  };

  const applyTheme = (t: "dark" | "light") => {
    localStorage.setItem("app_theme", t);
    if (t === "dark") {
      document.documentElement.classList.add("dark");
      document.documentElement.classList.remove("light");
    } else {
      document.documentElement.classList.remove("dark");
      document.documentElement.classList.add("light");
    }
  };

  const toggleTheme = async () => {
    const next = theme === "dark" ? "light" : "dark";
    setTheme(next);
    applyTheme(next);
    try {
      await invoke("set_db_setting", { key: "app_theme", value: next });
    } catch (err) {
      console.error(err);
    }
  };

  const refreshModels = async () => {
    try {
      const list: ModelFileInfo[] = await invoke("list_installed_models", { customDir: null });
      setInstalledModels(list);
    } catch (err) {
      console.error("Failed listing models:", err);
    }
  };

  const refreshStatus = async () => {
    try {
      const st: SidecarStatus = await invoke("get_sidecar_status");
      setStatus(st);
    } catch (err) {
      console.error("Failed fetching sidecar status:", err);
    }
  };

  const loadSessions = async () => {
    try {
      const list: ChatSession[] = await invoke("get_chat_sessions");
      setSessions(list);
      if (list.length > 0) {
        setActiveSessionId(list[0].id);
      } else {
        await handleNewChat();
      }
    } catch (err) {
      console.error("Failed loading sessions:", err);
    }
  };

  const handleNewChat = async () => {
    try {
      const newSess: ChatSession = await invoke("create_chat_session", {
        title: "New Conversation",
      });
      setSessions((prev) => [newSess, ...prev]);
      setActiveSessionId(newSess.id);
      setActiveTab("chat");
    } catch (err) {
      console.error("Failed creating chat session:", err);
    }
  };

  const handleDeleteSession = async (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    try {
      await invoke("delete_chat_session", { sessionId: id });
      const remaining = sessions.filter((s) => s.id !== id);
      setSessions(remaining);
      if (activeSessionId === id) {
        if (remaining.length > 0) {
          setActiveSessionId(remaining[0].id);
        } else {
          await handleNewChat();
        }
      }
    } catch (err) {
      console.error("Failed deleting chat session:", err);
    }
  };

  const handleUpdateSessionTitle = (id: string, title: string) => {
    setSessions((prev) =>
      prev.map((s) => (s.id === id ? { ...s, title } : s))
    );
  };

  const handleSelectModelFromList = async (modelPath: string) => {
    try {
      await invoke("start_sidecar", {
        modelPath,
        host: "127.0.0.1",
        port: 11434,
        nGpuLayers: 99,
        contextSize: 8192,
        threads: 8,
      });
      await refreshStatus();
      setActiveTab("chat");
    } catch (err) {
      alert(`Failed starting model: ${err}`);
    }
  };

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-neutral-100 dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 font-sans transition-colors">
      {/* Custom Window Titlebar */}
      <Titlebar status={status} />

      {/* Main Workspace Layout */}
      <div className="flex flex-1 overflow-hidden">
        {/* Navigation Sidebar */}
        <Sidebar
          activeTab={activeTab}
          onSelectTab={setActiveTab}
          sessions={sessions}
          activeSessionId={activeSessionId}
          onSelectSession={setActiveSessionId}
          onNewChat={handleNewChat}
          onDeleteSession={handleDeleteSession}
          theme={theme}
          onToggleTheme={toggleTheme}
          status={status}
        />

        {/* Active View Container */}
        <main className="flex-1 flex overflow-hidden relative">
          {activeTab === "chat" && (
            <ChatWorkspace
              status={status}
              installedModels={installedModels}
              activeSessionId={activeSessionId}
              sessions={sessions}
              onRefreshModels={refreshModels}
              onRefreshStatus={refreshStatus}
              onOpenModelsLibrary={() => setActiveTab("models")}
              onUpdateSessionTitle={handleUpdateSessionTitle}
            />
          )}

          {activeTab === "models" && (
            <ModelsView
              installedModels={installedModels}
              downloadProgress={downloadProgress}
              onRefreshModels={refreshModels}
              activeModelPath={status?.current_model || undefined}
              onSelectModel={handleSelectModelFromList}
            />
          )}

          {activeTab === "engine" && (
            <EngineView
              status={status}
              onRefreshStatus={refreshStatus}
              installedModelsCount={installedModels.length}
            />
          )}

          {activeTab === "settings" && <SettingsView />}
        </main>
      </div>
    </div>
  );
};

export default App;
