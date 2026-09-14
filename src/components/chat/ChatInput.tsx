import React, { useRef, useEffect } from "react";
import { FiArrowUp, FiSquare, FiPaperclip, FiX, FiGlobe } from "react-icons/fi";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { AttachedFile } from "../../types";

interface ChatInputProps {
  input: string;
  setInput: (v: string) => void;
  onSend: () => void;
  isGenerating: boolean;
  onStop: () => void;
  attachedFiles: AttachedFile[];
  setAttachedFiles: React.Dispatch<React.SetStateAction<AttachedFile[]>>;
  isWebSearchEnabled: boolean;
  setIsWebSearchEnabled: (v: boolean) => void;
  disabled?: boolean;
}

export const ChatInput: React.FC<ChatInputProps> = ({
  input,
  setInput,
  onSend,
  isGenerating,
  onStop,
  attachedFiles,
  setAttachedFiles,
  isWebSearchEnabled,
  setIsWebSearchEnabled,
  disabled = false,
}) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 180)}px`;
    }
  }, [input]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (!isGenerating && input.trim()) {
        onSend();
      }
    }
  };

  const handleAttachFile = async () => {
    try {
      const selected = await openFileDialog({
        multiple: false,
        filters: [
          {
            name: "Text & Documents",
            extensions: ["txt", "md", "json", "csv", "py", "rs", "js", "ts", "tsx", "html", "css"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        // Read text content using browser fetch or file API
        // For local files in Tauri, we can read via text response or fetch
        const res = await fetch(`https://asset.localhost/${selected}`);
        let content = "";
        if (res.ok) {
          content = await res.text();
        } else {
          content = `[Attached local file: ${selected}]`;
        }

        const fileName = selected.split(/[\/\\]/).pop() || "file.txt";
        setAttachedFiles((prev) => [
          ...prev,
          {
            id: Math.random().toString(36).substring(2, 9),
            name: fileName,
            content,
            size: content.length,
          },
        ]);
      }
    } catch (err) {
      console.error(err);
    }
  };

  const removeAttachment = (id: string) => {
    setAttachedFiles((prev) => prev.filter((f) => f.id !== id));
  };

  return (
    <div className="w-full max-w-3xl mx-auto px-4 pb-4">
      {/* Attached Files Pills */}
      {attachedFiles.length > 0 && (
        <div className="flex flex-wrap gap-2 mb-2">
          {attachedFiles.map((file) => (
            <div
              key={file.id}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-neutral-100 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 text-xs text-neutral-800 dark:text-neutral-300 select-none shadow-xs"
            >
              <FiPaperclip className="w-3 h-3 text-neutral-500 dark:text-neutral-400" />
              <span className="truncate max-w-[150px] font-medium">{file.name}</span>
              <button
                onClick={() => removeAttachment(file.id)}
                className="hover:text-neutral-900 dark:hover:text-white transition-colors ml-0.5"
              >
                <FiX className="w-3 h-3" />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Main Input Box (Ollama / ChatGPT style) */}
      <div className="relative rounded-2xl bg-neutral-50 dark:bg-neutral-800/90 border border-neutral-300 dark:border-neutral-700/80 shadow-md focus-within:border-neutral-400 dark:focus-within:border-neutral-500 focus-within:ring-1 focus-within:ring-neutral-400 dark:focus-within:ring-neutral-500 transition-all">
        <textarea
          ref={textareaRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          placeholder="Send a message..."
          rows={1}
          className="w-full pt-3 pb-10 pl-4 pr-12 bg-transparent text-neutral-900 dark:text-neutral-100 text-xs focus:outline-none resize-none placeholder-neutral-400 dark:placeholder-neutral-500 leading-relaxed font-sans"
        />

        {/* Bottom Toolbar inside input */}
        <div className="absolute left-3 bottom-2.5 flex items-center gap-2">
          <button
            onClick={handleAttachFile}
            className="p-1 rounded-md text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-700/50 transition-colors cursor-pointer"
            title="Attach file"
          >
            <FiPaperclip className="w-3.5 h-3.5" />
          </button>

          <button
            onClick={() => setIsWebSearchEnabled(!isWebSearchEnabled)}
            className={`flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] transition-colors cursor-pointer ${
              isWebSearchEnabled
                ? "bg-neutral-200 dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100 font-medium"
                : "text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-200/40 dark:hover:bg-neutral-700/40"
            }`}
            title="Toggle Web Grounding"
          >
            <FiGlobe className="w-3 h-3" />
            <span>Search</span>
          </button>
        </div>

        {/* Send or Stop Button */}
        <div className="absolute right-2.5 bottom-2.5">
          {isGenerating ? (
            <button
              onClick={onStop}
              className="w-7 h-7 rounded-full bg-neutral-900 text-white dark:bg-white dark:text-black flex items-center justify-center hover:bg-neutral-800 dark:hover:bg-neutral-200 transition-colors shadow-sm cursor-pointer"
              title="Stop generating"
            >
              <FiSquare className="w-3 h-3 fill-current" />
            </button>
          ) : (
            <button
              onClick={onSend}
              disabled={!input.trim() || disabled}
              className={`w-7 h-7 rounded-full flex items-center justify-center transition-all ${
                input.trim() && !disabled
                  ? "bg-neutral-900 text-white hover:bg-neutral-800 dark:bg-white dark:text-black dark:hover:bg-neutral-200 cursor-pointer shadow-sm"
                  : "bg-neutral-200 text-neutral-400 dark:bg-neutral-700 dark:text-neutral-500 cursor-not-allowed"
              }`}
              title="Send (Enter)"
            >
              <FiArrowUp className="w-4 h-4 stroke-[2.5]" />
            </button>
          )}
        </div>
      </div>

      <div className="text-[10px] text-neutral-400 dark:text-neutral-500 text-center mt-2 select-none">
        Ollama Lite uses local offline inference via llama.cpp. Shift + Enter for new line.
      </div>
    </div>
  );
};
