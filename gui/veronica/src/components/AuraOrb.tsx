import React, { useEffect, useRef } from "react";
import { resolvePreset, MODE_DRAWS } from "thinking-orbs";
import { AssistantState } from "../types/protocol";

interface Props {
  state: AssistantState;
}

export const AuraOrb: React.FC<Props> = ({ state }) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  const isSpeaking = state === "SPEAKING";
  const isListening = state === "LISTENING";

  // Calibrated target speeds (scaled to native 2.34 engine speed):
  // - IDLE: Calm, smooth, relaxed slow drift (0.65)
  // - LISTENING / INPUT: Controlled moderate pace (1.35) - never rushes or spins too fast
  // - SPEAKING: Slightly faster active response motion (2.80) while AI responds
  const targetSpeed = isSpeaking ? 2.80 : isListening ? 1.35 : 0.65;

  const targetSpeedRef = useRef(targetSpeed);
  targetSpeedRef.current = targetSpeed;

  const currentSpeedRef = useRef(0.65);
  const phaseRef = useRef(0);
  const lastTimeRef = useRef(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const size = 64;
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    canvas.width = Math.round(size * dpr);
    canvas.height = Math.round(size * dpr);
    canvas.style.width = `${size}px`;
    canvas.style.height = `${size}px`;

    const preset = resolvePreset("composing", size);
    const drawFn = MODE_DRAWS[preset.mode];

    let animId: number;
    lastTimeRef.current = performance.now();

    const render = (now: number) => {
      animId = requestAnimationFrame(render);

      // Delta time capped to 50ms to prevent any time-gap leaps
      const dt = Math.min((now - lastTimeRef.current) / 1000, 0.05);
      lastTimeRef.current = now;

      // Smoothly ease current speed toward target speed
      const diff = targetSpeedRef.current - currentSpeedRef.current;
      currentSpeedRef.current += diff * 0.08;

      // CONTINUOUS PHASE ACCUMULATION:
      // Prevents the library's bug where multiplying absolute timestamp
      // by a changing speed causes wild, supersonic phase jumps.
      phaseRef.current += dt * currentSpeedRef.current;

      // Clear & Draw
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, size, size);
      drawFn(ctx, size, phaseRef.current, true, preset.opts);
    };

    animId = requestAnimationFrame(render);

    return () => {
      cancelAnimationFrame(animId);
    };
  }, []);

  return (
    <div className="flex flex-col items-center justify-center select-none py-4 w-full">
      {/* Free-Floating Composing Ribbon with Continuous Phase Easing */}
      <div className="relative flex items-center justify-center my-2">
        <div
          className={`transition-all duration-700 ease-out ${
            state === "IDLE" ? "scale-[1.48] opacity-90" : "scale-[1.65] opacity-100"
          }`}
        >
          <canvas ref={canvasRef} />
        </div>
      </div>

      {/* Clean Typography Status Badge */}
      <div className="flex items-center gap-2 mt-3 text-xs font-mono">
        <div
          className={`px-3.5 py-1 rounded-full border backdrop-blur-md shadow-sm transition-all duration-300 ${
            isListening
              ? "bg-zinc-900 text-cyan-300 border-cyan-800/60"
              : isSpeaking
              ? "bg-zinc-900 text-indigo-300 border-indigo-800/60"
              : "bg-zinc-900 text-zinc-400 border-zinc-800/60"
          }`}
        >
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
