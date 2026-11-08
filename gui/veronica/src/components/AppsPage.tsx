import React, { useState } from "react";
import {
  LayoutGrid,
  Play,
  Square,
  Search,
  ExternalLink,
  Laptop,
  CheckCircle2,
  FolderOpen,
} from "lucide-react";

interface Props {
  onLaunchApp: (appName: string) => void;
}

interface QuickApp {
  name: string;
  category: string;
  iconText: string;
  target: string;
}

interface AppGroup {
  id: string;
  name: string;
  description: string;
  apps: string[];
}

const QUICK_APPS: QuickApp[] = [
  { name: "Google Chrome", category: "Browser", iconText: "CH", target: "chrome" },
  { name: "Visual Studio Code", category: "Development", iconText: "VS", target: "code" },
  { name: "Windows Terminal", category: "System", iconText: "WT", target: "wt" },
  { name: "Notepad", category: "Utility", iconText: "NP", target: "notepad" },
  { name: "Calculator", category: "Utility", iconText: "CAL", target: "calc" },
  { name: "Spotify", category: "Media", iconText: "SP", target: "spotify" },
  { name: "Task Manager", category: "System", iconText: "TM", target: "taskmgr" },
  { name: "File Explorer", category: "System", iconText: "EXP", target: "explorer" },
];

const APP_GROUPS: AppGroup[] = [
  {
    id: "dev-setup",
    name: "Dev Workspace",
    description: "Launches VS Code, Terminal, and Chrome simultaneously.",
    apps: ["Visual Studio Code", "Windows Terminal", "Google Chrome"],
  },
  {
    id: "focus-mode",
    name: "Deep Focus",
    description: "Clean workspace with Spotify and Notepad only.",
    apps: ["Spotify", "Notepad"],
  },
  {
    id: "sys-admin",
    name: "System Diagnostics",
    description: "Opens Task Manager, PowerShell, and Terminal.",
    apps: ["Task Manager", "Windows Terminal"],
  },
];

export const AppsPage: React.FC<Props> = ({ onLaunchApp }) => {
  const [search, setSearch] = useState("");
  const [launchedNotice, setLaunchedNotice] = useState<string | null>(null);

  const handleLaunch = (target: string, displayName: string) => {
    onLaunchApp(target);
    setLaunchedNotice(`Launched ${displayName}`);
    setTimeout(() => setLaunchedNotice(null), 3000);
  };

  const filteredApps = QUICK_APPS.filter((app) =>
    app.name.toLowerCase().includes(search.toLowerCase()) ||
    app.category.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="flex-1 flex flex-col p-6 overflow-y-auto max-w-7xl mx-auto w-full gap-6 select-none font-sans">
      {/* Page Header */}
      <div className="flex items-center justify-between pb-4 border-b border-zinc-800/80">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <LayoutGrid className="w-5 h-5 text-zinc-300" />
            <h1 className="text-lg font-semibold tracking-wide text-zinc-100 font-mono uppercase">
              Applications & Workspace Manager
            </h1>
          </div>
          <p className="text-xs text-zinc-500 font-sans">
            Launch installed Windows applications and multi-app workspace groups via Veronica OS platform adapter.
          </p>
        </div>

        {launchedNotice && (
          <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-zinc-900 border border-zinc-700/60 text-xs text-zinc-200 font-mono shadow-lg animate-in fade-in">
            <CheckCircle2 className="w-4 h-4 text-zinc-100" />
            <span>{launchedNotice}</span>
          </div>
        )}
      </div>

      {/* Section 1: Workspace App Groups */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <Laptop className="w-4 h-4 text-zinc-400" />
            <h2 className="text-xs font-mono font-semibold tracking-wider text-zinc-300 uppercase">
              Workspace Groups (Batch Launch)
            </h2>
          </div>
          <span className="text-[11px] font-mono text-zinc-500">
            {APP_GROUPS.length} Preset Groups
          </span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {APP_GROUPS.map((group) => (
            <div
              key={group.id}
              className="bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl p-4 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-xl group"
            >
              <div>
                <div className="flex items-center justify-between mb-2">
                  <h3 className="text-sm font-semibold text-zinc-200 font-mono">
                    {group.name}
                  </h3>
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-400">
                    {group.apps.length} APPS
                  </span>
                </div>
                <p className="text-xs text-zinc-500 mb-3 leading-relaxed">
                  {group.description}
                </p>

                <div className="flex flex-wrap gap-1.5 mb-4">
                  {group.apps.map((app) => (
                    <span
                      key={app}
                      className="px-2 py-0.5 rounded-md bg-zinc-900/80 border border-zinc-800/80 text-[10px] text-zinc-400 font-mono"
                    >
                      {app}
                    </span>
                  ))}
                </div>
              </div>

              <div className="flex items-center gap-2 pt-3 border-t border-zinc-900">
                <button
                  onClick={() => {
                    group.apps.forEach((a) => {
                      const match = QUICK_APPS.find((qa) => qa.name === a);
                      handleLaunch(match ? match.target : a.toLowerCase(), group.name);
                    });
                  }}
                  className="flex-1 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-xl text-xs font-mono flex items-center justify-center gap-1.5 transition-colors border border-zinc-700/60 shadow-lg cursor-pointer"
                >
                  <Play className="w-3.5 h-3.5" /> Launch Group
                </button>
                <button
                  onClick={() => handleLaunch("taskmgr", "Task Manager")}
                  className="px-3 py-2 bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 rounded-xl text-xs font-mono flex items-center justify-center transition-colors border border-zinc-800 cursor-pointer"
                  title="Close processes"
                >
                  <Square className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Section 2: Quick Launch Apps */}
      <div className="flex-1 flex flex-col">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <FolderOpen className="w-4 h-4 text-zinc-400" />
            <h2 className="text-xs font-mono font-semibold tracking-wider text-zinc-300 uppercase">
              Quick Application Launcher
            </h2>
          </div>

          <div className="relative w-64">
            <Search className="w-3.5 h-3.5 text-zinc-500 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Filter installed apps..."
              className="w-full bg-zinc-900/80 border border-zinc-800 focus:border-zinc-600 rounded-xl pl-9 pr-3 py-1.5 text-xs text-zinc-200 placeholder:text-zinc-600 focus:outline-none font-mono"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3">
          {filteredApps.map((app) => (
            <div
              key={app.name}
              onClick={() => handleLaunch(app.target, app.name)}
              className="bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 hover:border-zinc-600/80 rounded-2xl p-3.5 flex items-center justify-between cursor-pointer group transition-all shadow-lg hover:shadow-black/50"
            >
              <div className="flex items-center gap-3 min-w-0">
                <div className="w-10 h-10 rounded-xl bg-zinc-900 border border-zinc-800 group-hover:border-zinc-700 flex items-center justify-center text-xs font-mono font-bold text-zinc-300 group-hover:text-white transition-colors shrink-0">
                  {app.iconText}
                </div>
                <div className="min-w-0">
                  <h4 className="text-xs font-semibold text-zinc-200 group-hover:text-white truncate transition-colors">
                    {app.name}
                  </h4>
                  <p className="text-[10px] text-zinc-500 font-mono truncate">
                    {app.category}
                  </p>
                </div>
              </div>

              <div className="p-1.5 rounded-lg text-zinc-600 group-hover:text-zinc-300 transition-colors shrink-0">
                <ExternalLink className="w-3.5 h-3.5" />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

