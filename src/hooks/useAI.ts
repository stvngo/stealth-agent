import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useCallback, useRef } from "react";
import { useAppStore } from "../stores/appStore";

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Install Tauri event listeners that bridge backend AI stream events into the
 * Zustand store. MUST be called exactly ONCE at the root of the component
 * tree (currently in App.tsx). Calling it from multiple components -- or
 * embedding it in `useAI` which is consumed from multiple places -- results
 * in duplicate listeners, which manifests as duplicated assistant messages.
 *
 * A module-level guard also prevents double-registration in the extreme edge
 * case of StrictMode double-mount + delayed async unlisten, so even if this
 * is accidentally called twice in a row we won't leak listeners.
 */
let eventsMounted = false;

export function useAIEventBridge() {
  const updateStreamingContent = useAppStore((s) => s.updateStreamingContent);
  const finalizeStreaming = useAppStore((s) => s.finalizeStreaming);

  // Stable refs so the listener callbacks always call the latest store actions
  // without us needing to re-register on every store update.
  const updateRef = useRef(updateStreamingContent);
  updateRef.current = updateStreamingContent;
  const finalizeRef = useRef(finalizeStreaming);
  finalizeRef.current = finalizeStreaming;

  useEffect(() => {
    if (!isTauri()) return;
    if (eventsMounted) {
      // Another instance has already registered listeners. Bail out silently.
      return;
    }
    eventsMounted = true;

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
      eventsMounted = false;
      unlisteners.forEach((p) => p.then((fn) => fn()));
    };
  }, []);
}

/**
 * Pure consumer hook. Safe to call from anywhere in the tree -- registers
 * NO event listeners. Only returns action callbacks and streaming state.
 */
export function useAI() {
  const {
    addMessage,
    finalizeStreaming,
    setIsStreaming,
    config,
    isStreaming,
  } = useAppStore();

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
