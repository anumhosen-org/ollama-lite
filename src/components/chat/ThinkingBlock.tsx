import React, { useState } from "react";
import { FiChevronDown, FiChevronRight } from "react-icons/fi";
import { HiOutlineLightBulb } from "react-icons/hi2";

interface ThinkingBlockProps {
  thinking: string;
  isActivelyThinking?: boolean;
  durationSeconds?: number;
}

export const ThinkingBlock: React.FC<ThinkingBlockProps> = ({
  thinking,
  isActivelyThinking = false,
  durationSeconds,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);

  if (!thinking && !isActivelyThinking) return null;

  return (
    <div className="my-2 rounded-lg border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900/80 text-xs overflow-hidden transition-all select-text">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between px-3 py-2 text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800/40 transition-colors select-none"
      >
        <div className="flex items-center gap-2">
          <HiOutlineLightBulb
            className={`w-4 h-4 ${
              isActivelyThinking ? "text-amber-500 dark:text-amber-400 animate-pulse" : "text-neutral-400 dark:text-neutral-500"
            }`}
          />
          <span className="font-medium text-neutral-800 dark:text-neutral-300">
            {isActivelyThinking
              ? "Thinking..."
              : durationSeconds
              ? `Thought for ${durationSeconds.toFixed(1)} seconds`
              : "Thought process"}
          </span>
        </div>

        <div className="flex items-center gap-1 text-neutral-500 dark:text-neutral-400">
          <span>{isExpanded ? "Hide" : "Show"}</span>
          {isExpanded ? (
            <FiChevronDown className="w-3.5 h-3.5" />
          ) : (
            <FiChevronRight className="w-3.5 h-3.5" />
          )}
        </div>
      </button>

      {isExpanded && (
        <div className="px-3.5 py-2.5 border-t border-neutral-200 dark:border-neutral-800/80 text-neutral-700 dark:text-neutral-400 font-mono text-[11px] leading-relaxed whitespace-pre-wrap max-h-80 overflow-y-auto bg-neutral-100/70 dark:bg-neutral-950/40 select-text">
          {thinking}
        </div>
      )}
    </div>
  );
};
