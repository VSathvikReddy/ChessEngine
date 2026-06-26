import type { SearchInfoMsg } from "../types/game";

interface TelemetryProps {
  info: SearchInfoMsg | null;
  aiEval: number | null;
  status: "connecting" | "open" | "closed" | "error";
}

export function Telemetry({ info, aiEval, status }: TelemetryProps) {
  const statusColor: Record<string, string> = {
    open: "#4caf50",
    connecting: "#ff9800",
    closed: "#9e9e9e",
    error: "#f44336",
  };

  const evalBar = aiEval !== null ? Math.max(-1000, Math.min(1000, aiEval)) : 0;
  const whitePercent = 50 + (evalBar / 1000) * 50;

  return (
    <div style={{
      background: "#1a1a2e",
      color: "#e0e0e0",
      borderRadius: 8,
      padding: "16px 20px",
      minWidth: 220,
      fontFamily: "monospace",
      fontSize: 13,
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 16 }}>
        <div style={{
          width: 8, height: 8, borderRadius: "50%",
          background: statusColor[status] ?? "#9e9e9e",
        }} />
        <span style={{ color: "#aaa", fontSize: 11 }}>
          WebSocket {status}
        </span>
      </div>

      <div style={{ marginBottom: 16 }}>
        <div style={{ color: "#aaa", fontSize: 11, marginBottom: 4 }}>Evaluation</div>
        <div style={{
          height: 16,
          borderRadius: 4,
          background: "#333",
          overflow: "hidden",
          position: "relative",
        }}>
          <div style={{
            position: "absolute",
            left: 0,
            width: `${whitePercent}%`,
            height: "100%",
            background: "#f0d9b5",
            transition: "width 0.3s ease",
          }} />
        </div>
        {aiEval !== null && (
          <div style={{ textAlign: "center", marginTop: 4, fontSize: 12 }}>
            {aiEval > 0 ? "+" : ""}{(aiEval / 100).toFixed(2)}
          </div>
        )}
      </div>

      {info ? (
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "6px 12px" }}>
          <Stat label="Depth" value={String(info.depth)} />
          <Stat label="Nodes" value={fmtNum(info.nodes)} />
          <Stat label="TT hits" value={fmtNum(info.tt_hits)} />
          <Stat label="Time" value={`${info.elapsed_ms}ms`} />
          <Stat
            label="Nodes/s"
            value={info.elapsed_ms > 0
              ? fmtNum(Math.round((info.nodes / info.elapsed_ms) * 1000))
              : "—"}
          />
          <Stat
            label="TT rate"
            value={info.nodes > 0
              ? `${((info.tt_hits / info.nodes) * 100).toFixed(1)}%`
              : "—"}
          />
        </div>
      ) : (
        <div style={{ color: "#555", fontSize: 12 }}>Awaiting search…</div>
      )}
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <>
      <span style={{ color: "#777" }}>{label}</span>
      <span style={{ color: "#e0e0e0", textAlign: "right" }}>{value}</span>
    </>
  );
}

function fmtNum(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}
