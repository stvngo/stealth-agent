import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Mic, Square, Plus, FileText } from "lucide-react";
import { useAppStore } from "../stores/appStore";
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
          <span
            className="w-2 h-2 rounded-full"
            style={{
              background: isRecording ? "var(--error)" : "var(--text-tertiary)",
              animation: isRecording ? "pulse-dot 1.4s ease-in-out infinite" : "none",
            }}
          />
          <span
            className="text-[11px] font-medium"
            style={{ color: "var(--text-secondary)" }}
          >
            {isRecording ? "Recording" : "Idle"}
          </span>
        </div>
        <button
          onClick={toggleRecording}
          className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-[6px] text-[11px] font-medium transition-colors"
          style={{
            background: isRecording ? "var(--error)" : "var(--accent)",
            color: "white",
          }}
        >
          {isRecording ? (
            <>
              <Square size={10} fill="currentColor" strokeWidth={0} />
              Stop
            </>
          ) : (
            <>
              <Mic size={11} strokeWidth={2.2} />
              Record
            </>
          )}
        </button>
      </div>

      {/* Transcript entries */}
      <div className="flex-1 overflow-y-auto px-3 py-3 space-y-2">
        {transcript.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full gap-2 text-center">
            <div
              className="w-9 h-9 rounded-full flex items-center justify-center"
              style={{ background: "var(--bg-hover)" }}
            >
              <FileText size={15} style={{ color: "var(--text-tertiary)" }} />
            </div>
            <p
              className="text-[12px] font-medium"
              style={{ color: "var(--text-secondary)" }}
            >
              No transcript yet
            </p>
            <p
              className="text-[11px] leading-relaxed max-w-[260px]"
              style={{ color: "var(--text-tertiary)" }}
            >
              Hit Record to capture audio, or add notes manually below.
            </p>
          </div>
        )}

        {transcript.map((entry, i) => (
          <div key={i} className="text-[12px] leading-relaxed">
            <span
              className="font-semibold mr-1.5"
              style={{
                color:
                  entry.speaker === "Me"
                    ? "var(--accent-hover)"
                    : "var(--success)",
              }}
            >
              {entry.speaker}
            </span>
            <span style={{ color: "var(--text-primary)" }}>{entry.text}</span>
          </div>
        ))}
      </div>

      {/* Manual entry */}
      <div
        className="p-2 space-y-2"
        style={{ borderTop: "1px solid var(--border)" }}
      >
        <div
          className="inline-flex p-0.5 rounded-[7px]"
          style={{ background: "var(--bg-input)", border: "1px solid var(--border)" }}
        >
          <button
            onClick={() => setManualSpeaker("interviewer")}
            className="text-[11px] px-2.5 py-1 rounded-[5px] font-medium transition-colors"
            style={{
              background:
                manualSpeaker === "interviewer"
                  ? "var(--accent)"
                  : "transparent",
              color:
                manualSpeaker === "interviewer"
                  ? "white"
                  : "var(--text-secondary)",
            }}
          >
            Interviewer
          </button>
          <button
            onClick={() => setManualSpeaker("me")}
            className="text-[11px] px-2.5 py-1 rounded-[5px] font-medium transition-colors"
            style={{
              background:
                manualSpeaker === "me" ? "var(--accent)" : "transparent",
              color:
                manualSpeaker === "me" ? "white" : "var(--text-secondary)",
            }}
          >
            Me
          </button>
        </div>
        <div className="flex gap-2">
          <input
            value={manualText}
            onChange={(e) => setManualText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && addEntry()}
            placeholder="Add transcript note…"
            className="flex-1 rounded-[8px] px-2.5 py-1.5 text-[12px] outline-none placeholder:opacity-40"
            style={{
              background: "var(--bg-input)",
              color: "var(--text-primary)",
              border: "1px solid var(--border)",
            }}
          />
          <button
            onClick={addEntry}
            className="shrink-0 w-8 h-8 rounded-[8px] flex items-center justify-center transition-opacity hover:opacity-90"
            style={{ background: "var(--accent)", color: "white" }}
            title="Add"
          >
            <Plus size={14} strokeWidth={2.5} />
          </button>
        </div>
      </div>
    </div>
  );
}
