import React from "react";
import { FiCopy, FiCheck, FiUser } from "react-icons/fi";
import { OllamaLogo } from "../common/OllamaLogo";
import { ThinkingBlock } from "./ThinkingBlock";
import { MarkdownRenderer } from "./MarkdownRenderer";
import { ChatMessage } from "../../types";

interface ChatMessageListProps {
  messages: ChatMessage[];
  isGenerating: boolean;
  messagesEndRef: React.RefObject<HTMLDivElement | null>;
}

export const ChatMessageList: React.FC<ChatMessageListProps> = ({
  messages,
  isGenerating,
  messagesEndRef,
}) => {
  return (
    <div className="flex-1 overflow-y-auto px-4 py-6 select-text">
      <div className="max-w-3xl mx-auto space-y-6">
        {messages.map((msg, index) => {
          const isAssistant = msg.role === "assistant";
          const isLatestAssistant =
            isAssistant && index === messages.length - 1 && isGenerating;

          return (
            <div
              key={msg.id}
              className={`flex gap-3 text-xs ${
                isAssistant ? "justify-start" : "justify-end"
              }`}
            >
              {isAssistant && (
                <div className="w-6 h-6 rounded-full bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center shrink-0 mt-0.5 border border-neutral-300 dark:border-neutral-700 text-neutral-800 dark:text-neutral-300 shadow-xs">
                  <OllamaLogo className="w-5 h-5 text-neutral-900 dark:text-white" />
                </div>
              )}

              <div
                className={`flex flex-col max-w-[85%] ${
                  isAssistant ? "items-start w-full" : "items-end"
                }`}
              >
                {/* User message bubble */}
                {!isAssistant ? (
                  <div className="px-4 py-2.5 rounded-2xl bg-neutral-100 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 font-sans leading-relaxed border border-neutral-300 dark:border-neutral-750 shadow-xs">
                    <p className="whitespace-pre-wrap">{msg.content}</p>
                  </div>
                ) : (
                  <div className="w-full">
                    {/* Thinking process block */}
                    {msg.thinking && (
                      <ThinkingBlock
                        thinking={msg.thinking}
                        isActivelyThinking={msg.isThinking}
                        durationSeconds={msg.thinkingTime}
                      />
                    )}

                    {/* Markdown response content */}
                    {msg.content ? (
                      <MarkdownRenderer content={msg.content} />
                    ) : isLatestAssistant && !msg.thinking ? (
                      <div className="flex items-center gap-1.5 py-2 text-neutral-500 dark:text-neutral-400">
                        <span className="w-1.5 h-1.5 rounded-full bg-neutral-500 dark:bg-neutral-400 animate-bounce"></span>
                        <span className="w-1.5 h-1.5 rounded-full bg-neutral-500 dark:bg-neutral-400 animate-bounce [animation-delay:0.2s]"></span>
                        <span className="w-1.5 h-1.5 rounded-full bg-neutral-500 dark:bg-neutral-400 animate-bounce [animation-delay:0.4s]"></span>
                      </div>
                    ) : null}

                    {/* Assistant Message Footer / Actions */}
                    {msg.content && !isLatestAssistant && (
                      <div className="flex items-center gap-2 mt-2 text-[10px] text-neutral-500 select-none">
                        <CopyMessageButton text={msg.content} />
                        <span>•</span>
                        <span>{msg.timestamp}</span>
                      </div>
                    )}
                  </div>
                )}
              </div>

              {!isAssistant && (
                <div className="w-6 h-6 rounded-full bg-neutral-200 dark:bg-neutral-700 flex items-center justify-center shrink-0 mt-0.5 text-neutral-700 dark:text-neutral-300">
                  <FiUser className="w-3.5 h-3.5" />
                </div>
              )}
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>
    </div>
  );
};

const CopyMessageButton: React.FC<{ text: string }> = ({ text }) => {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <button
      onClick={handleCopy}
      className="flex items-center gap-1 text-neutral-500 hover:text-neutral-300 transition-colors"
      title="Copy response"
    >
      {copied ? (
        <>
          <FiCheck className="w-3 h-3 text-emerald-400" />
          <span className="text-emerald-400">Copied</span>
        </>
      ) : (
        <>
          <FiCopy className="w-3 h-3" />
          <span>Copy</span>
        </>
      )}
    </button>
  );
};
