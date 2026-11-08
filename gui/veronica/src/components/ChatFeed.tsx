import React, { useEffect, useRef } from "react";
import { ChatMessage } from "../types/protocol";

interface Props {
  messages: ChatMessage[];
}

export const ChatFeed: React.FC<Props> = ({ messages }) => {
  const bottomRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  if (messages.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-6 text-center text-zinc-500 font-mono text-xs">
        <p>No messages yet.</p>
        <p className="text-zinc-600 mt-1">
          Speak naturally into your microphone or type below.
        </p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4 font-sans text-sm scrollbar-thin scrollbar-thumb-zinc-800 scrollbar-track-transparent">
      {messages.map((msg) => {
        const isBot = msg.speaker === "VERONICA";

        return (
          <div
            key={msg.id}
            className={`flex ${isBot ? "justify-start" : "justify-end"}`}
          >
            <div
              className={`max-w-[85%] rounded-2xl px-4 py-2.5 shadow-sm leading-relaxed ${
                isBot
                  ? "bg-zinc-900/90 border border-zinc-800 text-zinc-100 rounded-tl-sm"
                  : "bg-zinc-800 border border-zinc-700/80 text-zinc-100 rounded-tr-sm"
              }`}
            >
              <div className="flex items-center justify-between gap-4 mb-1 text-[10px] font-mono opacity-60">
                <span>{isBot ? "Veronica" : "You"}</span>
                <span>
                  {msg.timestamp.toLocaleTimeString([], {
                    hour: "2-digit",
                    minute: "2-digit",
                    second: "2-digit",
                  })}
                </span>
              </div>
              <p className="whitespace-pre-wrap select-text">{msg.text}</p>
            </div>
          </div>
        );
      })}
      <div ref={bottomRef} />
    </div>
  );
};
