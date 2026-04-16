import { useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
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
      <div className="flex-1 overflow-y-auto px-3 py-2 space-y-3">
        {messages.length === 0 && !streamingContent && (
          <div className="flex flex-col items-center justify-center h-full text-center opacity-50">
            <div className="text-2xl mb-2">👻</div>
            <p className="text-xs" style={{ color: "var(--text-secondary)" }}>
              Invisible mode active
            </p>
            <p
              className="text-xs mt-1"
              style={{ color: "var(--text-secondary)" }}
            >
              <kbd className="px-1 py-0.5 rounded text-[10px]" style={{ background: "var(--bg-input)" }}>⌘ `</kbd>{" "}
              Screenshot + AI
            </p>
            <p
              className="text-xs mt-1"
              style={{ color: "var(--text-secondary)" }}
            >
              <kbd className="px-1 py-0.5 rounded text-[10px]" style={{ background: "var(--bg-input)" }}>⌘ B</kbd>{" "}
              Toggle visibility
            </p>
          </div>
        )}

        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`${msg.role === "user" ? "flex justify-end" : ""}`}
          >
            <div
              className={`max-w-full rounded-lg px-3 py-2 text-[13px] leading-relaxed ${
                msg.role === "user"
                  ? "ml-8"
                  : "mr-2"
              }`}
              style={{
                background:
                  msg.role === "user" ? "var(--accent)" : "var(--bg-secondary)",
                color: "var(--text-primary)",
              }}
            >
              {msg.role === "user" ? (
                <p>
                  {msg.hasScreenshot && (
                    <span className="text-[10px] opacity-70 mr-1">📸</span>
                  )}
                  {msg.content}
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
              className="max-w-full rounded-lg px-3 py-2 mr-2 text-[13px] leading-relaxed"
              style={{ background: "var(--bg-secondary)" }}
            >
              <div className="markdown-content">
                <ReactMarkdown>{streamingContent}</ReactMarkdown>
              </div>
              <span className="inline-block w-1.5 h-4 ml-0.5 animate-pulse" style={{ background: "var(--accent)" }} />
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div
        className="p-2 border-t flex gap-2 items-end"
        style={{ borderColor: "var(--border)" }}
      >
        <button
          onClick={() => screenshotAndAsk()}
          disabled={isStreaming}
          className="shrink-0 w-8 h-8 rounded-md flex items-center justify-center text-sm transition-colors hover:opacity-80 disabled:opacity-30"
          style={{ background: "var(--bg-input)" }}
          title="Screenshot + Ask AI (⌘ `)"
        >
          📸
        </button>
        <textarea
          ref={inputRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Ask anything..."
          rows={1}
          className="flex-1 resize-none rounded-md px-3 py-1.5 text-[13px] outline-none placeholder:opacity-40"
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
          className="shrink-0 w-8 h-8 rounded-md flex items-center justify-center text-sm transition-colors hover:opacity-80 disabled:opacity-30"
          style={{ background: "var(--accent)" }}
        >
          ↑
        </button>
      </div>
    </div>
  );
}
