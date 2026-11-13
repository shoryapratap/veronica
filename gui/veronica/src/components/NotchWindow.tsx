import React, { useState, useEffect, useRef } from "react";
import {
  Zap,
  Mic,
  MicOff,
  ArrowUp,
  Maximize2,
  ChevronUp,
  Sparkles,
  Bot,
  User,
  Volume2,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useDaemonSocket } from "../hooks/useDaemonSocket";

export const NotchWindow: React.FC = () => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [time, setTime] = useState("");
  const [battery, setBattery] = useState<{ percentage: number; isCharging: boolean }>({
    percentage: 100,
    isCharging: true,
  });

  // Daemon WebSocket hook for real-time AI states, transcripts, and sending text/voice
  const { status, assistantState, messages, sendTextMessage } = useDaemonSocket();

  // Voice speech-to-text state
  const [isListening, setIsListening] = useState(false);
  const [inputText, setInputText] = useState("");
  const recognitionRef = useRef<any>(null);
  const speechTextRef = useRef<string>("");
  const chatEndRef = useRef<HTMLDivElement>(null);
  const inputInputRef = useRef<HTMLInputElement>(null);

  // 1. Live 12-Hour Clock
  useEffect(() => {
    const updateTime = () => {
      const now = new Date();
      setTime(
        now.toLocaleTimeString([], {
          hour: "numeric",
          minute: "2-digit",
          hour12: true,
        })
      );
    };
    updateTime();
    const interval = setInterval(updateTime, 1000);
    return () => clearInterval(interval);
  }, []);

  // 2. Live Battery Telemetry
  useEffect(() => {
    const fetchBatteryTauri = async () => {
      try {
        const info = await invoke<{ percentage: number; is_charging: boolean }>(
          "get_battery_info"
        );
        if (info) {
          setBattery({
            percentage: info.percentage,
            isCharging: info.is_charging,
          });
        }
      } catch {
        // web fallback
      }
    };

    fetchBatteryTauri();
    const interval = setInterval(fetchBatteryTauri, 5000);

    if ("getBattery" in navigator) {
      (navigator as any).getBattery().then((batt: any) => {
        const update = () => {
          setBattery({
            percentage: Math.round(batt.level * 100),
            isCharging: batt.charging,
          });
        };
        update();
        batt.addEventListener("levelchange", update);
        batt.addEventListener("chargingchange", update);
      });
    }

    return () => clearInterval(interval);
  }, []);

  // 3. Auto scroll chat feed to bottom
  useEffect(() => {
    if (isExpanded) {
      chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages, isExpanded, assistantState]);

  // 4. Focus input when expanded
  useEffect(() => {
    if (isExpanded) {
      setTimeout(() => {
        inputInputRef.current?.focus();
      }, 100);
    }
  }, [isExpanded]);

  // 5. Expand Notch into Voice/Text Small Window
  const handleExpand = async () => {
    try {
      await invoke("set_notch_expanded", { expanded: true });
      setIsExpanded(true);
      // Auto-start listening for voice on expand if SpeechRecognition is available
      startListening();
    } catch {
      setIsExpanded(true);
    }
  };

  // 6. Collapse back to slim notch
  const handleCollapse = async () => {
    stopListening();
    try {
      await invoke("set_notch_expanded", { expanded: false });
      setIsExpanded(false);
    } catch {
      setIsExpanded(false);
    }
  };

  // 7. Restore full main Veronica HUD window
  const handleOpenFullApp = async () => {
    stopListening();
    try {
      await invoke("restore_main_window");
    } catch {
      // fallback
    }
  };

  // 8. Speech Recognition (Web Speech API with graceful fallback to Daemon)
  const startListening = () => {
    const SpeechRecognition =
      (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;

    if (!SpeechRecognition) {
      setIsListening(true);
      return;
    }

    try {
      if (recognitionRef.current) {
        recognitionRef.current.abort();
      }

      const recognition = new SpeechRecognition();
      recognition.continuous = false;
      recognition.interimResults = true;
      recognition.lang = "en-US";

      recognition.onstart = () => {
        setIsListening(true);
        speechTextRef.current = "";
      };

      recognition.onresult = (event: any) => {
        let transcript = "";
        for (let i = event.resultIndex; i < event.results.length; i++) {
          transcript += event.results[i][0].transcript;
        }
        speechTextRef.current = transcript;
        setInputText(transcript);
      };

      recognition.onerror = () => {
        setIsListening(false);
      };

      recognition.onend = () => {
        setIsListening(false);
        const finalSpoken = speechTextRef.current.trim();
        if (finalSpoken) {
          sendTextMessage(finalSpoken);
          setInputText("");
          speechTextRef.current = "";
        }
      };

      recognitionRef.current = recognition;
      recognition.start();
    } catch {
      setIsListening(false);
    }
  };

  const stopListening = () => {
    if (recognitionRef.current) {
      try {
        recognitionRef.current.stop();
      } catch {
        // ignore
      }
    }
    setIsListening(false);
  };

  const toggleListening = () => {
    if (isListening) {
      stopListening();
    } else {
      startListening();
    }
  };

  const handleSend = () => {
    const text = inputText.trim();
    if (!text) return;
    sendTextMessage(text);
    setInputText("");
    speechTextRef.current = "";
    if (isListening) {
      stopListening();
    }
  };

  // =========================================================================
  // Render: Collapsed Dynamic Island Notch
  // =========================================================================
  if (!isExpanded) {
    return (
      <div
        onClick={handleExpand}
        title="Click to interact with Veronica"
        className="w-full h-full flex justify-center items-start select-none cursor-pointer group bg-transparent overflow-hidden"
      >
        <div className="relative w-[234px] h-[32px] flex items-center justify-between px-4 transition-all duration-200 group-hover:brightness-110 active:scale-[0.98]">
          {/* SVG Custom Curved Notch with concave top shoulders matching the screenshot */}
          <svg
            className="absolute inset-0 w-full h-full pointer-events-none drop-shadow-[0_8px_16px_rgba(0,0,0,0.9)]"
            viewBox="0 0 234 32"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M 0 0 C 8 0 12 7 13 14 L 13 16 C 13 25 19 32 28 32 L 206 32 C 215 32 221 25 221 16 L 221 14 C 222 7 226 0 234 0 Z"
              fill="#000000"
            />
          </svg>

          {/* Left: Clock Time */}
          <div className="relative z-10 flex items-center pl-0.5">
            <span className="text-[11.5px] font-sans font-medium text-zinc-100 tracking-tight leading-none">
              {time || "12:00 PM"}
            </span>
          </div>

          {/* Right: Charging Bolt, Battery Capsule with internal Bolt, and Percentage */}
          <div className="relative z-10 flex items-center gap-1.5 pr-0.5">
            {battery.isCharging && (
              <Zap className="w-3 h-3 text-[#22c55e] fill-[#22c55e] shrink-0" />
            )}

            <div className="relative flex items-center">
              <div className="w-[21px] h-[11px] rounded-[3px] bg-[#22c55e] flex items-center justify-center p-[1px] shadow-sm">
                <Zap className="w-2.5 h-2.5 text-black fill-black stroke-[1.5]" />
              </div>
              <div className="w-[1.5px] h-[4.5px] bg-[#22c55e] rounded-r-xs absolute -right-[2px]" />
            </div>

            <span className="text-[11.5px] font-sans font-semibold text-zinc-100 tracking-tight leading-none">
              {battery.percentage}
            </span>
          </div>
        </div>
      </div>
    );
  }

  // =========================================================================
  // Render: Expanded Small Voice & Text AI Window
  // =========================================================================
  const isListeningActive = isListening || assistantState === "LISTENING";
  const isSpeaking = assistantState === "SPEAKING";
  const isThinking =
    messages.length > 0 &&
    messages[messages.length - 1].speaker === "USER" &&
    !isSpeaking;

  return (
    <div className="w-full h-full flex flex-col items-center justify-start p-1.5 bg-transparent select-none overflow-hidden font-sans">
      <div className="w-[464px] h-[244px] bg-[#09090b]/96 backdrop-blur-2xl border border-zinc-800/90 rounded-[22px] shadow-[0_20px_50px_rgba(0,0,0,0.95)] flex flex-col overflow-hidden text-zinc-100">
        {/* Top Header */}
        <div className="px-3.5 py-2 border-b border-zinc-800/70 flex items-center justify-between shrink-0 bg-zinc-950/60">
          <div className="flex items-center gap-2">
            {/* Status Aura Glow Dot */}
            <span className="relative flex h-2 w-2">
              <span
                className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${
                  isListeningActive
                    ? "bg-emerald-400"
                    : isSpeaking
                    ? "bg-violet-400"
                    : "bg-zinc-400"
                }`}
              />
              <span
                className={`relative inline-flex rounded-full h-2 w-2 ${
                  isListeningActive
                    ? "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]"
                    : isSpeaking
                    ? "bg-violet-500 shadow-[0_0_8px_rgba(139,92,246,0.8)]"
                    : "bg-zinc-300 shadow-[0_0_8px_rgba(255,255,255,0.4)]"
                }`}
              />
            </span>

            <span className="font-semibold text-xs tracking-wider text-white">VERONICA</span>

            {/* State Badge */}
            <span
              className={`px-2 py-0.5 rounded-full text-[10px] font-mono font-medium border ${
                isListeningActive
                  ? "bg-emerald-950/60 text-emerald-300 border-emerald-500/30"
                  : isSpeaking
                  ? "bg-violet-950/60 text-violet-300 border-violet-500/30"
                  : "bg-zinc-900/80 text-zinc-400 border-zinc-800"
              }`}
            >
              {isListeningActive
                ? "LISTENING"
                : isSpeaking
                ? "SPEAKING"
                : status === "ONLINE"
                ? "ONLINE"
                : "STANDBY"}
            </span>
          </div>

          {/* Right Header Controls */}
          <div className="flex items-center gap-2">
            <span className="text-[11px] font-mono text-zinc-400">{time}</span>

            {/* Expand to Full App Button */}
            <button
              onClick={handleOpenFullApp}
              title="Open full Veronica application HUD"
              className="p-1 rounded-md text-zinc-400 hover:text-white hover:bg-zinc-800/80 transition-colors cursor-pointer"
            >
              <Maximize2 className="w-3.5 h-3.5" />
            </button>

            {/* Collapse back to notch */}
            <button
              onClick={handleCollapse}
              title="Collapse to notch"
              className="p-1 rounded-md text-zinc-400 hover:text-white hover:bg-zinc-800/80 transition-colors cursor-pointer"
            >
              <ChevronUp className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* Middle Dialogue Feed */}
        <div className="flex-1 p-3 overflow-y-auto space-y-2 text-xs scrollbar-thin scrollbar-thumb-zinc-700/40">
          {messages.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-center py-4 text-zinc-400 space-y-2">
              <div
                className={`w-9 h-9 rounded-full flex items-center justify-center transition-all ${
                  isListeningActive
                    ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 animate-pulse"
                    : "bg-zinc-900/80 text-zinc-400 border border-zinc-800"
                }`}
              >
                {isListeningActive ? (
                  <Mic className="w-4 h-4 animate-bounce" />
                ) : (
                  <Sparkles className="w-4 h-4 text-zinc-300" />
                )}
              </div>
              <p className="text-zinc-300 font-medium text-[12px]">
                {isListeningActive
                  ? "Listening to your voice..."
                  : "Say something or type a request"}
              </p>
              <p className="text-[10px] text-zinc-500 max-w-[320px]">
                Ask questions, control apps, launch tools, or automate desktop actions.
              </p>
            </div>
          ) : (
            messages.slice(-4).map((msg) => (
              <div
                key={msg.id}
                className={`flex gap-2 ${
                  msg.speaker === "USER" ? "justify-end" : "justify-start"
                }`}
              >
                {msg.speaker !== "USER" && (
                  <div className="w-5 h-5 rounded-full bg-zinc-800 border border-zinc-700/60 flex items-center justify-center text-zinc-300 shrink-0 mt-0.5">
                    <Bot className="w-3 h-3" />
                  </div>
                )}
                <div
                  className={`max-w-[85%] rounded-xl px-3 py-1.5 leading-relaxed ${
                    msg.speaker === "USER"
                      ? "bg-zinc-800/90 text-zinc-100 border border-zinc-700/60 rounded-tr-none text-right"
                      : "bg-zinc-900/80 text-zinc-200 border border-zinc-800/80 rounded-tl-none text-left"
                  }`}
                >
                  <p className="text-[11.5px] whitespace-pre-wrap">{msg.text}</p>
                </div>
                {msg.speaker === "USER" && (
                  <div className="w-5 h-5 rounded-full bg-zinc-700/60 flex items-center justify-center text-zinc-300 shrink-0 mt-0.5">
                    <User className="w-3 h-3" />
                  </div>
                )}
              </div>
            ))
          )}

          {/* Thinking indicator */}
          {isThinking && (
            <div className="flex items-center gap-2 text-cyan-400 text-[11px] pt-1">
              <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-ping" />
              <span>Veronica is thinking...</span>
            </div>
          )}

          {/* Speaking indicator */}
          {isSpeaking && (
            <div className="flex items-center gap-2 text-violet-400 text-[11px] pt-1">
              <Volume2 className="w-3.5 h-3.5 animate-pulse" />
              <span>Veronica is speaking...</span>
            </div>
          )}

          <div ref={chatEndRef} />
        </div>

        {/* Bottom Input & Voice Control Bar */}
        <div className="p-2 border-t border-zinc-800/70 bg-zinc-950/70 flex items-center gap-2 shrink-0">
          {/* Mic Toggle Button */}
          <button
            onClick={toggleListening}
            title={isListeningActive ? "Stop listening" : "Click to speak"}
            className={`p-2 rounded-xl transition-all cursor-pointer flex items-center justify-center ${
              isListeningActive
                ? "bg-emerald-500/25 text-emerald-300 border border-emerald-500/50 shadow-[0_0_12px_rgba(16,185,129,0.3)] animate-pulse"
                : "bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-800"
            }`}
          >
            {isListeningActive ? (
              <Mic className="w-3.5 h-3.5 text-emerald-400" />
            ) : (
              <MicOff className="w-3.5 h-3.5" />
            )}
          </button>

          {/* Text Input */}
          <input
            ref={inputInputRef}
            type="text"
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                handleSend();
              }
            }}
            placeholder={
              isListeningActive
                ? "Listening... Speak your prompt"
                : "Ask or instruct Veronica..."
            }
            className="flex-1 bg-zinc-900/90 text-zinc-100 placeholder-zinc-500 px-3 py-1.5 rounded-xl border border-zinc-800 focus:outline-none focus:border-zinc-600 text-xs transition-colors"
          />

          {/* Send Button */}
          <button
            onClick={handleSend}
            disabled={!inputText.trim()}
            title="Send prompt to Veronica"
            className="p-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 disabled:opacity-40 disabled:hover:bg-zinc-800 text-zinc-200 transition-all cursor-pointer flex items-center justify-center shrink-0"
          >
            <ArrowUp className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
};
