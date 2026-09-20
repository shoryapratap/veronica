import React, { useState } from "react";
import { Send } from "lucide-react";

interface Props {
  onSendMessage: (text: string) => boolean;
  disabled?: boolean;
}

export const InputBar: React.FC<Props> = ({ onSendMessage, disabled = false }) => {
  const [input, setInput] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = input.trim();
    if (!trimmed || disabled) return;

    if (onSendMessage(trimmed)) {
      setInput("");
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="p-4 bg-zinc-950/80 backdrop-blur-md border-t border-zinc-900 flex items-center gap-2"
    >
      <div className="relative flex-1">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={
            disabled
              ? "Connecting to Veronica Daemon..."
              : "Ask Veronica anything, or type a command..."
          }
          disabled={disabled}
          className="w-full bg-zinc-900/80 border border-zinc-800 focus:border-indigo-500 rounded-xl px-4 py-2.5 text-sm text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:ring-1 focus:ring-indigo-500/50 transition-all font-sans disabled:opacity-50"
        />
      </div>

      <button
        type="submit"
        disabled={disabled || !input.trim()}
        className="px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-800 text-white rounded-xl flex items-center justify-center transition-colors shadow-lg shadow-indigo-600/20 disabled:shadow-none shrink-0"
      >
        <Send className="w-4 h-4" />
      </button>
    </form>
  );
};
