import React from "react";
import { Terminal, Layers } from "lucide-react";

export const MainWindow: React.FC = () => {
  return (
    <div className="flex-1 flex flex-col bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl p-3 overflow-hidden relative shadow-2xl">
      {/* Window Header */}
      <div className="flex items-center justify-between pb-2.5 mb-2.5 border-b border-zinc-800/60 shrink-0 px-1">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-center shadow-md">
            <Layers className="w-4 h-4 text-zinc-300" />
          </div>
          <div>
            <h2 className="text-xs font-mono font-bold tracking-wider text-zinc-100 uppercase">
              Primary Workspace
            </h2>
            <p className="text-[10px] font-mono text-zinc-500 uppercase tracking-widest">
              System Canvas
            </p>
          </div>
        </div>

        {/* Right Header Controls Accent */}
        <div className="flex items-center gap-2.5">
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-900/90 border border-zinc-800 text-[10px] font-mono text-zinc-400">
            <span className="w-1.5 h-1.5 rounded-full bg-zinc-400 animate-pulse" />
            <span>STANDBY</span>
          </div>
          <div className="h-3 w-[1px] bg-zinc-800" />
          <div className="flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
            <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
            <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
          </div>
        </div>
      </div>

      {/* Main Canvas Body */}
      <div className="flex-1 rounded-xl bg-zinc-900/40 border border-zinc-800/40 relative flex flex-col items-center justify-center p-6 overflow-hidden">
        {/* Subtle Tech Grid Background */}
        <div className="absolute inset-0 bg-[radial-gradient(#27272a_1px,transparent_1px)] [background-size:16px_16px] opacity-40" />

        {/* Corner Tech Brackets */}
        <div className="absolute top-3 left-3 w-4 h-4 border-t-2 border-l-2 border-zinc-700/60" />
        <div className="absolute top-3 right-3 w-4 h-4 border-t-2 border-r-2 border-zinc-700/60" />
        <div className="absolute bottom-3 left-3 w-4 h-4 border-b-2 border-l-2 border-zinc-700/60" />
        <div className="absolute bottom-3 right-3 w-4 h-4 border-b-2 border-r-2 border-zinc-700/60" />

        {/* Center Futuristic Graphic */}
        <div className="relative z-10 flex flex-col items-center text-center max-w-sm">
          <div className="w-14 h-14 rounded-2xl bg-zinc-900/80 border border-zinc-700/60 flex items-center justify-center shadow-xl shadow-black/40 mb-3.5 group">
            <Terminal className="w-6 h-6 text-zinc-400 group-hover:text-zinc-100 transition-colors" />
          </div>
          <h3 className="text-sm font-semibold text-zinc-200 tracking-wide font-mono uppercase mb-1">
            Workspace Ready
          </h3>
          <p className="text-xs text-zinc-500 font-sans leading-relaxed">
            Awaiting active workload, multi-window screen orchestration, or vision diagnostics.
          </p>
        </div>

        {/* Background Radar / Concentric Rings */}
        <div className="absolute w-72 h-72 rounded-full border border-zinc-800/20 pointer-events-none" />
        <div className="absolute w-96 h-96 rounded-full border border-zinc-800/10 pointer-events-none" />
      </div>
    </div>
  );
};
