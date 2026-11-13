import { useState } from "react";
import { StatusBar, TabType } from "./components/StatusBar";
import { AuraOrb } from "./components/AuraOrb";
import { ChatFeed } from "./components/ChatFeed";
import { InputBar } from "./components/InputBar";
import { FeedPlaceholders } from "./components/FeedPlaceholders";
import { MainWindow } from "./components/MainWindow";
import { EarthGlobe } from "./components/EarthGlobe";
import { AppsPage } from "./components/AppsPage";
import { SettingsPage } from "./components/SettingsPage";
import { useDaemonSocket } from "./hooks/useDaemonSocket";
import { MessageSquare } from "lucide-react";
import "./App.css";

function App() {
  const { status, assistantState, cpu, ram, messages, sendTextMessage } =
    useDaemonSocket();

  const [activeTab, setActiveTab] = useState<TabType>("main");
  const isOnline = status === "ONLINE";

  const handleLaunchApp = (target: string) => {
    sendTextMessage(`launch ${target}`);
  };

  const handleCloseApp = (target: string) => {
    sendTextMessage(`close ${target}`);
  };

  const handleSwitchModel = (newModel: string) => {
    sendTextMessage(`/model ${newModel}`);
  };

  return (
    <div className="flex flex-col h-screen w-screen bg-black text-zinc-100 font-sans selection:bg-zinc-700/50 select-none overflow-hidden">
      {/* 1. Top System Bar with Brand, Navbar (Main / Apps / Settings), and Telemetry */}
      <StatusBar
        status={status}
        cpu={cpu}
        ram={ram}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />

      {/* 2. Page Content Router */}
      {activeTab === "main" && (
        <div className="flex-1 flex overflow-hidden p-3 gap-3 min-h-0">
          {/* ======================================================== */}
          {/* LEFT SECTION: 2 Small Windows (Screen/Cam) + Chat Feed */}
          {/* ======================================================== */}
          <section className="w-[340px] xl:w-[380px] 2xl:w-[420px] flex flex-col gap-3 shrink-0 h-full">
            {/* Top: 2 Small Windows (Screen Sharing & Camera) */}
            <FeedPlaceholders />

            {/* Bottom: Chat Section (Expanded Feed) */}
            <div className="flex-1 flex flex-col bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-xl overflow-hidden min-h-0 shadow-xl">
              {/* Chat Header */}
              <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-zinc-800/60 shrink-0">
                <div className="flex items-center gap-2">
                  <MessageSquare className="w-3.5 h-3.5 text-zinc-400" />
                  <span className="text-xs font-mono font-medium text-zinc-200 tracking-wider uppercase">
                    Communication Log
                  </span>
                </div>
                <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-zinc-900 border border-zinc-800 text-[10px] text-zinc-400 font-mono">
                  <span className="w-1.5 h-1.5 rounded-full bg-zinc-300 animate-pulse" />
                  <span>{messages.length} MSG</span>
                </div>
              </div>

              {/* Chat Feed */}
              <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
                <ChatFeed messages={messages} />
              </div>
            </div>
          </section>

          {/* ======================================================== */}
          {/* MID SECTION: AI Orb + Main Window + Text Input Bar (Middle) */}
          {/* ======================================================== */}
          <section className="flex-1 flex flex-col gap-3 min-w-0 h-full">
            {/* Top: Aura Orb Center Stage */}
            <div className="bg-zinc-950/70 backdrop-blur-xl rounded-2xl py-3 px-6 shrink-0 relative overflow-hidden flex flex-col items-center justify-center shadow-xl">
              {/* Visual AI Presence Orb */}
              <AuraOrb state={assistantState} />
            </div>

            {/* Middle: Empty Main Window (Clean Canvas) */}
            <MainWindow />

            {/* Bottom: Text Input Bar (Centered in the middle section) */}
            <InputBar onSendMessage={sendTextMessage} disabled={!isOnline} />
          </section>

          {/* ======================================================== */}
          {/* RIGHT SECTION: 3D Earth Globe */}
          {/* ======================================================== */}
          <section className="w-[340px] xl:w-[380px] 2xl:w-[420px] flex flex-col shrink-0 h-full">
            <EarthGlobe />
          </section>
        </div>
      )}

      {/* 3. Apps Page View */}
      {activeTab === "apps" && (
        <AppsPage
          onLaunchApp={handleLaunchApp}
          onCloseApp={handleCloseApp}
        />
      )}

      {/* 4. Settings Page View */}
      {activeTab === "settings" && <SettingsPage onSwitchModel={handleSwitchModel} />}
    </div>
  );
}

export default App;
