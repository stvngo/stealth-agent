import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Check, Save, Command } from "lucide-react";
import { useAppStore } from "../stores/appStore";

export function Settings() {
  const { config, setConfig } = useAppStore();
  const [saved, setSaved] = useState(false);

  const handleSave = async () => {
    try {
      await invoke("set_config", {
        apiKey: config.apiKey || null,
        model: config.model || null,
        baseUrl: config.baseUrl || null,
        resume: config.resumeText || null,
      });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save config:", e);
    }
  };

  return (
    <div className="flex flex-col h-full overflow-y-auto px-3 py-3 space-y-3.5">
      <Field label="API Key">
        <input
          type="password"
          value={config.apiKey}
          onChange={(e) => setConfig({ apiKey: e.target.value })}
          placeholder="sk-…"
          className="w-full rounded-[7px] px-2.5 py-1.5 text-[12px] outline-none placeholder:opacity-40"
          style={inputStyle}
        />
      </Field>

      <Field label="Model">
        <select
          value={config.model}
          onChange={(e) => setConfig({ model: e.target.value })}
          className="w-full rounded-[7px] px-2.5 py-1.5 text-[12px] outline-none"
          style={inputStyle}
        >
          <option value="gpt-4o">GPT-4o</option>
          <option value="gpt-4o-mini">GPT-4o Mini</option>
          <option value="gpt-4.1">GPT-4.1</option>
          <option value="gpt-4.1-mini">GPT-4.1 Mini</option>
          <option value="claude-sonnet-4-20250514">Claude Sonnet 4</option>
          <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet</option>
        </select>
      </Field>

      <Field
        label="API Base URL"
        hint="Change for Anthropic, Azure, or self-hosted models"
      >
        <input
          value={config.baseUrl}
          onChange={(e) => setConfig({ baseUrl: e.target.value })}
          placeholder="https://api.openai.com/v1"
          className="w-full rounded-[7px] px-2.5 py-1.5 text-[12px] outline-none placeholder:opacity-40"
          style={inputStyle}
        />
      </Field>

      <Field label="Resume / Background">
        <textarea
          value={config.resumeText}
          onChange={(e) => setConfig({ resumeText: e.target.value })}
          placeholder="Paste your resume or background info here…"
          rows={5}
          className="w-full rounded-[7px] px-2.5 py-1.5 text-[12px] outline-none resize-none placeholder:opacity-40"
          style={inputStyle}
        />
      </Field>

      <button
        onClick={handleSave}
        className="w-full py-2 rounded-[8px] text-[12px] font-medium flex items-center justify-center gap-1.5 transition-all"
        style={{
          background: saved ? "var(--success)" : "var(--accent)",
          color: "white",
        }}
      >
        {saved ? (
          <>
            <Check size={13} strokeWidth={2.5} /> Saved
          </>
        ) : (
          <>
            <Save size={13} strokeWidth={2.2} /> Save settings
          </>
        )}
      </button>

      <div
        className="pt-3 space-y-1.5"
        style={{ borderTop: "1px solid var(--border)" }}
      >
        <p
          className="text-[10px] font-semibold uppercase tracking-wider mb-2"
          style={{ color: "var(--text-tertiary)" }}
        >
          Shortcuts
        </p>
        <ShortcutRow label="Toggle visibility">
          <ShortcutKey>
            <Command size={10} strokeWidth={2.4} />
          </ShortcutKey>
          <ShortcutKey>B</ShortcutKey>
        </ShortcutRow>
        <ShortcutRow label="Screenshot + AI">
          <ShortcutKey>
            <Command size={10} strokeWidth={2.4} />
          </ShortcutKey>
          <ShortcutKey>`</ShortcutKey>
        </ShortcutRow>
        <ShortcutRow label="Move window">
          <ShortcutKey>
            <Command size={10} strokeWidth={2.4} />
          </ShortcutKey>
          <ShortcutKey>⇧</ShortcutKey>
          <ShortcutKey>↑↓←→</ShortcutKey>
        </ShortcutRow>
      </div>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  background: "var(--bg-input)",
  color: "var(--text-primary)",
  border: "1px solid var(--border)",
};

function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <label
        className="block text-[10px] font-semibold uppercase tracking-wider mb-1.5"
        style={{ color: "var(--text-tertiary)" }}
      >
        {label}
      </label>
      {children}
      {hint && (
        <p
          className="text-[10px] mt-1"
          style={{ color: "var(--text-tertiary)" }}
        >
          {hint}
        </p>
      )}
    </div>
  );
}

function ShortcutRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div
      className="flex justify-between items-center text-[11px]"
      style={{ color: "var(--text-secondary)" }}
    >
      <span>{label}</span>
      <div className="flex items-center gap-1">{children}</div>
    </div>
  );
}

function ShortcutKey({ children }: { children: React.ReactNode }) {
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
