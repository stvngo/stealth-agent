import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState, useCallback } from "react";
import { useAppStore } from "../stores/appStore";
import type { TranscriptEntry } from "../stores/appStore";

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function useAudio() {
  const [isRecording, setIsRecording] = useState(false);
  const { addTranscriptEntry } = useAppStore();

  useEffect(() => {
    if (!isTauri()) return;

    const unlisten = listen<TranscriptEntry>("transcript-entry", (event) => {
      addTranscriptEntry(event.payload);
    });

    invoke<{ is_recording: boolean }>("get_recording_status")
      .then((status) => {
        setIsRecording(status.is_recording);
      })
      .catch(() => {});

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [addTranscriptEntry]);

  const startRecording = useCallback(async () => {
    if (!isTauri()) return;
    try {
      const result = await invoke<{ is_recording: boolean }>("start_recording");
      setIsRecording(result.is_recording);
    } catch (e) {
      console.error("Failed to start recording:", e);
    }
  }, []);

  const stopRecording = useCallback(async () => {
    if (!isTauri()) return;
    try {
      const result = await invoke<{ is_recording: boolean }>("stop_recording");
      setIsRecording(result.is_recording);
    } catch (e) {
      console.error("Failed to stop recording:", e);
    }
  }, []);

  const toggleRecording = useCallback(async () => {
    if (isRecording) {
      await stopRecording();
    } else {
      await startRecording();
    }
  }, [isRecording, startRecording, stopRecording]);

  return { isRecording, startRecording, stopRecording, toggleRecording };
}
