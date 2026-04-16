import { create } from "zustand";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
  hasScreenshot?: boolean;
}

export interface TranscriptEntry {
  speaker: "Me" | "Interviewer" | "Unknown";
  text: string;
  timestamp_ms: number;
}

export interface AppConfig {
  apiKey: string;
  model: string;
  baseUrl: string;
  resumeText: string;
}

const STORAGE_KEY = "invisible-agent-config";

function loadConfig(): AppConfig {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return { ...defaultConfig(), ...JSON.parse(stored) };
    }
  } catch {}
  return defaultConfig();
}

function defaultConfig(): AppConfig {
  return {
    apiKey: "",
    model: "gpt-4o",
    baseUrl: "https://api.openai.com/v1",
    resumeText: "",
  };
}

function saveConfig(config: AppConfig) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  } catch {}
}

interface AppState {
  messages: ChatMessage[];
  transcript: TranscriptEntry[];
  isStreaming: boolean;
  streamingContent: string;
  config: AppConfig;
  activeTab: "chat" | "transcript" | "settings";
  isVisible: boolean;

  addMessage: (msg: ChatMessage) => void;
  updateStreamingContent: (token: string) => void;
  finalizeStreaming: (fullText: string) => void;
  clearStreamingContent: () => void;
  setIsStreaming: (v: boolean) => void;
  addTranscriptEntry: (entry: TranscriptEntry) => void;
  setConfig: (config: Partial<AppConfig>) => void;
  setActiveTab: (tab: "chat" | "transcript" | "settings") => void;
  setIsVisible: (v: boolean) => void;
  clearMessages: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  messages: [],
  transcript: [],
  isStreaming: false,
  streamingContent: "",
  config: loadConfig(),
  activeTab: "chat",
  isVisible: true,

  addMessage: (msg) => set((s) => ({ messages: [...s.messages, msg] })),

  updateStreamingContent: (token) =>
    set((s) => ({ streamingContent: s.streamingContent + token })),

  finalizeStreaming: (fullText) => {
    const msg: ChatMessage = {
      id: crypto.randomUUID(),
      role: "assistant",
      content: fullText,
      timestamp: Date.now(),
    };
    set((s) => ({
      messages: [...s.messages, msg],
      streamingContent: "",
      isStreaming: false,
    }));
  },

  clearStreamingContent: () => set({ streamingContent: "" }),
  setIsStreaming: (v) => set({ isStreaming: v }),

  addTranscriptEntry: (entry) =>
    set((s) => ({ transcript: [...s.transcript, entry] })),

  setConfig: (partial) =>
    set((s) => {
      const newConfig = { ...s.config, ...partial };
      saveConfig(newConfig);
      return { config: newConfig };
    }),

  setActiveTab: (tab) => set({ activeTab: tab }),
  setIsVisible: (v) => set({ isVisible: v }),
  clearMessages: () => set({ messages: [], streamingContent: "" }),
}));
