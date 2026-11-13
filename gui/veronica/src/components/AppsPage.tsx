import React, { useState, useEffect, useMemo } from "react";
import {
  LayoutGrid,
  Layers,
  Play,
  Square,
  Search,
  Plus,
  Trash2,
  Pencil,
  SlidersHorizontal,
  X,
  Check,
  CheckCircle2,
  Terminal,
  Globe,
  Music,
  Shield,
  FileCode,
  FileText,
  Gamepad2,
  Camera,
  Cpu,
  AppWindow,
  RotateCw,
  FolderOpen,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

export interface InstalledApp {
  name: string;
  path: string;
  app_type: string;
  installed_by: string;
  is_system_app: boolean;
  icon_path?: string | null;
}

export interface AppGroup {
  name: string;
  apps: string[];
  description?: string;
  created_at?: string;
}

interface AppsPageProps {
  onLaunchApp: (appName: string) => void;
  onCloseApp?: (appName: string) => void;
}

export const AppsPage: React.FC<AppsPageProps> = ({ onLaunchApp, onCloseApp }) => {
  const [apps, setApps] = useState<InstalledApp[]>([]);
  const [groups, setGroups] = useState<AppGroup[]>([]);
  const [loading, setLoading] = useState(true);

  // Search & Filter (Left Window)
  const [search, setSearch] = useState("");
  const [filterType, setFilterType] = useState<"ALL" | "USER" | "SYSTEM" | "WIN32" | "UWP">("ALL");

  // Notifications
  const [notice, setNotice] = useState<{ text: string; type: "launch" | "close" | "info" } | null>(null);

  // Create Group Modal State (Right Window)
  const [isCreatingGroup, setIsCreatingGroup] = useState(false);
  const [newGroupName, setNewGroupName] = useState("");
  const [newGroupDesc, setNewGroupDesc] = useState("");
  const [selectedApps, setSelectedApps] = useState<string[]>([]);
  const [groupPickerSearch, setGroupPickerSearch] = useState("");

  // Rename Group Modal State (Right Window)
  const [renamingGroup, setRenamingGroup] = useState<AppGroup | null>(null);
  const [renameInputValue, setRenameInputValue] = useState("");

  // Edit Group Modal State (Add/Remove apps)
  const [editingGroup, setEditingGroup] = useState<AppGroup | null>(null);
  const [editGroupName, setEditGroupName] = useState("");
  const [editGroupDesc, setEditGroupDesc] = useState("");
  const [editSelectedApps, setEditSelectedApps] = useState<string[]>([]);
  const [editPickerSearch, setEditPickerSearch] = useState("");

  const showNotice = (text: string, type: "launch" | "close" | "info" = "info") => {
    setNotice({ text, type });
    setTimeout(() => setNotice(null), 3000);
  };

  // Load real scanned apps with real icons directly from system index
  const loadData = async () => {
    setLoading(true);
    let loadedApps: InstalledApp[] = [];

    // 1. Try Tauri invoke (direct from %APPDATA%\Veronica\installed_apps.json)
    try {
      const fetched = await invoke<InstalledApp[]>("get_installed_apps");
      if (fetched && Array.isArray(fetched) && fetched.length > 0) {
        loadedApps = fetched;
      }
    } catch {
      // In Vite web dev preview
    }

    // 2. Fallback to /installed_apps.json (synced directly from scan_app)
    if (loadedApps.length === 0) {
      try {
        const res = await fetch("/installed_apps.json");
        if (res.ok) {
          const data = await res.json();
          if (Array.isArray(data) && data.length > 0) {
            loadedApps = data;
          }
        }
      } catch {
        // failed to fetch json
      }
    }

    setApps(loadedApps);

    // 3. Load workspace groups
    try {
      const fetchedGroups = await invoke<AppGroup[]>("get_app_groups");
      if (fetchedGroups && Array.isArray(fetchedGroups)) {
        setGroups(fetchedGroups);
      }
    } catch {
      // ignore
    }

    setLoading(false);
  };

  const handleRescan = async () => {
    setLoading(true);
    try {
      const refreshed = await invoke<InstalledApp[]>("rescan_apps");
      if (refreshed && Array.isArray(refreshed) && refreshed.length > 0) {
        setApps(refreshed);
        showNotice(`Rescanned ${refreshed.length} applications with real icons`, "info");
        setLoading(false);
        return;
      }
    } catch {
      // fallback
    }
    await loadData();
    showNotice("Refreshed applications", "info");
  };

  useEffect(() => {
    loadData();
  }, []);

  // Filtered Apps List
  const filteredApps = useMemo(() => {
    return apps.filter((app) => {
      const matchesSearch =
        app.name.toLowerCase().includes(search.toLowerCase()) ||
        app.path.toLowerCase().includes(search.toLowerCase());

      if (!matchesSearch) return false;

      if (filterType === "USER") return !app.is_system_app;
      if (filterType === "SYSTEM") return app.is_system_app;
      if (filterType === "WIN32") return app.app_type.toUpperCase() === "WIN32";
      if (filterType === "UWP") return app.app_type.toUpperCase() === "UWP";
      return true;
    });
  }, [apps, search, filterType]);

  // Fallback category icon if real icon fails
  const getAppCategoryIcon = (name: string) => {
    const lower = name.toLowerCase();
    if (lower.includes("chrome") || lower.includes("edge") || lower.includes("brave") || lower.includes("firefox") || lower.includes("browser")) {
      return <Globe className="w-5 h-5 text-zinc-300" />;
    }
    if (lower.includes("code") || lower.includes("visual studio") || lower.includes("git") || lower.includes("ide")) {
      return <FileCode className="w-5 h-5 text-zinc-300" />;
    }
    if (lower.includes("terminal") || lower.includes("cmd") || lower.includes("powershell") || lower.includes("bash")) {
      return <Terminal className="w-5 h-5 text-zinc-300" />;
    }
    if (lower.includes("spotify") || lower.includes("music") || lower.includes("audio") || lower.includes("player")) {
      return <Music className="w-5 h-5 text-zinc-300" />;
    }
    if (lower.includes("camera") || lower.includes("video") || lower.includes("obs") || lower.includes("stream")) {
      return <Camera className="w-5 h-5 text-zinc-300" />;
    }
    if (lower.includes("security") || lower.includes("defender") || lower.includes("verifier")) {
      return <Shield className="w-5 h-5 text-zinc-300" />;
    }
    if (lower.includes("steam") || lower.includes("xbox") || lower.includes("game")) {
      return <Gamepad2 className="w-5 h-5 text-zinc-300" />;
    }
    if (lower.includes("word") || lower.includes("excel") || lower.includes("notes") || lower.includes("document") || lower.includes("pdf")) {
      return <FileText className="w-5 h-5 text-zinc-300" />;
    }
    if (lower.includes("taskmgr") || lower.includes("tools") || lower.includes("diagnostics") || lower.includes("calc")) {
      return <Cpu className="w-5 h-5 text-zinc-300" />;
    }
    return <AppWindow className="w-5 h-5 text-zinc-400" />;
  };

  // Launch single app with real path & type resolution (no error dialogs)
  const handleLaunch = async (app: InstalledApp) => {
    let launched = false;
    try {
      await invoke("launch_app", {
        name: app.name,
        path: app.path,
        appType: app.app_type,
      });
      launched = true;
    } catch {
      // In web browser dev without Tauri, send via daemon WebSocket
      onLaunchApp(app.name);
      launched = true;
    }
    if (launched) {
      showNotice(`Launched ${app.name}`, "launch");
    }
  };

  // Close single app
  const handleClose = async (app: InstalledApp) => {
    try {
      await invoke("close_app", { target: app.name });
    } catch {
      // fallback to WebSocket
      if (onCloseApp) {
        onCloseApp(app.name);
      }
    }
    showNotice(`Terminated ${app.name}`, "close");
  };

  // Launch workspace group
  const handleLaunchGroup = (group: AppGroup) => {
    group.apps.forEach((appName, index) => {
      setTimeout(() => {
        const found = apps.find((a) => a.name.toLowerCase() === appName.toLowerCase());
        if (found) {
          handleLaunch(found);
        } else {
          onLaunchApp(appName);
        }
      }, index * 200);
    });
    showNotice(`Launching group "${group.name}" (${group.apps.length} apps)`, "launch");
  };

  // Close workspace group
  const handleCloseGroup = (group: AppGroup) => {
    group.apps.forEach((appName) => {
      const found = apps.find((a) => a.name.toLowerCase() === appName.toLowerCase());
      if (found) {
        handleClose(found);
      } else if (onCloseApp) {
        onCloseApp(appName);
      }
    });
    showNotice(`Terminated group "${group.name}"`, "close");
  };

  // Toggle selection in Group Creation modal
  const toggleAppSelection = (name: string) => {
    setSelectedApps((prev) =>
      prev.includes(name) ? prev.filter((a) => a !== name) : [...prev, name]
    );
  };

  // Save new workspace group
  const handleSaveGroup = async () => {
    const trimmed = newGroupName.trim();
    if (!trimmed) {
      showNotice("Please enter a group name", "info");
      return;
    }
    if (selectedApps.length === 0) {
      showNotice("Please select at least one application", "info");
      return;
    }

    const newGroup: AppGroup = {
      name: trimmed,
      apps: selectedApps,
      description: newGroupDesc.trim() || undefined,
    };

    try {
      await invoke("save_app_group", {
        name: trimmed,
        apps: selectedApps,
        description: newGroupDesc.trim() || null,
        oldName: null,
      });
    } catch {
      // local update fallback
    }

    setGroups((prev) => {
      const idx = prev.findIndex((g) => g.name.toLowerCase() === trimmed.toLowerCase());
      if (idx >= 0) {
        const copy = [...prev];
        copy[idx] = newGroup;
        return copy;
      }
      return [...prev, newGroup];
    });

    setIsCreatingGroup(false);
    setNewGroupName("");
    setNewGroupDesc("");
    setSelectedApps([]);
    setGroupPickerSearch("");
    showNotice(`Created group "${trimmed}"`, "info");
  };

  // Open Rename Modal
  const openRenameModal = (group: AppGroup) => {
    setRenamingGroup(group);
    setRenameInputValue(group.name);
  };

  // Save Rename
  const handleSaveRename = async () => {
    if (!renamingGroup) return;
    const trimmed = renameInputValue.trim();
    if (!trimmed) {
      showNotice("Group name cannot be empty", "info");
      return;
    }
    if (trimmed.toLowerCase() === renamingGroup.name.toLowerCase()) {
      setRenamingGroup(null);
      return;
    }

    try {
      await invoke("rename_app_group", {
        oldName: renamingGroup.name,
        newName: trimmed,
      });
    } catch {
      // fallback
    }

    setGroups((prev) =>
      prev.map((g) => (g.name === renamingGroup.name ? { ...g, name: trimmed } : g))
    );
    showNotice(`Renamed group to "${trimmed}"`, "info");
    setRenamingGroup(null);
  };

  // Open Edit Modal (Add/Remove apps)
  const openEditModal = (group: AppGroup) => {
    setEditingGroup(group);
    setEditGroupName(group.name);
    setEditGroupDesc(group.description || "");
    setEditSelectedApps([...group.apps]);
    setEditPickerSearch("");
  };

  // Toggle selection inside Edit Modal
  const toggleEditAppSelection = (name: string) => {
    setEditSelectedApps((prev) =>
      prev.includes(name) ? prev.filter((a) => a !== name) : [...prev, name]
    );
  };

  // Save Edit Changes (Name, Description, Apps)
  const handleSaveEdit = async () => {
    if (!editingGroup) return;
    const trimmedName = editGroupName.trim();
    if (!trimmedName) {
      showNotice("Group name cannot be empty", "info");
      return;
    }
    if (editSelectedApps.length === 0) {
      showNotice("Select at least one application", "info");
      return;
    }

    const updatedGroup: AppGroup = {
      name: trimmedName,
      apps: editSelectedApps,
      description: editGroupDesc.trim() || undefined,
    };

    try {
      await invoke("save_app_group", {
        name: trimmedName,
        apps: editSelectedApps,
        description: editGroupDesc.trim() || null,
        oldName: editingGroup.name,
      });
    } catch {
      // fallback
    }

    setGroups((prev) =>
      prev.map((g) => (g.name === editingGroup.name ? updatedGroup : g))
    );
    showNotice(`Updated group "${trimmedName}" (${editSelectedApps.length} apps)`, "info");
    setEditingGroup(null);
  };

  // Quick remove single app from group card pill
  const handleQuickRemoveApp = async (groupName: string, appName: string) => {
    try {
      await invoke("remove_app_from_group", {
        groupName,
        appName,
      });
    } catch {
      // fallback
    }

    setGroups((prev) =>
      prev.map((g) => {
        if (g.name === groupName) {
          return { ...g, apps: g.apps.filter((a) => a !== appName) };
        }
        return g;
      })
    );
    showNotice(`Removed ${appName} from ${groupName}`, "info");
  };

  // Delete workspace group
  const handleDeleteGroup = async (name: string) => {
    try {
      await invoke("delete_app_group", { name });
    } catch {
      // fallback
    }
    setGroups((prev) => prev.filter((g) => g.name !== name));
    showNotice(`Deleted group "${name}"`, "info");
  };

  // Apps available for picker inside create modal
  const pickerFilteredApps = useMemo(() => {
    return apps.filter((app) =>
      app.name.toLowerCase().includes(groupPickerSearch.toLowerCase())
    );
  }, [apps, groupPickerSearch]);

  // Apps available for picker inside edit modal
  const editPickerFilteredApps = useMemo(() => {
    return apps.filter((app) =>
      app.name.toLowerCase().includes(editPickerSearch.toLowerCase())
    );
  }, [apps, editPickerSearch]);

  return (
    <div className="flex-1 flex overflow-hidden p-3 gap-3 min-h-0 select-none font-sans relative">
      {/* Global Notification Toast */}
      {notice && (
        <div className="absolute top-5 right-6 z-50 flex items-center gap-2 px-3.5 py-2 rounded-xl bg-zinc-900/90 backdrop-blur-xl border border-zinc-700/80 text-xs font-mono text-zinc-100 shadow-2xl shadow-black/60 animate-in fade-in slide-in-from-top-2">
          {notice.type === "close" ? (
            <Square className="w-3.5 h-3.5 text-zinc-400" />
          ) : (
            <CheckCircle2 className="w-3.5 h-3.5 text-zinc-200" />
          )}
          <span>{notice.text}</span>
        </div>
      )}

      {/* ========================================================================= */}
      {/* LEFT WINDOW: INSTALLED APPLICATIONS (SCANNED 5-COLUMN GRID WITH REAL ICONS) */}
      {/* ========================================================================= */}
      <section className="flex-1 flex flex-col bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl overflow-hidden shadow-2xl min-w-0">
        {/* Window Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-800/60 shrink-0 bg-zinc-950/40">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-center shadow-md">
              <LayoutGrid className="w-4 h-4 text-zinc-300" />
            </div>
            <div>
              <h2 className="text-xs font-mono font-bold tracking-wider text-zinc-100 uppercase">
                Installed Applications
              </h2>
              <p className="text-[10px] font-mono text-zinc-500 uppercase tracking-widest">
                Scanned Windows Registry & System Index
              </p>
            </div>
          </div>

          {/* Right Header Status Accent */}
          <div className="flex items-center gap-2.5">
            <button
              onClick={handleRescan}
              title="Rescan Applications and Update Real Icons"
              className="p-1.5 rounded-lg bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
            >
              <RotateCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
            </button>
            <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-900/90 border border-zinc-800 text-[10px] font-mono text-zinc-400">
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-300 animate-pulse" />
              <span>{apps.length} APPS</span>
            </div>
            <div className="h-3 w-[1px] bg-zinc-800" />
            <div className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
            </div>
          </div>
        </div>

        {/* Search & Filter Toolbar */}
        <div className="p-3 border-b border-zinc-800/60 bg-zinc-900/20 shrink-0 flex flex-col gap-2.5">
          {/* Search Input Bar */}
          <div className="relative flex items-center">
            <Search className="w-4 h-4 text-zinc-500 absolute left-3 pointer-events-none" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search 174 scanned applications..."
              className="w-full pl-9 pr-8 py-1.5 rounded-xl bg-zinc-900/80 border border-zinc-800/80 text-xs text-zinc-200 placeholder:text-zinc-600 focus:outline-none focus:border-zinc-700 transition-colors font-mono"
            />
            {search && (
              <button
                onClick={() => setSearch("")}
                className="absolute right-2.5 text-zinc-500 hover:text-zinc-300 cursor-pointer"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            )}
          </div>

          {/* Filter Chips */}
          <div className="flex items-center gap-1.5 overflow-x-auto text-[10px] font-mono scrollbar-none">
            {(["ALL", "USER", "SYSTEM", "WIN32", "UWP"] as const).map((ft) => (
              <button
                key={ft}
                onClick={() => setFilterType(ft)}
                className={`px-2.5 py-0.5 rounded-lg border transition-all cursor-pointer shrink-0 ${
                  filterType === ft
                    ? "bg-zinc-800 text-zinc-100 border-zinc-600 shadow-sm"
                    : "bg-zinc-950/60 text-zinc-500 border-zinc-800/80 hover:text-zinc-300 hover:border-zinc-700"
                }`}
              >
                {ft}
              </button>
            ))}
            <span className="ml-auto text-[10px] font-mono text-zinc-500 shrink-0">
              {filteredApps.length} shown
            </span>
          </div>
        </div>

        {/* 5 in a row Grid of Scanned Apps with REAL ICONS */}
        <div className="flex-1 overflow-y-auto p-3 min-h-0">
          {loading ? (
            <div className="h-full flex flex-col items-center justify-center text-center p-8 text-zinc-500">
              <RotateCw className="w-6 h-6 text-zinc-500 animate-spin mb-2" />
              <p className="text-xs font-mono uppercase">Loading Scanned Applications...</p>
            </div>
          ) : filteredApps.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-center p-8 text-zinc-500">
              <AppWindow className="w-8 h-8 text-zinc-700 mb-2" />
              <p className="text-xs font-mono uppercase">No Applications Found</p>
              <p className="text-[11px] text-zinc-600 mt-1">
                No apps matched "{search}".
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-5 gap-2.5">
              {filteredApps.map((app) => (
                <div
                  key={`${app.name}-${app.path}`}
                  onClick={() => handleLaunch(app)}
                  className="group relative flex flex-col items-center justify-center p-3 rounded-xl bg-zinc-900/35 hover:bg-zinc-800/80 border border-zinc-800/60 hover:border-zinc-500 transition-all cursor-pointer shadow-sm hover:shadow-lg hover:shadow-black/50 text-center"
                  title={`${app.name}\nType: ${app.app_type}\nClick to launch`}
                >
                  {/* Subtle Close button visible on card hover */}
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleClose(app);
                    }}
                    title={`Terminate ${app.name}`}
                    className="opacity-0 group-hover:opacity-100 p-1 rounded-md bg-zinc-950/90 hover:bg-red-950/80 hover:text-red-300 hover:border-red-800/60 border border-zinc-800 text-zinc-500 transition-all absolute top-1.5 right-1.5 cursor-pointer z-10"
                  >
                    <Square className="w-2.5 h-2.5" />
                  </button>

                  {/* Real App Icon Container */}
                  <div className="w-12 h-12 rounded-xl bg-zinc-900/70 border border-zinc-800/80 group-hover:border-zinc-500 group-hover:scale-105 flex items-center justify-center mb-2 shadow-md transition-all relative overflow-hidden p-1.5">
                    {app.icon_path ? (
                      <img
                        src={app.icon_path}
                        alt={app.name}
                        className="w-full h-full object-contain filter drop-shadow"
                        onError={(e) => {
                          // If local png fails, hide image and show category icon
                          (e.currentTarget as HTMLElement).style.display = "none";
                          const fallback = e.currentTarget.parentElement?.querySelector(".icon-fallback") as HTMLElement;
                          if (fallback) fallback.style.display = "flex";
                        }}
                      />
                    ) : null}

                    {/* Fallback container when icon_path is absent or fails */}
                    <div
                      className={`icon-fallback w-full h-full flex items-center justify-center ${
                        app.icon_path ? "hidden" : "flex"
                      }`}
                    >
                      {getAppCategoryIcon(app.name)}
                    </div>
                  </div>

                  {/* App Name Only */}
                  <span className="text-[11px] font-medium text-zinc-300 group-hover:text-white line-clamp-2 leading-tight tracking-tight break-words w-full">
                    {app.name}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </section>

      {/* ========================================================================= */}
      {/* RIGHT WINDOW: WORKSPACE GROUPS (MULTI-APP ORCHESTRATION) */}
      {/* ========================================================================= */}
      <section className="flex-1 flex flex-col bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl overflow-hidden shadow-2xl min-w-0 relative">
        {/* Window Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-800/60 shrink-0 bg-zinc-950/40">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-center shadow-md">
              <Layers className="w-4 h-4 text-zinc-300" />
            </div>
            <div>
              <h2 className="text-xs font-mono font-bold tracking-wider text-zinc-100 uppercase">
                Workspace Groups
              </h2>
              <p className="text-[10px] font-mono text-zinc-500 uppercase tracking-widest">
                Multi-App Automation & Workspaces
              </p>
            </div>
          </div>

          {/* Right Header Status Accent */}
          <div className="flex items-center gap-2.5">
            <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-900/90 border border-zinc-800 text-[10px] font-mono text-zinc-400">
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-300 animate-pulse" />
              <span>{groups.length} GROUPS</span>
            </div>
            <div className="h-3 w-[1px] bg-zinc-800" />
            <div className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-700" />
            </div>
          </div>
        </div>

        {/* Content Area: Groups List */}
        <div className="flex-1 overflow-y-auto p-3 flex flex-col gap-3 min-h-0">
          {groups.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center text-center p-8 text-zinc-500">
              <div className="w-12 h-12 rounded-2xl bg-zinc-900/80 border border-zinc-800 flex items-center justify-center mb-3">
                <FolderOpen className="w-6 h-6 text-zinc-600" />
              </div>
              <h3 className="text-xs font-mono font-bold uppercase text-zinc-300 mb-1">
                No Workspace Groups Configured
              </h3>
              <p className="text-[11px] text-zinc-500 max-w-xs leading-relaxed">
                Bundle your most frequently used development, research, or design tools together to launch or close them in a single action.
              </p>
            </div>
          ) : (
            groups.map((group) => (
              <div
                key={group.name}
                className="p-3.5 rounded-xl bg-zinc-900/40 border border-zinc-800/70 hover:border-zinc-700/80 transition-all flex flex-col gap-2.5 shadow-sm"
              >
                {/* Group Title Row */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 min-w-0">
                    <span className="text-xs font-semibold text-zinc-100 font-mono tracking-wide truncate">
                      {group.name}
                    </span>
                    <span className="px-1.5 py-0.5 rounded text-[9px] font-mono bg-zinc-800/80 text-zinc-400 border border-zinc-700/60 shrink-0">
                      {group.apps.length} APPS
                    </span>
                  </div>

                  {/* Actions: Rename, Edit, Delete */}
                  <div className="flex items-center gap-1.5">
                    {/* Rename Group Button */}
                    <button
                      onClick={() => openRenameModal(group)}
                      title={`Rename group "${group.name}"`}
                      className="px-2 py-1 rounded-lg bg-zinc-900/90 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-300 hover:text-white text-[10px] font-mono flex items-center gap-1 transition-colors cursor-pointer"
                    >
                      <Pencil className="w-3 h-3 text-zinc-400" />
                      <span>Rename</span>
                    </button>

                    {/* Edit Group Button (Add/Remove apps) */}
                    <button
                      onClick={() => openEditModal(group)}
                      title={`Edit apps in "${group.name}"`}
                      className="px-2 py-1 rounded-lg bg-zinc-900/90 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-300 hover:text-white text-[10px] font-mono flex items-center gap-1 transition-colors cursor-pointer"
                    >
                      <SlidersHorizontal className="w-3 h-3 text-zinc-400" />
                      <span>Edit</span>
                    </button>

                    {/* Delete Group Button */}
                    <button
                      onClick={() => handleDeleteGroup(group.name)}
                      title={`Delete group "${group.name}"`}
                      className="p-1 rounded text-zinc-500 hover:text-red-400 hover:bg-red-950/40 transition-colors cursor-pointer"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                {/* Description */}
                {group.description && (
                  <p className="text-[11px] text-zinc-500 leading-relaxed font-sans">
                    {group.description}
                  </p>
                )}

                {/* Apps Included in Group */}
                <div className="flex flex-wrap gap-1.5 items-center">
                  {group.apps.map((appName) => {
                    const matchApp = apps.find((a) => a.name.toLowerCase() === appName.toLowerCase());
                    return (
                      <span
                        key={appName}
                        className="group/pill flex items-center gap-1.5 px-2 py-0.5 rounded-lg bg-zinc-950/80 border border-zinc-800 hover:border-zinc-700 text-[10px] font-mono text-zinc-300 relative transition-all"
                      >
                        {matchApp?.icon_path ? (
                          <img src={matchApp.icon_path} alt="" className="w-3.5 h-3.5 object-contain" />
                        ) : (
                          <span className="w-1.5 h-1.5 rounded-full bg-zinc-500" />
                        )}
                        <span className="truncate max-w-[130px]">{appName}</span>
                        {/* 1-click Quick remove from group */}
                        <button
                          onClick={() => handleQuickRemoveApp(group.name, appName)}
                          title={`Remove ${appName} from ${group.name}`}
                          className="opacity-0 group-hover/pill:opacity-100 hover:text-red-400 ml-0.5 transition-opacity cursor-pointer"
                        >
                          <X className="w-2.5 h-2.5" />
                        </button>
                      </span>
                    );
                  })}

                  {/* Quick Add App button */}
                  <button
                    onClick={() => openEditModal(group)}
                    title="Add more apps to this group"
                    className="flex items-center gap-1 px-2 py-0.5 rounded-lg border border-dashed border-zinc-800 hover:border-zinc-600 bg-zinc-950/40 text-[10px] font-mono text-zinc-500 hover:text-zinc-300 transition-colors cursor-pointer"
                  >
                    <Plus className="w-2.5 h-2.5" />
                    <span>Add</span>
                  </button>
                </div>

                {/* Batch Actions Bar */}
                <div className="flex items-center gap-2 pt-1 border-t border-zinc-800/40">
                  <button
                    onClick={() => handleLaunchGroup(group)}
                    className="flex-1 py-1.5 rounded-lg bg-zinc-800/90 hover:bg-zinc-700 border border-zinc-700/80 hover:border-zinc-500 text-zinc-200 hover:text-white text-xs font-mono flex items-center justify-center gap-1.5 transition-all shadow-sm cursor-pointer"
                  >
                    <Play className="w-3 h-3 fill-current" />
                    <span>Launch Group</span>
                  </button>
                  <button
                    onClick={() => handleCloseGroup(group)}
                    className="px-3 py-1.5 rounded-lg bg-zinc-950 hover:bg-zinc-900 border border-zinc-800 hover:border-zinc-700 text-zinc-400 hover:text-zinc-200 text-xs font-mono flex items-center justify-center gap-1 transition-all cursor-pointer"
                    title="Terminate all apps in group"
                  >
                    <Square className="w-3 h-3" />
                    <span>Close</span>
                  </button>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Pinned Bottom Bar: Create Group Button */}
        <div className="p-3 border-t border-zinc-800/60 bg-zinc-950/90 shrink-0">
          <button
            onClick={() => setIsCreatingGroup(true)}
            className="w-full py-2.5 px-4 rounded-xl bg-gradient-to-r from-zinc-900 via-zinc-800 to-zinc-900 hover:from-zinc-800 hover:via-zinc-700 hover:to-zinc-800 border border-zinc-700/80 hover:border-zinc-500 text-zinc-100 font-mono text-xs uppercase tracking-wider flex items-center justify-center gap-2 shadow-lg shadow-black/40 transition-all cursor-pointer group"
          >
            <Plus className="w-4 h-4 text-zinc-300 group-hover:rotate-90 transition-transform duration-300" />
            <span>Create Group</span>
          </button>
        </div>

        {/* ========================================================================= */}
        {/* RENAME GROUP MODAL OVERLAY */}
        {/* ========================================================================= */}
        {renamingGroup && (
          <div className="absolute inset-0 z-40 bg-black/85 backdrop-blur-md p-4 flex flex-col justify-center items-center animate-in fade-in duration-150">
            <div className="w-full max-w-sm bg-zinc-950 border border-zinc-800 rounded-2xl p-4 shadow-2xl flex flex-col gap-3">
              <div className="flex items-center justify-between pb-2 border-b border-zinc-800">
                <div className="flex items-center gap-2">
                  <Pencil className="w-4 h-4 text-zinc-200" />
                  <h3 className="text-xs font-mono font-bold uppercase tracking-wider text-zinc-100">
                    Rename Workspace Group
                  </h3>
                </div>
                <button
                  onClick={() => setRenamingGroup(null)}
                  className="text-zinc-500 hover:text-zinc-200 p-1 rounded-lg hover:bg-zinc-800/60 cursor-pointer"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>

              <div className="flex flex-col gap-1">
                <label className="text-[10px] font-mono text-zinc-400 uppercase tracking-wider">
                  New Group Name
                </label>
                <input
                  type="text"
                  value={renameInputValue}
                  onChange={(e) => setRenameInputValue(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleSaveRename();
                    if (e.key === "Escape") setRenamingGroup(null);
                  }}
                  className="w-full px-3 py-2 rounded-xl bg-zinc-900 border border-zinc-700 text-xs text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:border-zinc-500 font-mono"
                  autoFocus
                />
              </div>

              <div className="flex items-center gap-2 pt-2 border-t border-zinc-800">
                <button
                  onClick={() => setRenamingGroup(null)}
                  className="flex-1 py-1.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-xs font-mono text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSaveRename}
                  className="flex-1 py-1.5 rounded-xl bg-zinc-100 hover:bg-white text-black font-mono text-xs font-bold uppercase tracking-wider transition-all shadow-md cursor-pointer"
                >
                  Rename Group
                </button>
              </div>
            </div>
          </div>
        )}

        {/* ========================================================================= */}
        {/* EDIT GROUP MODAL (ADD MORE APPS OR REMOVE ONE) */}
        {/* ========================================================================= */}
        {editingGroup && (
          <div className="absolute inset-0 z-30 bg-black/85 backdrop-blur-md p-4 flex flex-col animate-in fade-in zoom-in-95 duration-200">
            {/* Modal Header */}
            <div className="flex items-center justify-between pb-3 border-b border-zinc-800 shrink-0">
              <div className="flex items-center gap-2">
                <SlidersHorizontal className="w-4 h-4 text-zinc-200" />
                <h3 className="text-xs font-mono font-bold uppercase tracking-wider text-zinc-100">
                  Edit Group: {editingGroup.name}
                </h3>
              </div>
              <button
                onClick={() => setEditingGroup(null)}
                className="text-zinc-500 hover:text-zinc-200 p-1 rounded-lg hover:bg-zinc-800/60 cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Modal Form Content */}
            <div className="flex-1 overflow-y-auto py-3 flex flex-col gap-3 min-h-0">
              {/* Group Name & Description */}
              <div className="grid grid-cols-2 gap-2">
                <div className="flex flex-col gap-1">
                  <label className="text-[10px] font-mono text-zinc-400 uppercase tracking-wider">
                    Group Name
                  </label>
                  <input
                    type="text"
                    value={editGroupName}
                    onChange={(e) => setEditGroupName(e.target.value)}
                    className="w-full px-3 py-1.5 rounded-xl bg-zinc-900/90 border border-zinc-800 text-xs text-zinc-100 focus:outline-none focus:border-zinc-600 font-mono"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label className="text-[10px] font-mono text-zinc-400 uppercase tracking-wider">
                    Description
                  </label>
                  <input
                    type="text"
                    value={editGroupDesc}
                    onChange={(e) => setEditGroupDesc(e.target.value)}
                    className="w-full px-3 py-1.5 rounded-xl bg-zinc-900/90 border border-zinc-800 text-xs text-zinc-100 focus:outline-none focus:border-zinc-600 font-sans"
                  />
                </div>
              </div>

              {/* Currently Selected Apps (with direct click to remove) */}
              <div className="flex flex-col gap-1.5 bg-zinc-900/40 border border-zinc-800/80 rounded-xl p-2.5">
                <div className="flex items-center justify-between">
                  <span className="text-[10px] font-mono text-zinc-400 uppercase tracking-wider">
                    Included Apps ({editSelectedApps.length})
                  </span>
                  {editSelectedApps.length > 0 && (
                    <span className="text-[9px] font-mono text-zinc-500">
                      Click × to remove
                    </span>
                  )}
                </div>

                <div className="flex flex-wrap gap-1.5 max-h-24 overflow-y-auto">
                  {editSelectedApps.length === 0 ? (
                    <p className="text-[10px] font-mono text-zinc-600 py-1">
                      No apps included. Select apps below to add them.
                    </p>
                  ) : (
                    editSelectedApps.map((appName) => {
                      const matchApp = apps.find((a) => a.name.toLowerCase() === appName.toLowerCase());
                      return (
                        <span
                          key={appName}
                          className="flex items-center gap-1.5 px-2 py-0.5 rounded-lg bg-zinc-950 border border-zinc-700/80 text-[10px] font-mono text-zinc-200 shadow-sm"
                        >
                          {matchApp?.icon_path ? (
                            <img src={matchApp.icon_path} alt="" className="w-3.5 h-3.5 object-contain" />
                          ) : (
                            <span className="w-1.5 h-1.5 rounded-full bg-zinc-500" />
                          )}
                          <span className="truncate max-w-[120px]">{appName}</span>
                          <button
                            onClick={() => toggleEditAppSelection(appName)}
                            title={`Remove ${appName}`}
                            className="text-zinc-500 hover:text-red-400 ml-0.5 cursor-pointer"
                          >
                            <X className="w-3 h-3" />
                          </button>
                        </span>
                      );
                    })
                  )}
                </div>
              </div>

              {/* Add More Apps Checklist */}
              <div className="flex flex-col gap-1.5 flex-1 min-h-[160px]">
                <div className="flex items-center justify-between">
                  <label className="text-[10px] font-mono text-zinc-400 uppercase tracking-wider">
                    Add More Applications
                  </label>
                  <span className="text-[10px] font-mono text-zinc-500">
                    Check or uncheck to toggle
                  </span>
                </div>

                <div className="relative">
                  <Search className="w-3.5 h-3.5 text-zinc-500 absolute left-2.5 top-2.5 pointer-events-none" />
                  <input
                    type="text"
                    value={editPickerSearch}
                    onChange={(e) => setEditPickerSearch(e.target.value)}
                    placeholder="Search apps to add..."
                    className="w-full pl-8 pr-3 py-1.5 rounded-lg bg-zinc-900/80 border border-zinc-800 text-xs text-zinc-200 placeholder:text-zinc-600 focus:outline-none focus:border-zinc-700 font-mono"
                  />
                </div>

                <div className="flex-1 overflow-y-auto border border-zinc-800/80 rounded-xl p-2 bg-zinc-950/60 flex flex-col gap-1 min-h-0">
                  {editPickerFilteredApps.map((app) => {
                    const isSelected = editSelectedApps.includes(app.name);
                    return (
                      <div
                        key={app.name}
                        onClick={() => toggleEditAppSelection(app.name)}
                        className={`flex items-center justify-between p-2 rounded-lg cursor-pointer transition-all ${
                          isSelected
                            ? "bg-zinc-800/90 border border-zinc-600/80 text-white"
                            : "hover:bg-zinc-900/60 border border-transparent text-zinc-400"
                        }`}
                      >
                        <div className="flex items-center gap-2.5 min-w-0 pr-2">
                          <div
                            className={`w-4 h-4 rounded border flex items-center justify-center transition-all ${
                              isSelected
                                ? "bg-zinc-100 border-zinc-100 text-black"
                                : "border-zinc-700 bg-zinc-900"
                            }`}
                          >
                            {isSelected && <Check className="w-3 h-3 stroke-[3]" />}
                          </div>
                          {app.icon_path ? (
                            <img src={app.icon_path} alt="" className="w-4 h-4 object-contain" />
                          ) : null}
                          <span className="text-xs truncate font-medium">{app.name}</span>
                        </div>
                        <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-500 shrink-0">
                          {app.app_type}
                        </span>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>

            {/* Modal Actions Footer */}
            <div className="flex items-center gap-2 pt-3 border-t border-zinc-800 shrink-0">
              <button
                onClick={() => setEditingGroup(null)}
                className="flex-1 py-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-xs font-mono text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveEdit}
                className="flex-1 py-2 rounded-xl bg-zinc-100 hover:bg-white text-black font-mono text-xs font-bold uppercase tracking-wider transition-all shadow-md cursor-pointer"
              >
                Save Changes
              </button>
            </div>
          </div>
        )}

        {/* ========================================================================= */}
        {/* CREATE GROUP MODAL OVERLAY (INSIDE RIGHT WINDOW) */}
        {/* ========================================================================= */}
        {isCreatingGroup && (
          <div className="absolute inset-0 z-30 bg-black/85 backdrop-blur-md p-4 flex flex-col animate-in fade-in zoom-in-95 duration-200">
            {/* Modal Header */}
            <div className="flex items-center justify-between pb-3 border-b border-zinc-800 shrink-0">
              <div className="flex items-center gap-2">
                <Plus className="w-4 h-4 text-zinc-200" />
                <h3 className="text-xs font-mono font-bold uppercase tracking-wider text-zinc-100">
                  Create Workspace Group
                </h3>
              </div>
              <button
                onClick={() => setIsCreatingGroup(false)}
                className="text-zinc-500 hover:text-zinc-200 p-1 rounded-lg hover:bg-zinc-800/60 cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Modal Form Content */}
            <div className="flex-1 overflow-y-auto py-3 flex flex-col gap-3 min-h-0">
              {/* Group Name */}
              <div className="flex flex-col gap-1">
                <label className="text-[10px] font-mono text-zinc-400 uppercase tracking-wider">
                  Group Name *
                </label>
                <input
                  type="text"
                  value={newGroupName}
                  onChange={(e) => setNewGroupName(e.target.value)}
                  placeholder="e.g. Fullstack Dev, Creator Suite, Morning Flow"
                  className="w-full px-3 py-2 rounded-xl bg-zinc-900/90 border border-zinc-800 text-xs text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:border-zinc-600 font-mono"
                  autoFocus
                />
              </div>

              {/* Group Description */}
              <div className="flex flex-col gap-1">
                <label className="text-[10px] font-mono text-zinc-400 uppercase tracking-wider">
                  Description (Optional)
                </label>
                <input
                  type="text"
                  value={newGroupDesc}
                  onChange={(e) => setNewGroupDesc(e.target.value)}
                  placeholder="e.g. Launches code editor, terminal, and browser"
                  className="w-full px-3 py-2 rounded-xl bg-zinc-900/90 border border-zinc-800 text-xs text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:border-zinc-600 font-sans"
                />
              </div>

              {/* App Picker Header & Search */}
              <div className="flex flex-col gap-1.5 flex-1 min-h-[160px]">
                <div className="flex items-center justify-between">
                  <label className="text-[10px] font-mono text-zinc-400 uppercase tracking-wider">
                    Select Applications ({selectedApps.length} Selected) *
                  </label>
                  {selectedApps.length > 0 && (
                    <button
                      onClick={() => setSelectedApps([])}
                      className="text-[10px] font-mono text-zinc-500 hover:text-zinc-300 cursor-pointer"
                    >
                      Clear All
                    </button>
                  )}
                </div>

                <div className="relative">
                  <Search className="w-3.5 h-3.5 text-zinc-500 absolute left-2.5 top-2.5 pointer-events-none" />
                  <input
                    type="text"
                    value={groupPickerSearch}
                    onChange={(e) => setGroupPickerSearch(e.target.value)}
                    placeholder="Search from 174 scanned apps..."
                    className="w-full pl-8 pr-3 py-1.5 rounded-lg bg-zinc-900/80 border border-zinc-800 text-xs text-zinc-200 placeholder:text-zinc-600 focus:outline-none focus:border-zinc-700 font-mono"
                  />
                </div>

                {/* Scanned App Selection Checklist */}
                <div className="flex-1 overflow-y-auto border border-zinc-800/80 rounded-xl p-2 bg-zinc-950/60 flex flex-col gap-1 min-h-0">
                  {pickerFilteredApps.length === 0 ? (
                    <p className="text-[11px] font-mono text-zinc-600 text-center py-4">
                      No matching apps found.
                    </p>
                  ) : (
                    pickerFilteredApps.map((app) => {
                      const isSelected = selectedApps.includes(app.name);
                      return (
                        <div
                          key={app.name}
                          onClick={() => toggleAppSelection(app.name)}
                          className={`flex items-center justify-between p-2 rounded-lg cursor-pointer transition-all ${
                            isSelected
                              ? "bg-zinc-800/90 border border-zinc-600/80 text-white"
                              : "hover:bg-zinc-900/60 border border-transparent text-zinc-400"
                          }`}
                        >
                          <div className="flex items-center gap-2.5 min-w-0 pr-2">
                            <div
                              className={`w-4 h-4 rounded border flex items-center justify-center transition-all ${
                                isSelected
                                  ? "bg-zinc-100 border-zinc-100 text-black"
                                  : "border-zinc-700 bg-zinc-900"
                              }`}
                            >
                              {isSelected && <Check className="w-3 h-3 stroke-[3]" />}
                            </div>
                            {app.icon_path ? (
                              <img src={app.icon_path} alt="" className="w-4 h-4 object-contain" />
                            ) : null}
                            <span className="text-xs truncate font-medium">{app.name}</span>
                          </div>
                          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-500 shrink-0">
                            {app.app_type}
                          </span>
                        </div>
                      );
                    })
                  )}
                </div>
              </div>
            </div>

            {/* Modal Actions Footer */}
            <div className="flex items-center gap-2 pt-3 border-t border-zinc-800 shrink-0">
              <button
                onClick={() => setIsCreatingGroup(false)}
                className="flex-1 py-2 rounded-xl bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-xs font-mono text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveGroup}
                className="flex-1 py-2 rounded-xl bg-zinc-100 hover:bg-white text-black font-mono text-xs font-bold uppercase tracking-wider transition-all shadow-md cursor-pointer"
              >
                Save Workspace Group
              </button>
            </div>
          </div>
        )}
      </section>
    </div>
  );
};
