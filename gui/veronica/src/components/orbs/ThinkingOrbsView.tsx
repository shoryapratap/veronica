import React, { useState } from "react";
import { ThinkingOrb, OrbState, OrbSize } from "thinking-orbs";
import { AssistantState } from "../../types/protocol";

interface Props {
  assistantState: AssistantState;
  size?: OrbSize;
}

const ALL_STATES: { id: OrbState; label: string; mode: string }[] = [
  { id: "listening", label: "Listening", mode: "wave" },
  { id: "solving", label: "Solving", mode: "rubik" },
  { id: "working", label: "Working", mode: "orbits" },
  { id: "searching", label: "Searching", mode: "globe" },
  { id: "composing", label: "Composing", mode: "ribbon" },
  { id: "shaping", label: "Shaping", mode: "morph" },
  { id: "breathing", label: "Breathing", mode: "ring" },
  { id: "connecting", label: "Connecting", mode: "web" },
  { id: "weaving", label: "Weaving", mode: "braid" },
];

export const ThinkingOrbsView: React.FC<Props> = ({
  assistantState,
  size = 64,
}) => {
  const [autoSync, setAutoSync] = useState(true);
  const [manualState, setManualState] = useState<OrbState>("listening");

  // Map Veronica's live assistant state to a ThinkingOrb state
  const getAutoState = (): OrbState => {
    switch (assistantState) {
      case "LISTENING":
        return "listening";
      case "SPEAKING":
        return "solving";
      case "IDLE":
      default:
        return "breathing";
    }
  };

  const activeState: OrbState = autoSync ? getAutoState() : manualState;

  return (
    <div className="flex flex-col items-center justify-center select-none w-full max-w-md mx-auto">
      {/* Centered Thinking Orb Display with scaling wrapper */}
      <div className="relative flex items-center justify-center my-3 p-6 rounded-3xl bg-zinc-900/50 border border-zinc-800/80 shadow-2xl">
        <div className="transform scale-150 transition-transform duration-300">
          <ThinkingOrb
            state={activeState}
            size={size}
            theme="dark"
            speed={1.1}
          />
        </div>
      </div>

      {/* State Caption */}
      <div className="text-xs font-mono text-zinc-300 mb-2.5 flex items-center gap-2">
        <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
        <span className="capitalize font-medium">{activeState}</span>
        <span className="text-zinc-500 text-[10px]">
          ({ALL_STATES.find((s) => s.id === activeState)?.mode})
        </span>
      </div>

      {/* Control Bar: Auto-Sync vs Manual Mode */}
      <div className="flex items-center gap-2 mb-2">
        <button
          onClick={() => setAutoSync(!autoSync)}
          className={`px-3 py-0.5 rounded-full text-[10px] font-mono border transition-all ${
            autoSync
              ? "bg-cyan-950/70 border-cyan-700/60 text-cyan-300 shadow-sm"
              : "bg-zinc-900 border-zinc-800 text-zinc-400 hover:text-zinc-200"
          }`}
        >
          {autoSync ? "● Auto-Sync with Voice" : "○ Manual Mode"}
        </button>
      </div>

      {/* Chips list to click and see ALL 9 states */}
      <div className="flex flex-wrap items-center justify-center gap-1.5 px-4 max-w-sm">
        {ALL_STATES.map((st) => {
          const isCurrent = activeState === st.id;
          return (
            <button
              key={st.id}
              onClick={() => {
                setAutoSync(false);
                setManualState(st.id);
              }}
              className={`px-2.5 py-0.5 rounded-md text-[10px] font-mono transition-all ${
                isCurrent
                  ? "bg-cyan-600 text-white font-semibold shadow-sm shadow-cyan-600/30"
                  : "bg-zinc-900 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 border border-zinc-800"
              }`}
            >
              {st.label}
            </button>
          );
        })}
      </div>
    </div>
  );
};
