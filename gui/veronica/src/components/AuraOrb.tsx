import React, { useState } from "react";
import { AssistantState } from "../types/protocol";
import { ReactBitsOrb } from "./orbs/ReactBitsOrb";
import { ThinkingOrbsView } from "./orbs/ThinkingOrbsView";
import { Sparkles, Brain, Mic, Volume2 } from "lucide-react";

interface Props {
  state: AssistantState;
}

type OrbMode = "reactbits" | "thinking";

export const AuraOrb: React.FC<Props> = ({ state }) => {
  const [mode, setMode] = useState<OrbMode>(() => {
    return (localStorage.getItem("veronica_orb_mode") as OrbMode) || "reactbits";
  });

  // State for React Bits Hue
  const [reactBitsHue, setReactBitsHue] = useState<number>(288); // 288 is the violet from screenshot!
  const [autoHue, setAutoHue] = useState(true);

  const handleModeChange = (newMode: OrbMode) => {
    setMode(newMode);
    localStorage.setItem("veronica_orb_mode", newMode);
  };

  // Determine dynamic hue if autoHue is enabled
  const getDynamicHue = (): number => {
    if (!autoHue) return reactBitsHue;
    switch (state) {
      case "LISTENING":
        return 190; // Cyan
      case "SPEAKING":
        return 288; // Vibrant Violet
      case "IDLE":
      default:
        return 250; // Deep Indigo
    }
  };

  const activeHue = getDynamicHue();

  return (
    <div className="flex flex-col items-center justify-center select-none py-2 w-full">
      {/* Top Switcher: React Bits (OGL) vs Thinking Orbs */}
      <div className="flex items-center gap-1.5 p-1 bg-zinc-900/90 border border-zinc-800/80 rounded-full mb-3 backdrop-blur-md shadow-lg">
        <button
          onClick={() => handleModeChange("reactbits")}
          className={`flex items-center gap-1.5 px-3.5 py-1 rounded-full text-xs font-mono transition-all ${
            mode === "reactbits"
              ? "bg-indigo-600 text-white font-medium shadow-md shadow-indigo-600/30"
              : "text-zinc-400 hover:text-zinc-200"
          }`}
        >
          <Sparkles className="w-3.5 h-3.5" />
          <span>React Bits (OGL)</span>
        </button>

        <button
          onClick={() => handleModeChange("thinking")}
          className={`flex items-center gap-1.5 px-3.5 py-1 rounded-full text-xs font-mono transition-all ${
            mode === "thinking"
              ? "bg-cyan-600 text-white font-medium shadow-md shadow-cyan-600/30"
              : "text-zinc-400 hover:text-zinc-200"
          }`}
        >
          <Brain className="w-3.5 h-3.5" />
          <span>Thinking Orbs (9 States)</span>
        </button>
      </div>

      {/* Main Orb Display Stage */}
      {mode === "reactbits" ? (
        <div className="flex flex-col items-center">
          {/* OGL Shader Orb */}
          <div className="relative flex items-center justify-center">
            <ReactBitsOrb
              hue={activeHue}
              hoverIntensity={state === "SPEAKING" ? 0.7 : state === "LISTENING" ? 0.5 : 0.35}
              forceHoverState={state !== "IDLE"}
              size={200}
            />
          </div>

          {/* Controls for React Bits: Color Presets & Auto Hue */}
          <div className="flex flex-col items-center gap-1.5 mt-2">
            <div className="flex items-center gap-1.5 text-xs font-mono">
              <button
                onClick={() => setAutoHue(!autoHue)}
                className={`px-2.5 py-0.5 rounded-full text-[10px] border transition-all ${
                  autoHue
                    ? "bg-indigo-950/80 text-indigo-300 border-indigo-700/60 shadow-sm"
                    : "bg-zinc-900 text-zinc-400 border-zinc-800 hover:text-zinc-200"
                }`}
              >
                {autoHue ? "● Auto Color Sync" : "○ Fixed Color"}
              </button>

              <div className="flex items-center gap-1 ml-1">
                {[
                  { label: "Violet", hue: 288, bg: "bg-purple-500" },
                  { label: "Cyan", hue: 190, bg: "bg-cyan-400" },
                  { label: "Emerald", hue: 140, bg: "bg-emerald-400" },
                  { label: "Gold", hue: 45, bg: "bg-amber-400" },
                  { label: "Rose", hue: 330, bg: "bg-pink-500" },
                ].map((c) => (
                  <button
                    key={c.hue}
                    onClick={() => {
                      setAutoHue(false);
                      setReactBitsHue(c.hue);
                    }}
                    title={c.label}
                    className={`w-4 h-4 rounded-full ${c.bg} transition-all ${
                      !autoHue && reactBitsHue === c.hue
                        ? "ring-2 ring-white scale-125"
                        : "opacity-60 hover:opacity-100"
                    }`}
                  />
                ))}
              </div>
            </div>

            {/* State Indicator */}
            <div className="flex items-center gap-2 mt-1 text-[11px] font-mono text-zinc-400">
              {state === "LISTENING" ? (
                <span className="text-cyan-300 flex items-center gap-1">
                  <Mic className="w-3 h-3 animate-pulse" /> Listening to you...
                </span>
              ) : state === "SPEAKING" ? (
                <span className="text-indigo-300 flex items-center gap-1">
                  <Volume2 className="w-3 h-3 animate-bounce" /> Veronica speaking...
                </span>
              ) : (
                <span>Hover or speak to interact</span>
              )}
            </div>
          </div>
        </div>
      ) : (
        <ThinkingOrbsView assistantState={state} size={64} />
      )}
    </div>
  );
};
