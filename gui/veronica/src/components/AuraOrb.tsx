import React from "react";
import { AssistantState } from "../types/protocol";
import { Mic, Volume2, Sparkles } from "lucide-react";

interface Props {
  state: AssistantState;
}

export const AuraOrb: React.FC<Props> = ({ state }) => {
  const getOrbStyles = () => {
    switch (state) {
      case "LISTENING":
        return {
          aura: "bg-cyan-500/30 scale-125 blur-2xl animate-ping",
          core: "bg-gradient-to-tr from-cyan-600 to-blue-500 shadow-cyan-500/50",
          icon: <Mic className="w-8 h-8 text-cyan-100 animate-pulse" />,
          label: "Listening to you...",
          badgeBg: "bg-cyan-950/60 text-cyan-300 border-cyan-800/50",
          waveColor: "bg-cyan-400",
        };
      case "SPEAKING":
        return {
          aura: "bg-indigo-500/35 scale-135 blur-3xl animate-pulse",
          core: "bg-gradient-to-tr from-violet-600 via-indigo-500 to-emerald-400 shadow-indigo-500/60",
          icon: <Volume2 className="w-8 h-8 text-white animate-bounce" />,
          label: "Veronica speaking...",
          badgeBg: "bg-indigo-950/60 text-indigo-300 border-indigo-800/50",
          waveColor: "bg-indigo-400",
        };
      case "IDLE":
      default:
        return {
          aura: "bg-indigo-600/15 scale-100 blur-xl animate-pulse",
          core: "bg-gradient-to-tr from-zinc-800 via-zinc-900 to-indigo-950 shadow-indigo-900/30",
          icon: <Sparkles className="w-7 h-7 text-indigo-400/80" />,
          label: "Veronica is ready",
          badgeBg: "bg-zinc-900/60 text-zinc-400 border-zinc-800/40",
          waveColor: "bg-zinc-600",
        };
    }
  };

  const current = getOrbStyles();

  return (
    <div className="flex flex-col items-center justify-center my-4 select-none">
      {/* Visualizer Container */}
      <div className="relative flex items-center justify-center w-40 h-40">
        {/* Outer Glow Halo */}
        <div
          className={`absolute w-32 h-32 rounded-full transition-all duration-700 ease-out ${current.aura}`}
        />

        {/* Middle Pulse Ring */}
        <div
          className={`absolute w-28 h-28 rounded-full border border-indigo-500/20 transition-all duration-500 ${
            state !== "IDLE" ? "scale-110 opacity-100" : "scale-100 opacity-30"
          }`}
        />

        {/* Central Core Orb */}
        <div
          className={`relative w-20 h-20 rounded-full border border-white/10 shadow-2xl flex items-center justify-center transition-all duration-500 transform hover:scale-105 ${current.core}`}
        >
          {current.icon}
        </div>
      </div>

      {/* State Badge & Micro-Waveform */}
      <div className="flex flex-col items-center gap-2 mt-2">
        <div
          className={`flex items-center gap-2 px-3 py-1 rounded-full border text-xs font-mono tracking-wide backdrop-blur-md transition-all duration-300 ${current.badgeBg}`}
        >
          <span
            className={`w-2 h-2 rounded-full ${
              state === "LISTENING"
                ? "bg-cyan-400 animate-ping"
                : state === "SPEAKING"
                ? "bg-indigo-400 animate-pulse"
                : "bg-zinc-500"
            }`}
          />
          {current.label}
        </div>

        {/* Active Audio Waveform Bars */}
        {state !== "IDLE" && (
          <div className="flex items-center gap-1 h-3 mt-1">
            {[0.4, 0.8, 0.5, 0.9, 0.6, 1.0, 0.7, 0.4].map((scale, i) => (
              <span
                key={i}
                className={`w-1 rounded-full animate-pulse ${current.waveColor}`}
                style={{
                  height: `${Math.round(scale * 12)}px`,
                  animationDelay: `${i * 90}ms`,
                  animationDuration: "500ms",
                }}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
