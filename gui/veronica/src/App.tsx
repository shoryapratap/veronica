import { StatusBar } from "./components/StatusBar";
import { AuraOrb } from "./components/AuraOrb";
import { ChatFeed } from "./components/ChatFeed";
import { InputBar } from "./components/InputBar";
import { useDaemonSocket } from "./hooks/useDaemonSocket";
import "./App.css";

function App() {
  const { status, assistantState, cpu, ram, messages, sendTextMessage } =
    useDaemonSocket();

  const isOnline = status === "ONLINE";

  return (
    <div className="flex flex-col h-screen bg-zinc-950 text-zinc-100 font-sans selection:bg-indigo-500/30 select-none overflow-hidden">
      {/* 1. Top System & Connection Status Bar */}
      <StatusBar status={status} cpu={cpu} ram={ram} />

      {/* 2. Visual AI Presence Orb */}
      <div className="border-b border-zinc-900/60 bg-gradient-to-b from-zinc-950 via-zinc-950/90 to-zinc-900/20 py-2">
        <AuraOrb state={assistantState} />
      </div>

      {/* 3. Real-Time Conversation Stream */}
      <main className="flex-1 flex flex-col min-h-0 bg-zinc-950/50">
        <ChatFeed messages={messages} />
      </main>

      {/* 4. Desktop Text Input Bar */}
      <InputBar onSendMessage={sendTextMessage} disabled={!isOnline} />
    </div>
  );
}

export default App;
