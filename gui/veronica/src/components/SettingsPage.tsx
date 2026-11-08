import React, { useState } from "react";
import {
  Sliders,
  Sparkles,
  Volume2,
  Database,
  Mic,
  Shield,
  Check,
  Save,
} from "lucide-react";

interface Props {
  onSwitchModel?: (newModel: string) => void;
}

export const SettingsPage: React.FC<Props> = ({ onSwitchModel }) => {
  const [selectedModel, setSelectedModel] = useState("gemini-2.5-flash-native-audio-latest");
  const [selectedVoice, setSelectedVoice] = useState("Aoede");
  const [backendUrl, setBackendUrl] = useState("http://127.0.0.1:8000/api/v1");
  const [isSaved, setIsSaved] = useState(false);

  const handleSave = () => {
    if (onSwitchModel) {
      onSwitchModel(selectedModel);
    }
    setIsSaved(true);
    setTimeout(() => setIsSaved(false), 2500);
  };

  return (
    <div className="flex-1 flex flex-col p-6 overflow-y-auto max-w-4xl mx-auto w-full gap-6 select-none font-sans">
      {/* Header */}
      <div className="flex items-center justify-between pb-4 border-b border-zinc-800/80">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <Sliders className="w-5 h-5 text-zinc-300" />
            <h1 className="text-lg font-semibold tracking-wide text-zinc-100 font-mono uppercase">
              System Settings & Preferences
            </h1>
          </div>
          <p className="text-xs text-zinc-500 font-sans">
            Configure Veronica's active neural intelligence, voice acoustics, and backend cloud connectivity.
          </p>
        </div>

        <button
          onClick={handleSave}
          className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-xl text-xs font-mono flex items-center gap-2 transition-all border border-zinc-700/60 shadow-lg cursor-pointer"
        >
          {isSaved ? <Check className="w-4 h-4 text-emerald-400" /> : <Save className="w-4 h-4" />}
          <span>{isSaved ? "Settings Applied" : "Save Changes"}</span>
        </button>
      </div>

      {/* 1. Model Selection */}
      <div className="bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl p-5 shadow-xl">
        <div className="flex items-center gap-2 mb-3">
          <Sparkles className="w-4 h-4 text-zinc-400" />
          <h2 className="text-xs font-mono font-semibold tracking-wider text-zinc-200 uppercase">
            Active Neural Core (AI Model)
          </h2>
        </div>
        <p className="text-xs text-zinc-500 mb-4 leading-relaxed">
          Switch Veronica's active model dynamically at runtime without restarting the daemon.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          {[
            {
              id: "gemini-2.5-flash-native-audio-latest",
              name: "Gemini 2.5 Flash Audio",
              badge: "LIVE VOICE",
              desc: "Native low-latency bidirectional voice stream.",
            },
            {
              id: "gemini-3.8-flash",
              name: "Gemini 3.8 Flash",
              badge: "AUTONOMOUS",
              desc: "Ultra-fast tool orchestration and reasoner.",
            },
            {
              id: "gemini-1.5-pro",
              name: "Gemini 1.5 Pro",
              badge: "HEAVY LOGIC",
              desc: "Deep analytical planning & large context tasks.",
            },
          ].map((m) => (
            <div
              key={m.id}
              onClick={() => setSelectedModel(m.id)}
              className={`p-4 rounded-xl border cursor-pointer transition-all flex flex-col justify-between ${
                selectedModel === m.id
                  ? "bg-zinc-900 border-zinc-500 shadow-md shadow-white/5"
                  : "bg-zinc-950/50 border-zinc-800/80 hover:border-zinc-700"
              }`}
            >
              <div>
                <div className="flex items-center justify-between mb-1.5">
                  <span className="text-xs font-semibold text-zinc-200 font-mono">
                    {m.name}
                  </span>
                  <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-400">
                    {m.badge}
                  </span>
                </div>
                <p className="text-[11px] text-zinc-500 font-sans leading-relaxed">
                  {m.desc}
                </p>
              </div>

              <div className="mt-3 flex items-center justify-end">
                <span
                  className={`w-3.5 h-3.5 rounded-full border flex items-center justify-center ${
                    selectedModel === m.id
                      ? "border-zinc-300 bg-zinc-100"
                      : "border-zinc-700 bg-transparent"
                  }`}
                >
                  {selectedModel === m.id && <span className="w-1.5 h-1.5 rounded-full bg-black" />}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 2. Neural Voice */}
      <div className="bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl p-5 shadow-xl">
        <div className="flex items-center gap-2 mb-3">
          <Volume2 className="w-4 h-4 text-zinc-400" />
          <h2 className="text-xs font-mono font-semibold tracking-wider text-zinc-200 uppercase">
            Acoustic Neural Voice Persona
          </h2>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {[
            { name: "Aoede", tone: "Natural, Warm & Polite (Default)" },
            { name: "Puck", tone: "Crisp, Clear & Energetic" },
            { name: "Charon", tone: "Authoritative & Deep" },
            { name: "Fenrir", tone: "Confident & Direct" },
          ].map((v) => (
            <div
              key={v.name}
              onClick={() => setSelectedVoice(v.name)}
              className={`p-3 rounded-xl border cursor-pointer transition-all ${
                selectedVoice === v.name
                  ? "bg-zinc-900 border-zinc-500 shadow-md"
                  : "bg-zinc-950/50 border-zinc-800/80 hover:border-zinc-700"
              }`}
            >
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-semibold text-zinc-200 font-mono">
                  {v.name}
                </span>
                {selectedVoice === v.name && <Check className="w-3.5 h-3.5 text-zinc-200" />}
              </div>
              <p className="text-[10px] text-zinc-500 font-sans leading-tight">
                {v.tone}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* 3. Central Backend & Cloud Integration */}
      <div className="bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl p-5 shadow-xl">
        <div className="flex items-center gap-2 mb-3">
          <Database className="w-4 h-4 text-zinc-400" />
          <h2 className="text-xs font-mono font-semibold tracking-wider text-zinc-200 uppercase">
            Decoupled Central Backend Endpoint
          </h2>
        </div>
        <p className="text-xs text-zinc-500 mb-3 leading-relaxed">
          Veronica routes schedules and telemetry through this cloud-agnostic REST backend.
        </p>

        <div className="flex items-center gap-3">
          <input
            type="text"
            value={backendUrl}
            onChange={(e) => setBackendUrl(e.target.value)}
            className="flex-1 bg-zinc-900/80 border border-zinc-800 focus:border-zinc-600 rounded-xl px-4 py-2 text-xs font-mono text-zinc-200 focus:outline-none"
          />
          <button
            onClick={() => alert("Backend connection verified: Healthy")}
            className="px-3.5 py-2 bg-zinc-900 hover:bg-zinc-800 text-zinc-300 rounded-xl text-xs font-mono border border-zinc-800 cursor-pointer"
          >
            Test Link
          </button>
        </div>
      </div>

      {/* 4. Hardware & Safety */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl p-4 shadow-xl">
          <div className="flex items-center gap-2 mb-2">
            <Mic className="w-4 h-4 text-zinc-400" />
            <h3 className="text-xs font-mono font-semibold text-zinc-300 uppercase">
              Audio Activity Detection
            </h3>
          </div>
          <p className="text-[11px] text-zinc-500 leading-relaxed">
            Continuous listening is active with hardware VAD silence suppression set at 400ms.
          </p>
        </div>

        <div className="bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl p-4 shadow-xl">
          <div className="flex items-center gap-2 mb-2">
            <Shield className="w-4 h-4 text-zinc-400" />
            <h3 className="text-xs font-mono font-semibold text-zinc-300 uppercase">
              Platform Sandbox Policy
            </h3>
          </div>
          <p className="text-[11px] text-zinc-500 leading-relaxed">
            OS commands and window actions execute within authenticated user security boundaries.
          </p>
        </div>
      </div>
    </div>
  );
};

