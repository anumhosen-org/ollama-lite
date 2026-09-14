import React, { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { FiCopy, FiCheck } from "react-icons/fi";

interface MarkdownRendererProps {
  content: string;
}

export const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({ content }) => {
  return (
    <div className="prose prose-neutral dark:prose-invert max-w-none text-xs leading-relaxed break-words">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          code({ className, children, ...props }) {
            const match = /language-(\w+)/.exec(className || "");
            const codeString = String(children).replace(/\n$/, "");
            const isInline = !match && !String(children).includes("\n");

            if (isInline) {
              return (
                <code
                  className="px-1.5 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800 text-neutral-800 dark:text-neutral-200 font-mono text-[11px]"
                  {...props}
                >
                  {children}
                </code>
              );
            }

            return (
              <CodeBlock language={match ? match[1] : "text"} code={codeString}>
                {children}
              </CodeBlock>
            );
          },
          p({ children }) {
            return <p className="mb-2.5 last:mb-0 text-neutral-800 dark:text-neutral-200 leading-relaxed">{children}</p>;
          },
          ul({ children }) {
            return <ul className="list-disc pl-5 mb-2.5 space-y-1 text-neutral-700 dark:text-neutral-300">{children}</ul>;
          },
          ol({ children }) {
            return <ol className="list-decimal pl-5 mb-2.5 space-y-1 text-neutral-700 dark:text-neutral-300">{children}</ol>;
          },
          li({ children }) {
            return <li className="text-neutral-700 dark:text-neutral-300">{children}</li>;
          },
          h1({ children }) {
            return <h1 className="text-base font-semibold text-neutral-900 dark:text-neutral-100 mt-3 mb-2">{children}</h1>;
          },
          h2({ children }) {
            return <h2 className="text-sm font-semibold text-neutral-900 dark:text-neutral-100 mt-2.5 mb-1.5">{children}</h2>;
          },
          h3({ children }) {
            return <h3 className="text-xs font-semibold text-neutral-900 dark:text-neutral-200 mt-2 mb-1">{children}</h3>;
          },
          table({ children }) {
            return (
              <div className="overflow-x-auto my-2.5 border border-neutral-200 dark:border-neutral-800 rounded-lg">
                <table className="w-full text-left text-xs border-collapse">{children}</table>
              </div>
            );
          },
          th({ children }) {
            return (
              <th className="px-3 py-1.5 bg-neutral-100 dark:bg-neutral-800/80 font-medium text-neutral-800 dark:text-neutral-200 border-b border-neutral-200 dark:border-neutral-700">
                {children}
              </th>
            );
          },
          td({ children }) {
            return (
              <td className="px-3 py-1.5 border-b border-neutral-200 dark:border-neutral-800 text-neutral-700 dark:text-neutral-300">
                {children}
              </td>
            );
          },
          blockquote({ children }) {
            return (
              <blockquote className="border-l-2 border-neutral-300 dark:border-neutral-700 pl-3 my-2 text-neutral-600 dark:text-neutral-400 italic">
                {children}
              </blockquote>
            );
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

interface CodeBlockProps {
  language: string;
  code: string;
  children: React.ReactNode;
}

const CodeBlock: React.FC<CodeBlockProps> = ({ language, code, children }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="my-3 rounded-lg overflow-hidden border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-950 text-xs">
      <div className="flex items-center justify-between px-3 py-1.5 bg-neutral-100 dark:bg-neutral-900/90 border-b border-neutral-200 dark:border-neutral-800 text-[11px] text-neutral-600 dark:text-neutral-400 font-mono">
        <span>{language}</span>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1 hover:text-neutral-900 dark:hover:text-white transition-colors cursor-pointer"
          title="Copy code"
        >
          {copied ? (
            <>
              <FiCheck className="w-3.5 h-3.5 text-emerald-500 dark:text-emerald-400" />
              <span className="text-emerald-500 dark:text-emerald-400">Copied</span>
            </>
          ) : (
            <>
              <FiCopy className="w-3.5 h-3.5" />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>
      <pre className="p-3 overflow-x-auto text-neutral-800 dark:text-neutral-200 font-mono text-[11.5px] leading-relaxed">
        <code>{children}</code>
      </pre>
    </div>
  );
};
