// ── Outbound (client → server) ────────────────────────────────────────────────

export type ClientMove = { type: "move"; from: string; to: string };
export type ClientNewGame = { type: "new_game"; difficulty?: number };
export type ClientUndo = { type: "undo" };
export type ClientMsg = ClientMove | ClientNewGame | ClientUndo;

// ── Inbound (server → client) ─────────────────────────────────────────────────

export interface GameStateMsg {
  type: "game_state";
  fen: string;
  legal_moves: [string, string][];
}

export interface AiMoveMsg {
  type: "ai_move";
  from: string;
  to: string;
  eval: number;
}

export interface SearchInfoMsg {
  type: "search_info";
  depth: number;
  nodes: number;
  tt_hits: number;
  elapsed_ms: number;
}

export interface GameOverMsg {
  type: "game_over";
  result: "checkmate" | "stalemate" | "draw" | string;
}

export interface ErrorMsg {
  type: "error";
  message: string;
}

export type ServerMsg =
  | GameStateMsg
  | AiMoveMsg
  | SearchInfoMsg
  | GameOverMsg
  | ErrorMsg;

// ── Board helpers ─────────────────────────────────────────────────────────────

/** FEN piece char → { color, type } */
export function parsePiece(ch: string): { color: "w" | "b"; type: string } {
  const upper = ch.toUpperCase();
  return {
    color: ch === upper ? "w" : "b",
    type: upper,
  };
}

/** "e2" → { file: 4, rank: 1 } (0-indexed) */
export function sqToCoords(sq: string): { file: number; rank: number } {
  return {
    file: sq.charCodeAt(0) - "a".charCodeAt(0),
    rank: parseInt(sq[1], 10) - 1,
  };
}

/** { file, rank } → "e2" */
export function coordsToSq(file: number, rank: number): string {
  return String.fromCharCode("a".charCodeAt(0) + file) + String(rank + 1);
}

// ── FEN parser ────────────────────────────────────────────────────────────────

export interface ParsedFen {
  /** squares[rank][file] = piece char or null */
  squares: (string | null)[][];
  sideToMove: "w" | "b";
  castling: string;
  enPassant: string | null;
}

export function parseFen(fen: string): ParsedFen {
  const parts = fen.split(" ");
  const rows = parts[0].split("/");
  const squares: (string | null)[][] = [];

  for (let rank = 7; rank >= 0; rank--) {
    const row: (string | null)[] = [];
    for (const ch of rows[7 - rank]) {
      const n = parseInt(ch, 10);
      if (!isNaN(n)) {
        for (let i = 0; i < n; i++) row.push(null);
      } else {
        row.push(ch);
      }
    }
    squares[rank] = row;
  }

  return {
    squares,
    sideToMove: (parts[1] as "w" | "b") ?? "w",
    castling: parts[2] ?? "-",
    enPassant: parts[3] !== "-" ? parts[3] : null,
  };
}
