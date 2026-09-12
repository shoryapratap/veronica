import { useState } from "react";
import { StatusBar } from "./components/StatusBar";
import { ConnectionStatus } from "./types/protocol";
import { Bot, Terminal, Mic, ShieldCheck } from "lucide-react";
import "./App.css";

function App() {
  // Mock status for now until we connect to the live daemon socket
  const [status] = useState<ConnectionStatus>("ONLINE");
  const [cpu] = useState<number>(4.2);
  const [ram] = useState<string>("142 / 16384 MB");

  return (
    <div className="flex flex-col h-screen bg-zinc-950 text-zinc-100 font-sans selection:bg-indigo-500/30 select-none">
      {/* Top Status Bar */}
      <StatusBar status={status} cpu={cpu} ram={ram} />

      {/* Main Center Stage */}
      <main className="flex-1 flex flex-col items-center justify-center p-8">
        <div className="relative flex items-center justify-center mb-8">
          {/* Glowing Aura Ring */}
          <div className="absolute w-36 h-36 rounded-full bg-indigo-500/20 blur-2xl animate-pulse" />
          <div className="relative w-28 h-28 rounded-3xl bg-zinc-900 border border-zinc-800/80 shadow-2xl flex items-center justify-center">
            <Bot className="w-14 h-14 text-indigo-400" />
          </div>
        </div>

        <h1 className="text-3xl font-bold tracking-tight mb-2 text-zinc-100">
          Project Veronica
        </h1>
        <p className="text-sm font-mono text-zinc-400 mb-8 max-w-sm text-center">
          Autonomous 3-Tier Decoupled AI System
        </p>

        {/* Quick Diagnostics Grid */}
        <div className="grid grid-cols-3 gap-4 w-full max-w-md text-xs font-mono">
          <div className="p-3 rounded-xl bg-zinc-900/60 border border-zinc-800/50 flex flex-col items-center gap-1.5">
            <Terminal className="w-4 h-4 text-emerald-400" />
            <span className="text-zinc-400">Daemon</span>
            <span className="text-emerald-400 font-semibold">127.0.0.1:8765</span>
          </div>

          <div className="p-3 rounded-xl bg-zinc-900/60 border border-zinc-800/50 flex flex-col items-center gap-1.5">
            <Mic className="w-4 h-4 text-cyan-400" />
            <span className="text-zinc-400">Audio Pipeline</span>
            <span className="text-cyan-400 font-semibold">Ready (CPAL)</span>
          </div>

          <div className="p-3 rounded-xl bg-zinc-900/60 border border-zinc-800/50 flex flex-col items-center gap-1.5">
            <ShieldCheck className="w-4 h-4 text-indigo-400" />
            <span className="text-zinc-400">Architecture</span>
            <span className="text-indigo-400 font-semibold">Decoupled</span>
          </div>
        </div>
      </main>

      {/* Bottom Footer */}
      <footer className="px-6 py-3 border-t border-zinc-900 text-center text-xs font-mono text-zinc-600">
        VERONICA AI // SYSTEM FACE v0.1.0
      </footer>
    </div>
  );
}

export default App;
