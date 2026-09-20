import React from "react";
import { ThinkingOrb } from "thinking-orbs";
import { AssistantState } from "../types/protocol";
import { Mic, Volume2, Sparkles } from "lucide-react";

interface Props {
  state: AssistantState;
}

export const AuraOrb: React.FC<Props> = ({ state }) => {
  const isSpeaking = state === "SPEAKING";
  const isListening = state === "LISTENING";

  // Orb only moves when either the user or the AI is talking
  const isMoving = isSpeaking || isListening;
  const speed = isSpeaking ? 1.4 : 1.1;

  return (
    <div className="flex flex-col items-center justify-center select-none py-4 w-full">
      {/* Independent Free-Floating Orb (No background box, No background glow) */}
      <div className="relative flex items-center justify-center my-2">
        <div className="transform scale-150 transition-transform duration-300">
          <ThinkingOrb
            state="composing"
            size={64}
            theme="dark"
            speed={speed}
            paused={!isMoving}
          />
        </div>
      </div>

      {/* Clean Status Badge */}
      <div className="flex items-center gap-2 mt-3 text-xs font-mono">
        <div
          className={`flex items-center gap-2 px-3.5 py-1 rounded-full border backdrop-blur-md shadow-sm transition-all duration-300 ${
            isListening
              ? "bg-zinc-900 text-cyan-300 border-cyan-800/60"
              : isSpeaking
              ? "bg-zinc-900 text-indigo-300 border-indigo-800/60"
              : "bg-zinc-900 text-zinc-400 border-zinc-800/60"
          }`}
        >
          {isListening ? (
            <Mic className="w-3.5 h-3.5 text-cyan-300 animate-pulse" />
          ) : isSpeaking ? (
            <Volume2 className="w-3.5 h-3.5 text-indigo-300 animate-bounce" />
          ) : (
            <Sparkles className="w-3.5 h-3.5 text-zinc-500" />
          )}
          <span>
            {isListening
              ? "Listening to you..."
              : isSpeaking
              ? "Veronica speaking..."
              : "Veronica is ready"}
          </span>
        </div>
      </div>
    </div>
  );
};
