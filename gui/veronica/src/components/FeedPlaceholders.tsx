import React from "react";
import { Monitor, Camera } from "lucide-react";

export const FeedPlaceholders: React.FC = () => {
  return (
    <div className="grid grid-cols-2 gap-2.5 shrink-0">
      {/* 1. Screen Sharing Window */}
      <div className="flex flex-col bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-xl p-2.5 overflow-hidden group hover:border-zinc-700/70 transition-all shadow-lg">
        <div className="flex items-center justify-between pb-2 border-b border-zinc-800/40 mb-2">
          <div className="flex items-center gap-1.5">
            <Monitor className="w-3.5 h-3.5 text-zinc-400 group-hover:text-zinc-200 transition-colors" />
            <span className="text-[11px] font-mono font-medium text-zinc-300 uppercase tracking-wider">
              Screen Share
            </span>
          </div>
          <span className="px-1.5 py-0.5 rounded text-[9px] font-mono bg-zinc-900 border border-zinc-800 text-zinc-500">
            OFFLINE
          </span>
        </div>

        {/* Viewport Canvas Placeholder */}
        <div className="h-24 rounded-lg bg-zinc-900/60 border border-zinc-800/50 flex flex-col items-center justify-center relative overflow-hidden">
          {/* Subtle tech grid background */}
          <div className="absolute inset-0 bg-[radial-gradient(#27272a_1px,transparent_1px)] [background-size:12px_12px] opacity-40" />

          {/* Crosshairs & Content */}
          <div className="relative z-10 flex flex-col items-center text-center p-2">
            <Monitor className="w-5 h-5 text-zinc-600 mb-1" />
            <p className="text-[10px] font-mono text-zinc-500">DISPLAY 1</p>
            <p className="text-[9px] font-mono text-zinc-600">Feed Inactive</p>
          </div>

          <div className="absolute bottom-1 right-1.5 text-[8px] font-mono text-zinc-600">
            1920x1080
          </div>
        </div>
      </div>

      {/* 2. Camera Feed Window */}
      <div className="flex flex-col bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-xl p-2.5 overflow-hidden group hover:border-zinc-700/70 transition-all shadow-lg">
        <div className="flex items-center justify-between pb-2 border-b border-zinc-800/40 mb-2">
          <div className="flex items-center gap-1.5">
            <Camera className="w-3.5 h-3.5 text-zinc-400 group-hover:text-zinc-200 transition-colors" />
            <span className="text-[11px] font-mono font-medium text-zinc-300 uppercase tracking-wider">
              Camera
            </span>
          </div>
          <span className="px-1.5 py-0.5 rounded text-[9px] font-mono bg-zinc-900 border border-zinc-800 text-zinc-500">
            STANDBY
          </span>
        </div>

        {/* Viewport Canvas Placeholder */}
        <div className="h-24 rounded-lg bg-zinc-900/60 border border-zinc-800/50 flex flex-col items-center justify-center relative overflow-hidden">
          {/* Corner Viewfinder Brackets */}
          <div className="absolute top-1.5 left-1.5 w-2 h-2 border-t border-l border-zinc-600" />
          <div className="absolute top-1.5 right-1.5 w-2 h-2 border-t border-r border-zinc-600" />
          <div className="absolute bottom-1.5 left-1.5 w-2 h-2 border-b border-l border-zinc-600" />
          <div className="absolute bottom-1.5 right-1.5 w-2 h-2 border-b border-r border-zinc-600" />

          {/* Crosshairs & Content */}
          <div className="relative z-10 flex flex-col items-center text-center p-2">
            <Camera className="w-5 h-5 text-zinc-600 mb-1" />
            <p className="text-[10px] font-mono text-zinc-500">OPTICAL CAM 01</p>
            <p className="text-[9px] font-mono text-zinc-600">Sensor Standby</p>
          </div>

          <div className="absolute bottom-1 right-1.5 text-[8px] font-mono text-zinc-600">
            FPS: 0
          </div>
        </div>
      </div>
    </div>
  );
};
