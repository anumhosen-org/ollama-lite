import React, { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FiSliders, FiTrash2 } from "react-icons/fi";
import {
  ChatMessage,
  ChatSession,
  ModelFileInfo,
  SidecarStatus,
  AttachedFile,
} from "../../types";
import { ModelSelector } from "./ModelSelector";
import { ChatMessageList } from "./ChatMessageList";
import { ChatInput } from "./ChatInput";
import { ChatParamsDrawer, SYSTEM_PRESETS } from "./ChatParamsDrawer";

interface ChatWorkspaceProps {
  status: SidecarStatus | null;
  installedModels: ModelFileInfo[];
  activeSessionId: string | null;
  sessions: ChatSession[];
  onRefreshModels: () => void;
  onRefreshStatus: () => void;
  onOpenModelsLibrary: () => void;
  onUpdateSessionTitle: (id: string, title: string) => void;
}

export const ChatWorkspace: React.FC<ChatWorkspaceProps> = ({
  status,
  installedModels,
  activeSessionId,
  sessions,
  onRefreshStatus,
  onOpenModelsLibrary,
  onUpdateSessionTitle,
}) => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState<string>("");
  const [isGenerating, setIsGenerating] = useState<boolean>(false);
  const [selectedModel, setSelectedModel] = useState<string>("");

  // Hyperparameters
  const [systemPrompt, setSystemPrompt] = useState<string>(SYSTEM_PRESETS[0].prompt);
  const [temperature, setTemperature] = useState<number>(0.7);
  const [contextSize, setContextSize] = useState<number>(8192);
  const [gpuLayers, setGpuLayers] = useState<number>(99);
  const [cpuThreads, setCpuThreads] = useState<number>(8);
  const [showParamsDrawer, setShowParamsDrawer] = useState<boolean>(false);

  // Attachments & Grounding
  const [attachedFiles, setAttachedFiles] = useState<AttachedFile[]>([]);
  const [isWebSearchEnabled, setIsWebSearchEnabled] = useState<boolean>(false);

  const abortControllerRef = useRef<AbortController | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto select first installed model if none selected
  useEffect(() => {
    if (!selectedModel && installedModels.length > 0) {
      setSelectedModel(installedModels[0].full_path);
    }
  }, [installedModels, selectedModel]);

  // Load session messages when active session changes
  useEffect(() => {
    if (activeSessionId) {
      loadSessionMessages(activeSessionId);
    }
  }, [activeSessionId]);

  const loadSessionMessages = async (sessionId: string) => {
    try {
      const dbMsgs: any[] = await invoke("get_session_messages", { sessionId });
      if (dbMsgs.length > 0) {
        setMessages(
          dbMsgs.map((m) => {
            let content = m.content;
            let thinking: string | undefined = undefined;

            // Extract <think> if saved
            if (content.includes("<think>")) {
              const match = content.match(/<think>([\s\S]*?)<\/think>/);
              if (match) {
                thinking = match[1].trim();
                content = content.replace(/<think>[\s\S]*?<\/think>/, "").trim();
              }
            }

            return {
              id: m.id,
              role: m.role,
              content,
              thinking,
              timestamp: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
            };
          })
        );
      } else {
        setMessages([
          {
            id: "welcome",
            role: "assistant",
            content: "Hello! I am your local AI assistant running via **Ollama Lite**. How can I help you today?",
            timestamp: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
          },
        ]);
      }
    } catch (err) {
      console.error("Failed loading session messages:", err);
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, isGenerating]);

  const handleSelectModel = async (modelPath: string) => {
    setSelectedModel(modelPath);
    // Restart sidecar with new model
    try {
      await invoke("start_sidecar", {
        modelPath,
        host: "127.0.0.1",
        port: 11434,
        nGpuLayers: gpuLayers,
        contextSize,
        threads: cpuThreads,
      });
      onRefreshStatus();
    } catch (err) {
      console.error("Failed switching sidecar model:", err);
    }
  };

  const handleSendMessage = async () => {
    const trimmed = input.trim();
    if (!trimmed || isGenerating) return;

    // Check if sidecar is running; if not, attempt to start it
    if (!status?.is_running) {
      if (selectedModel) {
        try {
          await invoke("start_sidecar", {
            modelPath: selectedModel,
            host: "127.0.0.1",
            port: 11434,
            nGpuLayers: gpuLayers,
            contextSize,
            threads: cpuThreads,
          });
          onRefreshStatus();
        } catch (err) {
          alert(`Failed to start sidecar engine: ${err}`);
          return;
        }
      } else if (installedModels.length > 0) {
        await handleSelectModel(installedModels[0].full_path);
      } else {
        alert("Please download a GGUF model in the Models tab first!");
        onOpenModelsLibrary();
        return;
      }
    }

    setInput("");
    const userMessageText = trimmed;

    // Generate auto-title for first user message
    if (activeSessionId) {
      const currentSess = sessions.find((s) => s.id === activeSessionId);
      if (currentSess && (currentSess.title === "New Conversation" || messages.length <= 1)) {
        const words = userMessageText.split(/\s+/).slice(0, 5).join(" ");
        const title = words.length > 25 ? `${words.slice(0, 25)}...` : words;
        onUpdateSessionTitle(activeSessionId, title);
        invoke("update_chat_session_title", { sessionId: activeSessionId, title }).catch(console.error);
      }
    }

    // Format context with files and search
    let enrichedContent = userMessageText;
    if (attachedFiles.length > 0) {
      const docs = attachedFiles
        .map((f) => `<context_document name="${f.name}">\n${f.content.slice(0, 3000)}\n</context_document>`)
        .join("\n\n");
      enrichedContent = `${docs}\n\n[User Request]:\n${userMessageText}`;
    }

    if (isWebSearchEnabled) {
      enrichedContent = `[Web Search Active]\n${enrichedContent}`;
    }

    setAttachedFiles([]);

    // Save user message to database
    if (activeSessionId) {
      invoke("save_chat_message", {
        sessionId: activeSessionId,
        role: "user",
        content: userMessageText,
      }).catch(console.error);
    }

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: "user",
      content: userMessageText,
      timestamp: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
    };

    const assistantMsgId = (Date.now() + 1).toString();
    const assistantMessage: ChatMessage = {
      id: assistantMsgId,
      role: "assistant",
      content: "",
      thinking: "",
      isThinking: false,
      timestamp: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
    };

    setMessages((prev) => [...prev, userMessage, assistantMessage]);
    setIsGenerating(true);
    abortControllerRef.current = new AbortController();

    const apiUrl = `${status?.api_url || "http://127.0.0.1:11434"}/v1/chat/completions`;
    const payloadMessages = [
      { role: "system", content: systemPrompt },
      ...messages
        .filter((m) => m.role !== "system" && m.id !== "welcome")
        .map((m) => ({ role: m.role, content: m.content })),
      { role: "user", content: enrichedContent },
    ];

    const startTime = Date.now();
    let accumulatedRaw = "";
    let accumulatedContent = "";
    let accumulatedThinking = "";
    let inThinkTag = false;

    try {
      const res = await fetch(apiUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          messages: payloadMessages,
          temperature,
          stream: true,
        }),
        signal: abortControllerRef.current.signal,
      });

      if (!res.body) throw new Error("SSE stream not readable");
      const reader = res.body.getReader();
      const decoder = new TextDecoder("utf-8");

      while (true) {
        const { value, done } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value, { stream: true });
        const lines = chunk.split("\n");

        for (const line of lines) {
          const trimmedLine = line.trim();
          if (trimmedLine.startsWith("data: ")) {
            const dataStr = trimmedLine.slice(6);
            if (dataStr === "[DONE]") break;

            try {
              const parsed = JSON.parse(dataStr);
              const delta = parsed.choices?.[0]?.delta;

              if (delta) {
                // Check for reasoning_content (DeepSeek-R1 standard)
                if (delta.reasoning_content) {
                  accumulatedThinking += delta.reasoning_content;
                  setMessages((prev) =>
                    prev.map((m) =>
                      m.id === assistantMsgId
                        ? { ...m, thinking: accumulatedThinking, isThinking: true }
                        : m
                    )
                  );
                }

                // Check for content tokens
                if (delta.content) {
                  const token = delta.content;
                  accumulatedRaw += token;

                  if (token.includes("<think>")) {
                    inThinkTag = true;
                    setMessages((prev) =>
                      prev.map((m) =>
                        m.id === assistantMsgId ? { ...m, isThinking: true } : m
                      )
                    );
                  }

                  if (inThinkTag) {
                    if (token.includes("</think>")) {
                      inThinkTag = false;
                      const parts = token.split("</think>");
                      accumulatedThinking += parts[0].replace("<think>", "");
                      accumulatedContent += parts[1] || "";
                      const thinkingDuration = (Date.now() - startTime) / 1000;
                      setMessages((prev) =>
                        prev.map((m) =>
                          m.id === assistantMsgId
                            ? {
                                ...m,
                                thinking: accumulatedThinking.trim(),
                                isThinking: false,
                                thinkingTime: thinkingDuration,
                                content: accumulatedContent,
                              }
                            : m
                        )
                      );
                    } else {
                      accumulatedThinking += token.replace("<think>", "");
                      setMessages((prev) =>
                        prev.map((m) =>
                          m.id === assistantMsgId
                            ? { ...m, thinking: accumulatedThinking, isThinking: true }
                            : m
                        )
                      );
                    }
                  } else {
                    accumulatedContent += token.replace("</think>", "");
                    setMessages((prev) =>
                      prev.map((m) =>
                        m.id === assistantMsgId
                          ? { ...m, content: accumulatedContent, isThinking: false }
                          : m
                      )
                    );
                  }
                }
              }
            } catch {
              // Ignore non-json chunks
            }
          }
        }
      }

      // Save complete assistant message to database
      if (activeSessionId) {
        const fullToSave = accumulatedThinking
          ? `<think>${accumulatedThinking}</think>\n${accumulatedContent}`
          : accumulatedContent;
        invoke("save_chat_message", {
          sessionId: activeSessionId,
          role: "assistant",
          content: fullToSave,
        }).catch(console.error);
      }
    } catch (err: any) {
      if (err.name !== "AbortError") {
        console.error("Inference request failed:", err);
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantMsgId
              ? {
                  ...m,
                  content: `${m.content}\n\n*[Error: Could not reach llama-server on port 11434. Make sure the engine is running in the Engine tab.]*`,
                  isThinking: false,
                }
              : m
          )
        );
      }
    } finally {
      setIsGenerating(false);
      abortControllerRef.current = null;
    }
  };

  const handleStopGeneration = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      setIsGenerating(false);
    }
  };

  const handleClearMessages = () => {
    setMessages([
      {
        id: "cleared",
        role: "assistant",
        content: "Chat cleared. What would you like to explore next?",
        timestamp: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
      },
    ]);
  };

  return (
    <div className="flex-1 h-full flex flex-col relative overflow-hidden bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 select-none transition-colors">
      {/* Top Workspace Header */}
      <div className="h-12 border-b border-neutral-200 dark:border-neutral-800 px-4 flex items-center justify-between bg-white/90 dark:bg-neutral-900/90 backdrop-blur-sm z-20 transition-colors">
        <ModelSelector
          installedModels={installedModels}
          selectedModel={selectedModel}
          onSelectModel={handleSelectModel}
          status={status}
          onOpenModelsLibrary={onOpenModelsLibrary}
        />

        <div className="flex items-center gap-1">
          <button
            onClick={handleClearMessages}
            className="p-1.5 rounded-md text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
            title="Clear Chat"
          >
            <FiTrash2 className="w-4 h-4" />
          </button>
          <button
            onClick={() => setShowParamsDrawer(!showParamsDrawer)}
            className={`p-1.5 rounded-md transition-colors ${
              showParamsDrawer
                ? "bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white"
                : "text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800"
            }`}
            title="Parameters & System Prompt"
          >
            <FiSliders className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Messages Scroll Area */}
      <ChatMessageList
        messages={messages}
        isGenerating={isGenerating}
        messagesEndRef={messagesEndRef}
      />

      {/* Floating Bottom Input */}
      <ChatInput
        input={input}
        setInput={setInput}
        onSend={handleSendMessage}
        isGenerating={isGenerating}
        onStop={handleStopGeneration}
        attachedFiles={attachedFiles}
        setAttachedFiles={setAttachedFiles}
        isWebSearchEnabled={isWebSearchEnabled}
        setIsWebSearchEnabled={setIsWebSearchEnabled}
      />

      {/* Side Hyperparameters Drawer */}
      <ChatParamsDrawer
        isOpen={showParamsDrawer}
        onClose={() => setShowParamsDrawer(false)}
        temperature={temperature}
        setTemperature={setTemperature}
        contextSize={contextSize}
        setContextSize={setContextSize}
        gpuLayers={gpuLayers}
        setGpuLayers={setGpuLayers}
        cpuThreads={cpuThreads}
        setCpuThreads={setCpuThreads}
        systemPrompt={systemPrompt}
        setSystemPrompt={setSystemPrompt}
      />
    </div>
  );
};
