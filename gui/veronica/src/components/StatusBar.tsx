import React from "react";
import { ConnectionStatus } from "../types/protocol";
import { Activity, Radio, Server } from "lucide-react";

interface Props {
  status: ConnectionStatus;
  cpu?: number;
  ram?: string;
}

export const StatusBar: React.FC<Props> = ({ status, cpu = 0, ram = "0/0 MB" }) => {
  const isOnline = status === "ONLINE";

  return (
    <header className="flex items-center justify-between px-6 py-3 bg-zinc-950/80 backdrop-blur-md border-b border-zinc-800/60 text-xs font-mono text-zinc-400">
      <div className="flex items-center gap-3">
        <span className="flex items-center gap-1.5 font-semibold tracking-wider text-zinc-200">
          <Radio className={`w-3.5 h-3.5 ${isOnline ? "text-emerald-400 animate-pulse" : "text-zinc-600"}`} />
          VERONICA // SYSTEM
        </span>
        <span className="text-zinc-700">|</span>
        <span
          className={`flex items-center gap-1.5 px-2 py-0.5 rounded-full border ${
            isOnline
              ? "bg-emerald-950/40 text-emerald-400 border-emerald-800/40"
              : "bg-rose-950/40 text-rose-400 border-rose-800/40"
          }`}
        >
          <span className={`w-1.5 h-1.5 rounded-full ${isOnline ? "bg-emerald-400" : "bg-rose-500"}`} />
          {status}
        </span>
      </div>

      <div className="flex items-center gap-5">
        <div className="flex items-center gap-1.5">
          <Activity className="w-3.5 h-3.5 text-indigo-400" />
          <span>CPU: {cpu.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1.5">
          <Server className="w-3.5 h-3.5 text-cyan-400" />
          <span>RAM: {ram}</span>
        </div>
      </div>
    </header>
  );
};
