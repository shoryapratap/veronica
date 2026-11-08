import { useState, useEffect, useRef, useCallback } from "react";
import {
  ConnectionStatus,
  AssistantState,
  DaemonMessage,
  ClientMessage,
  ChatMessage,
} from "../types/protocol";

const DAEMON_WS_URL = "ws://127.0.0.1:8765";
const RECONNECT_INTERVAL_MS = 2000;

export function useDaemonSocket() {
  const [status, setStatus] = useState<ConnectionStatus>("CONNECTING");
  const [assistantState, setAssistantState] = useState<AssistantState>("IDLE");
  const [cpu, setCpu] = useState<number>(0);
  const [ram, setRam] = useState<string>("0 / 0 MB");
  const [messages, setMessages] = useState<ChatMessage[]>([]);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);

  const connect = useCallback(() => {
    setStatus("CONNECTING");

    try {
      const ws = new WebSocket(DAEMON_WS_URL);
      wsRef.current = ws;

      ws.onopen = () => {
        setStatus("ONLINE");
        // Send initial ping
        const ping: ClientMessage = { type: "PING" };
        ws.send(JSON.stringify(ping));
      };

      ws.onmessage = (event) => {
        try {
          const msg: DaemonMessage = JSON.parse(event.data);
          handleDaemonMessage(msg);
        } catch (err) {
          console.error("Failed to parse daemon packet:", err);
        }
      };

      ws.onclose = () => {
        setStatus("DISCONNECTED");
        setAssistantState("IDLE");
        wsRef.current = null;
        scheduleReconnect();
      };

      ws.onerror = () => {
        setStatus("DISCONNECTED");
        ws.close();
      };
    } catch {
      setStatus("DISCONNECTED");
      scheduleReconnect();
    }
  }, []);

  const scheduleReconnect = useCallback(() => {
    if (reconnectTimeoutRef.current !== null) {
      window.clearTimeout(reconnectTimeoutRef.current);
    }
    reconnectTimeoutRef.current = window.setTimeout(() => {
      connect();
    }, RECONNECT_INTERVAL_MS);
  }, [connect]);

  const handleDaemonMessage = (msg: DaemonMessage) => {
    switch (msg.type) {
      case "HEARTBEAT":
        setStatus("ONLINE");
        break;

      case "TELEMETRY":
        setCpu(msg.payload.cpu_usage);
        setRam(`${msg.payload.ram_used_mb} / ${msg.payload.ram_total_mb} MB`);
        break;

      case "STATE_CHANGE":
        setAssistantState(msg.payload.state);
        break;

      case "TRANSCRIPT": {
        const { speaker, text } = msg.payload;
        if (!text) break;

        setMessages((prev) => {
          const lastMsg = prev[prev.length - 1];
          // If the last message has the same speaker within 4s, append text
          if (
            lastMsg &&
            lastMsg.speaker === speaker &&
            Date.now() - lastMsg.timestamp.getTime() < 4000
          ) {
            return [
              ...prev.slice(0, -1),
              {
                ...lastMsg,
                text: lastMsg.text + text,
              },
            ];
          }

          // Otherwise create a new turn
          return [
            ...prev,
            {
              id: Math.random().toString(36).substring(2, 9),
              speaker,
              text,
              timestamp: new Date(),
            },
          ];
        });
        break;
      }

      default:
        break;
    }
  };

  const sendTextMessage = useCallback((text: string) => {
    const trimmed = text.trim();
    if (!trimmed || !wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
      return false;
    }

    const clientMsg: ClientMessage = {
      type: "TEXT_MESSAGE",
      payload: { text: trimmed },
    };

    wsRef.current.send(JSON.stringify(clientMsg));

    // Optimistically record user text
    setMessages((prev) => [
      ...prev,
      {
        id: Math.random().toString(36).substring(2, 9),
        speaker: "USER",
        text: trimmed,
        timestamp: new Date(),
      },
    ]);

    return true;
  }, []);

  useEffect(() => {
    connect();
    return () => {
      if (reconnectTimeoutRef.current !== null) {
        window.clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connect]);

  return {
    status,
    assistantState,
    cpu,
    ram,
    messages,
    sendTextMessage,
  };
}
