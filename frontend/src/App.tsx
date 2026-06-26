import { useEffect, useState } from "react";
import { Board } from "./components/Board";
import { Telemetry } from "./components/Telemetry";
import { useWebSocket } from "./hooks/useWebSocket";
import type { SearchInfoMsg } from "./types/game";

const STARTPOS_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";

export default function App() {
  const { status, lastMessage, send } = useWebSocket();

  const [fen, setFen] = useState(STARTPOS_FEN);
  const [legalMoves, setLegalMoves] = useState<[string, string][]>([]);
  const [lastMove, setLastMove] = useState<[string, string] | null>(null);
  const [aiMove, setAiMove] = useState<[string, string] | null>(null);
  const [aiEval, setAiEval] = useState<number | null>(null);
  const [searchInfo, setSearchInfo] = useState<SearchInfoMsg | null>(null);
  const [gameOver, setGameOver] = useState<string | null>(null);
  const [flipped, setFlipped] = useState(false);
  const [difficulty, setDifficulty] = useState(5);

  useEffect(() => {
    if (!lastMessage) return;

    switch (lastMessage.type) {
      case "game_state":
        setFen(lastMessage.fen);
        setLegalMoves(lastMessage.legal_moves);
        setGameOver(null);
        break;
      case "ai_move":
        setAiMove([lastMessage.from, lastMessage.to]);
        setAiEval(lastMessage.eval);
        break;
      case "search_info":
        setSearchInfo(lastMessage);
        break;
      case "game_over":
        setGameOver(lastMessage.result);
        setLegalMoves([]);
        break;
      case "error":
        console.error("Server error:", lastMessage.message);
        break;
    }
  }, [lastMessage]);

  function handleMove(from: string, to: string) {
    setLastMove([from, to]);
    setAiMove(null);
    send({ type: "move", from, to });
  }

  function handleNewGame() {
    setLastMove(null);
    setAiMove(null);
    setAiEval(null);
    setSearchInfo(null);
    setGameOver(null);
    send({ type: "new_game", difficulty });
  }

  function handleUndo() {
    setAiMove(null);
    send({ type: "undo" });
  }

  return (
    <div style={{
      minHeight: "100vh",
      background: "#0f0f1a",
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      justifyContent: "center",
      padding: 24,
      gap: 24,
    }}>
      <h1 style={{
        color: "#e0e0e0",
        fontFamily: "Georgia, serif",
        fontSize: 28,
        margin: 0,
        letterSpacing: 2,
      }}>
        ♟ Chess Engine
      </h1>

      <div style={{ display: "flex", gap: 24, alignItems: "flex-start", flexWrap: "wrap", justifyContent: "center" }}>
        <div style={{ position: "relative" }}>
          {gameOver && (
            <div style={{
              position: "absolute",
              inset: 0,
              background: "#0008",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              zIndex: 10,
              borderRadius: 4,
            }}>
              <div style={{
                background: "#1a1a2e",
                padding: "20px 32px",
                borderRadius: 8,
                color: "#e0e0e0",
                textAlign: "center",
                fontFamily: "Georgia, serif",
              }}>
                <div style={{ fontSize: 24, marginBottom: 8 }}>Game Over</div>
                <div style={{ fontSize: 14, color: "#aaa", marginBottom: 16 }}>{gameOver}</div>
                <button onClick={handleNewGame} style={btnStyle("#4a4a8a")}>
                  New Game
                </button>
              </div>
            </div>
          )}

          <Board
            fen={fen}
            legalMoves={legalMoves}
            lastMove={lastMove}
            aiMove={aiMove}
            flipped={flipped}
            onMove={handleMove}
          />
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <Telemetry info={searchInfo} aiEval={aiEval} status={status} />

          <div style={{
            background: "#1a1a2e",
            borderRadius: 8,
            padding: "14px 20px",
            display: "flex",
            flexDirection: "column",
            gap: 10,
          }}>
            <label style={{ color: "#aaa", fontSize: 12, fontFamily: "monospace" }}>
              Difficulty (depth)
              <input
                type="range" min={1} max={8} value={difficulty}
                onChange={e => setDifficulty(Number(e.target.value))}
                style={{ width: "100%", marginTop: 4 }}
              />
              <span style={{ color: "#e0e0e0" }}>{difficulty} ply</span>
            </label>

            <button onClick={handleNewGame} style={btnStyle("#4a4a8a")}>
              New Game
            </button>
            <button onClick={handleUndo} style={btnStyle("#3a3a5a")}>
              Undo
            </button>
            <button onClick={() => setFlipped(f => !f)} style={btnStyle("#2a2a4a")}>
              Flip Board
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function btnStyle(bg: string): React.CSSProperties {
  return {
    background: bg,
    color: "#e0e0e0",
    border: "none",
    borderRadius: 6,
    padding: "8px 16px",
    cursor: "pointer",
    fontFamily: "monospace",
    fontSize: 13,
    transition: "opacity 0.15s",
  };
}
