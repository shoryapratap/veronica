import React from "react";
import { ConnectionStatus } from "../types/protocol";
import { Activity, Server, LayoutDashboard, LayoutGrid, Sliders } from "lucide-react";

export type TabType = "main" | "apps" | "settings";

interface Props {
  status: ConnectionStatus;
  cpu?: number;
  ram?: string;
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
}

export const StatusBar: React.FC<Props> = ({
  status,
  cpu = 0,
  ram = "0/0 MB",
  activeTab,
  onTabChange,
}) => {
  const isOnline = status === "ONLINE";

  return (
    <header className="flex items-center justify-between px-6 py-2.5 bg-black/70 backdrop-blur-xl border-b border-white/[0.08] text-xs font-mono text-zinc-400 shrink-0 select-none">
      {/* Left: Branding & Connection Status */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2.5">
          <img
            src="/veronica_logo.png"
            alt="Veronica Logo"
            className="w-7 h-7 rounded-lg object-contain shadow-[0_0_12px_rgba(255,255,255,0.08)] border border-white/10"
          />
          <span className="font-semibold tracking-wider text-zinc-100">VERONICA</span>
        </div>
        <span className="text-zinc-700">|</span>
        <span
          className={`flex items-center gap-1.5 px-2.5 py-0.5 rounded-full border backdrop-blur-md transition-all ${
            isOnline
              ? "bg-zinc-900/60 text-zinc-200 border-white/15 shadow-[0_0_10px_rgba(255,255,255,0.05)]"
              : "bg-zinc-950/60 text-zinc-500 border-zinc-800"
          }`}
        >
          <span
            className={`w-1.5 h-1.5 rounded-full transition-all ${
              isOnline ? "bg-white shadow-[0_0_6px_rgba(255,255,255,0.9)]" : "bg-zinc-600"
            }`}
          />
          <span className="tracking-wide">{status}</span>
        </span>
      </div>

      {/* Center: Top Navbar with Main / Apps / Settings */}
      <nav className="flex items-center gap-1 bg-zinc-950/90 p-1 rounded-xl border border-zinc-800/80 shadow-inner">
        <button
          onClick={() => onTabChange("main")}
          className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-mono font-medium transition-all cursor-pointer ${
            activeTab === "main"
              ? "bg-zinc-800 text-white border border-zinc-700/80 shadow-md shadow-black/40"
              : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60"
          }`}
        >
          <LayoutDashboard className="w-3.5 h-3.5" />
          <span>Main</span>
        </button>

        <button
          onClick={() => onTabChange("apps")}
          className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-mono font-medium transition-all cursor-pointer ${
            activeTab === "apps"
              ? "bg-zinc-800 text-white border border-zinc-700/80 shadow-md shadow-black/40"
              : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60"
          }`}
        >
          <LayoutGrid className="w-3.5 h-3.5" />
          <span>Apps</span>
        </button>

        <button
          onClick={() => onTabChange("settings")}
          className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-mono font-medium transition-all cursor-pointer ${
            activeTab === "settings"
              ? "bg-zinc-800 text-white border border-zinc-700/80 shadow-md shadow-black/40"
              : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60"
          }`}
        >
          <Sliders className="w-3.5 h-3.5" />
          <span>Settings</span>
        </button>
      </nav>

      {/* Right: Telemetry (CPU & RAM) */}
      <div className="flex items-center gap-5">
        <div className="flex items-center gap-1.5">
          <Activity className="w-3.5 h-3.5 text-zinc-300" />
          <span className="text-zinc-300">
            CPU: <span className="text-zinc-100 font-medium">{cpu.toFixed(1)}%</span>
          </span>
        </div>
        <div className="flex items-center gap-1.5">
          <Server className="w-3.5 h-3.5 text-zinc-300" />
          <span className="text-zinc-300">
            RAM: <span className="text-zinc-100 font-medium">{ram}</span>
          </span>
        </div>
      </div>
    </header>
  );
};
