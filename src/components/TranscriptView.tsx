import { useAppStore } from "../stores/appStore";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAudio } from "../hooks/useAudio";

export function TranscriptView() {
  const { transcript } = useAppStore();
  const { isRecording, toggleRecording } = useAudio();
  const [manualText, setManualText] = useState("");
  const [manualSpeaker, setManualSpeaker] = useState<"me" | "interviewer">(
    "interviewer",
  );

  const addEntry = async () => {
    if (!manualText.trim()) return;
    try {
      await invoke("add_transcript_entry", {
        speaker: manualSpeaker,
        text: manualText.trim(),
      });
      useAppStore.getState().addTranscriptEntry({
        speaker: manualSpeaker === "me" ? "Me" : "Interviewer",
        text: manualText.trim(),
        timestamp_ms: Date.now(),
      });
      setManualText("");
    } catch (e) {
      console.error("Failed to add transcript entry:", e);
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Recording controls */}
      <div
        className="px-3 py-2 flex items-center justify-between shrink-0"
        style={{ borderBottom: "1px solid var(--border)" }}
      >
        <div className="flex items-center gap-2">
          <div
            className="w-2 h-2 rounded-full"
            style={{
              background: isRecording ? "var(--error)" : "var(--text-secondary)",
              animation: isRecording ? "pulse 1.5s infinite" : "none",
            }}
          />
          <span className="text-[11px]" style={{ color: "var(--text-secondary)" }}>
            {isRecording ? "Recording..." : "Not recording"}
          </span>
        </div>
        <button
          onClick={toggleRecording}
          className="px-2 py-1 rounded text-[11px] font-medium transition-colors"
          style={{
            background: isRecording ? "var(--error)" : "var(--accent)",
            color: "white",
          }}
        >
          {isRecording ? "Stop" : "Record"}
        </button>
      </div>

      {/* Transcript entries */}
      <div className="flex-1 overflow-y-auto px-3 py-2 space-y-2">
        {transcript.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full opacity-50">
            <p
              className="text-xs"
              style={{ color: "var(--text-secondary)" }}
            >
              No transcript yet
            </p>
            <p
              className="text-xs mt-1"
              style={{ color: "var(--text-secondary)" }}
            >
              Hit Record to capture audio, or add notes manually.
            </p>
          </div>
        )}

        {transcript.map((entry, i) => (
          <div key={i} className="text-[12px]">
            <span
              className="font-semibold mr-1"
              style={{
                color:
                  entry.speaker === "Me"
                    ? "var(--accent)"
                    : "var(--success)",
              }}
            >
              {entry.speaker}:
            </span>
            <span style={{ color: "var(--text-primary)" }}>{entry.text}</span>
          </div>
        ))}
      </div>

      {/* Manual entry */}
      <div
        className="p-2 border-t space-y-2"
        style={{ borderColor: "var(--border)" }}
      >
        <div className="flex gap-1">
          <button
            onClick={() => setManualSpeaker("interviewer")}
            className="text-[11px] px-2 py-1 rounded"
            style={{
              background:
                manualSpeaker === "interviewer"
                  ? "var(--accent)"
                  : "var(--bg-input)",
            }}
          >
            Interviewer
          </button>
          <button
            onClick={() => setManualSpeaker("me")}
            className="text-[11px] px-2 py-1 rounded"
            style={{
              background:
                manualSpeaker === "me" ? "var(--accent)" : "var(--bg-input)",
            }}
          >
            Me
          </button>
        </div>
        <div className="flex gap-1">
          <input
            value={manualText}
            onChange={(e) => setManualText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && addEntry()}
            placeholder="Add transcript note..."
            className="flex-1 rounded px-2 py-1 text-[12px] outline-none"
            style={{
              background: "var(--bg-input)",
              color: "var(--text-primary)",
              border: "1px solid var(--border)",
            }}
          />
          <button
            onClick={addEntry}
            className="px-2 py-1 rounded text-[11px]"
            style={{ background: "var(--accent)" }}
          >
            Add
          </button>
        </div>
      </div>
    </div>
  );
}
