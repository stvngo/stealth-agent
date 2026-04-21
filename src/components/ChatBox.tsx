import { useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import { Camera, ArrowUp, Command, Image as ImageIcon, Sparkles } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { useAI } from "../hooks/useAI";

export function ChatBox() {
  const [input, setInput] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const { messages, streamingContent, isStreaming } = useAppStore();
  const { sendMessage, screenshotAndAsk } = useAI();

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingContent]);

  const handleSend = () => {
    if (!input.trim() || isStreaming) return;
    sendMessage(input.trim());
    setInput("");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Messages area */}
      <div className="flex-1 overflow-y-auto px-3 py-3 space-y-3">
        {messages.length === 0 && !streamingContent && (
          <div className="flex flex-col items-center justify-center h-full text-center gap-3">
            <div
              className="w-9 h-9 rounded-full flex items-center justify-center"
              style={{ background: "var(--accent-muted)" }}
            >
              <Sparkles size={16} style={{ color: "var(--accent-hover)" }} />
            </div>
            <p
              className="text-[12px] font-medium"
              style={{ color: "var(--text-secondary)" }}
            >
              Ask anything. Nobody can see me.
            </p>
            <div
              className="flex items-center gap-1.5 text-[11px]"
              style={{ color: "var(--text-tertiary)" }}
            >
              <Kbd>
                <Command size={10} strokeWidth={2.4} />
              </Kbd>
              <span>+</span>
              <Kbd>`</Kbd>
              <span>Screenshot + Ask</span>
            </div>
            <div
              className="flex items-center gap-1.5 text-[11px]"
              style={{ color: "var(--text-tertiary)" }}
            >
              <Kbd>
                <Command size={10} strokeWidth={2.4} />
              </Kbd>
              <span>+</span>
              <Kbd>B</Kbd>
              <span>Toggle visibility</span>
            </div>
          </div>
        )}

        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`${msg.role === "user" ? "flex justify-end" : ""}`}
          >
            <div
              className={`max-w-full rounded-[10px] px-3 py-2 text-[13px] leading-relaxed ${
                msg.role === "user" ? "ml-10" : "mr-2"
              }`}
              style={{
                background:
                  msg.role === "user" ? "var(--accent)" : "var(--bg-secondary)",
                color: msg.role === "user" ? "white" : "var(--text-primary)",
                border:
                  msg.role === "user"
                    ? "none"
                    : "1px solid var(--border)",
              }}
            >
              {msg.role === "user" ? (
                <p className="flex items-center gap-1.5">
                  {msg.hasScreenshot && (
                    <ImageIcon size={12} className="opacity-80 shrink-0" />
                  )}
                  <span>{msg.content}</span>
                </p>
              ) : (
                <div className="markdown-content">
                  <ReactMarkdown>{msg.content}</ReactMarkdown>
                </div>
              )}
            </div>
          </div>
        ))}

        {streamingContent && (
          <div>
            <div
              className="max-w-full rounded-[10px] px-3 py-2 mr-2 text-[13px] leading-relaxed"
              style={{
                background: "var(--bg-secondary)",
                border: "1px solid var(--border)",
              }}
            >
              <div className="markdown-content">
                <ReactMarkdown>{streamingContent}</ReactMarkdown>
              </div>
              <span
                className="inline-block w-[6px] h-[14px] ml-0.5 align-middle animate-pulse rounded-[1px]"
                style={{ background: "var(--accent-hover)" }}
              />
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div
        className="p-2 flex gap-2 items-end"
        style={{ borderTop: "1px solid var(--border)" }}
      >
        <button
          onClick={() => screenshotAndAsk()}
          disabled={isStreaming}
          className="shrink-0 w-8 h-8 rounded-[8px] flex items-center justify-center transition-colors hover:opacity-80 disabled:opacity-30"
          style={{
            background: "var(--bg-input)",
            border: "1px solid var(--border)",
            color: "var(--text-secondary)",
          }}
          title="Screenshot + Ask AI (⌘ `)"
        >
          <Camera size={14} strokeWidth={2} />
        </button>
        <textarea
          ref={inputRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Ask anything…"
          rows={1}
          className="flex-1 resize-none rounded-[8px] px-3 py-1.5 text-[13px] outline-none placeholder:opacity-40"
          style={{
            background: "var(--bg-input)",
            color: "var(--text-primary)",
            border: "1px solid var(--border)",
            maxHeight: "80px",
          }}
        />
        <button
          onClick={handleSend}
          disabled={isStreaming || !input.trim()}
          className="shrink-0 w-8 h-8 rounded-[8px] flex items-center justify-center transition-opacity hover:opacity-90 disabled:opacity-25"
          style={{ background: "var(--accent)", color: "white" }}
          title="Send"
        >
          <ArrowUp size={14} strokeWidth={2.5} />
        </button>
      </div>
    </div>
  );
}

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd
      className="inline-flex items-center justify-center px-1.5 py-0.5 rounded-[5px] font-medium text-[10px]"
      style={{
        background: "var(--bg-input)",
        border: "1px solid var(--border)",
        color: "var(--text-secondary)",
        minWidth: "18px",
      }}
    >
      {children}
    </kbd>
  );
}
