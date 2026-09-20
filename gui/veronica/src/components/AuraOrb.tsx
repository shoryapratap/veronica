import React, { useState, useEffect } from "react";
import { ThinkingOrb } from "thinking-orbs";
import { AssistantState } from "../types/protocol";
import { Mic, Volume2, Sparkles } from "lucide-react";

interface Props {
  state: AssistantState;
}

export const AuraOrb: React.FC<Props> = ({ state }) => {
  const isSpeaking = state === "SPEAKING";
  const isListening = state === "LISTENING";

  // Target speed based on state:
  // - IDLE: Very slow, calm, gentle drift (0.22) so it never looks dead or abruptly frozen
  // - LISTENING: Active conversational listening speed (1.15)
  // - SPEAKING: Energetic undulating speaking speed (1.45)
  const targetSpeed = isSpeaking ? 1.45 : isListening ? 1.15 : 0.22;

  const [currentSpeed, setCurrentSpeed] = useState(0.22);

  // Smoothly interpolate (lerp) speed to prevent any jarring start/stop jumps
  useEffect(() => {
    let animId: number;

    const smoothTransition = () => {
      setCurrentSpeed((prev) => {
        const diff = targetSpeed - prev;
        if (Math.abs(diff) < 0.005) return targetSpeed;
        return prev + diff * 0.08; // Smooth natural easing
      });

      animId = requestAnimationFrame(smoothTransition);
    };

    animId = requestAnimationFrame(smoothTransition);
    return () => cancelAnimationFrame(animId);
  }, [targetSpeed]);

  return (
    <div className="flex flex-col items-center justify-center select-none py-4 w-full">
      {/* Free-Floating Orb with Smooth Scale & Momentum */}
      <div className="relative flex items-center justify-center my-2">
        <div
          className={`transition-all duration-700 ease-out ${
            state === "IDLE" ? "scale-135 opacity-85" : "scale-155 opacity-100"
          }`}
        >
          <ThinkingOrb
            state="composing"
            size={64}
            theme="dark"
            speed={currentSpeed}
            paused={false}
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
