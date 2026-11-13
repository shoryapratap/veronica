import React, { useState } from "react";
import { Terminal, Mic, CornerDownLeft } from "lucide-react";

interface Props {
  onSendMessage: (text: string) => boolean;
  disabled?: boolean;
}

export const InputBar: React.FC<Props> = ({ onSendMessage, disabled = false }) => {
  const [input, setInput] = useState("");
  const [isListening, setIsListening] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = input.trim();
    if (!trimmed || disabled) return;

    if (onSendMessage(trimmed)) {
      setInput("");
    }
  };

  const toggleMic = () => {
    setIsListening((prev) => !prev);
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="bg-zinc-950/70 backdrop-blur-xl border border-zinc-800/60 rounded-2xl p-2.5 flex items-center gap-2.5 shadow-2xl shrink-0 group focus-within:border-zinc-700/80 transition-all"
    >
      <div className="pl-2 text-zinc-500 flex items-center gap-1.5 shrink-0 select-none">
        <Terminal className="w-4 h-4 text-zinc-400" />
      </div>

      <div className="relative flex-1">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={
            disabled
              ? "Connecting to Veronica Core Engine..."
              : isListening
              ? "Listening for voice command..."
              : "Ask Veronica anything, or type a system command..."
          }
          disabled={disabled}
          className="w-full bg-transparent border-none py-1.5 text-sm text-zinc-100 placeholder:text-zinc-600 focus:outline-none font-sans disabled:opacity-50"
        />
      </div>

      <div className="flex items-center gap-2 shrink-0">
        {/* Microphone Button */}
        <button
          type="button"
          onClick={toggleMic}
          disabled={disabled}
          title={isListening ? "Mute Microphone" : "Activate Voice Command"}
          className={`px-3 py-2 rounded-xl flex items-center justify-center transition-all border cursor-pointer ${
            isListening
              ? "bg-zinc-800 text-white border-zinc-400 shadow-[0_0_15px_rgba(255,255,255,0.2)] animate-pulse"
              : "bg-zinc-900/80 hover:bg-zinc-800 border-zinc-700/60 text-zinc-400 hover:text-zinc-100 shadow-md"
          }`}
        >
          <Mic className="w-3.5 h-3.5" />
        </button>

        {/* Enter / Send Button */}
        <button
          type="submit"
          disabled={disabled || !input.trim()}
          className="px-3 py-1.5 bg-zinc-900/90 hover:bg-zinc-800 disabled:bg-zinc-950/60 text-zinc-300 hover:text-white disabled:text-zinc-600 rounded-xl flex items-center gap-1.5 transition-all border border-zinc-800 hover:border-zinc-700 disabled:border-zinc-900 shadow-lg shadow-black/30 cursor-pointer disabled:cursor-not-allowed font-mono text-xs uppercase tracking-wider"
        >
          <CornerDownLeft className="w-3.5 h-3.5 text-zinc-500 group-hover:text-zinc-300" />
          <span>ENTER</span>
        </button>
      </div>
    </form>
  );
};
