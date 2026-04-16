import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useCallback, useRef } from "react";
import { useAppStore } from "../stores/appStore";

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function useAI() {
  const {
    addMessage,
    updateStreamingContent,
    finalizeStreaming,
    setIsStreaming,
    config,
    isStreaming,
  } = useAppStore();

  // Stable refs to avoid re-registering listeners
  const updateRef = useRef(updateStreamingContent);
  updateRef.current = updateStreamingContent;
  const finalizeRef = useRef(finalizeStreaming);
  finalizeRef.current = finalizeStreaming;

  useEffect(() => {
    if (!isTauri()) return;

    let active = true;

    const unlisteners = [
      listen<string>("ai-token", (event) => {
        if (active) updateRef.current(event.payload);
      }),
      listen<string>("ai-done", (event) => {
        if (active) finalizeRef.current(event.payload);
      }),
      listen<string>("ai-error", (event) => {
        if (active) finalizeRef.current(`**Error:** ${event.payload}`);
      }),
    ];

    return () => {
      active = false;
      unlisteners.forEach((p) => p.then((fn) => fn()));
    };
  }, []); // empty deps -- single set of listeners for app lifetime

  const sendMessage = useCallback(
    async (text: string, includeScreenshot: boolean = false) => {
      if (!isTauri() || isStreaming || !text.trim()) return;

      addMessage({
        id: crypto.randomUUID(),
        role: "user",
        content: text,
        timestamp: Date.now(),
        hasScreenshot: includeScreenshot,
      });

      setIsStreaming(true);

      try {
        await invoke("set_config", {
          apiKey: config.apiKey || null,
          model: config.model || null,
          baseUrl: config.baseUrl || null,
          resume: config.resumeText || null,
        });

        await invoke("send_message", {
          message: text,
          includeScreenshot,
        });
      } catch (e) {
        finalizeStreaming(`**Error:** ${e}`);
      }
    },
    [isStreaming, config, addMessage, setIsStreaming, finalizeStreaming],
  );

  const screenshotAndAsk = useCallback(
    async (question?: string) => {
      const q =
        question ||
        "Look at the screen and help me with what you see. What is the question being asked, and what is the best answer?";
      await sendMessage(q, true);
    },
    [sendMessage],
  );

  return { sendMessage, screenshotAndAsk, isStreaming };
}
