import { useCallback, useEffect, useRef, useState } from "react";
import type { ClientMsg, ServerMsg } from "../types/game";

type Status = "connecting" | "open" | "closed" | "error";

interface UseWebSocketReturn {
  status: Status;
  lastMessage: ServerMsg | null;
  send: (msg: ClientMsg) => void;
}

const WS_URL = import.meta.env.VITE_WS_URL ?? "ws://localhost:3000/ws";
const RECONNECT_DELAY_MS = 2000;

export function useWebSocket(): UseWebSocketReturn {
  const ws = useRef<WebSocket | null>(null);
  const [status, setStatus] = useState<Status>("connecting");
  const [lastMessage, setLastMessage] = useState<ServerMsg | null>(null);

  const connect = useCallback(() => {
    if (ws.current?.readyState === WebSocket.OPEN) return;

    const socket = new WebSocket(WS_URL);
    ws.current = socket;

    socket.onopen = () => {
      setStatus("open");
    };

    socket.onmessage = (event: MessageEvent<string>) => {
      try {
        const msg = JSON.parse(event.data) as ServerMsg;
        setLastMessage(msg);
      } catch {
        console.warn("Failed to parse server message:", event.data);
      }
    };

    socket.onclose = () => {
      setStatus("closed");
      // Auto-reconnect
      setTimeout(connect, RECONNECT_DELAY_MS);
    };

    socket.onerror = () => {
      setStatus("error");
      socket.close();
    };
  }, []);

  useEffect(() => {
    connect();
    return () => {
      ws.current?.close();
    };
  }, [connect]);

  const send = useCallback((msg: ClientMsg) => {
    if (ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(msg));
    } else {
      console.warn("WebSocket not open; message dropped:", msg);
    }
  }, []);

  return { status, lastMessage, send };
}
