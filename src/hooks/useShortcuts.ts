import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { useAI } from "./useAI";

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function useShortcuts() {
  const { screenshotAndAsk } = useAI();
  const callbackRef = useRef(screenshotAndAsk);
  callbackRef.current = screenshotAndAsk;

  useEffect(() => {
    if (!isTauri()) return;

    let active = true;
    const unlisten = listen("trigger-screenshot", () => {
      if (active) {
        callbackRef.current();
      }
    });

    return () => {
      active = false;
      unlisten.then((fn) => fn());
    };
  }, []); // empty deps -- single listener for the lifetime of the app
}
