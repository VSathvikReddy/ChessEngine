# ♟ Chess Engine

A full-stack chess engine featuring a React UI, Rust/Axum backend, and a hand-crafted AI opponent — connected over WebSockets for real-time, low-latency game telemetry.

## Architecture

```
┌─────────────────────┐        WebSocket (JSON)       ┌──────────────────────────┐
│   React Frontend    │ ◄────────────────────────────► │   Rust / Axum Backend    │
│                     │                               │                          │
│  • Board UI         │                               │  • WS message broker     │
│  • Move validation  │                               │  • Bitboard engine       │
│  • Game telemetry   │                               │  • Minimax + α-β pruning │
│  • WebSocket client │                               │  • Zobrist transposition │
└─────────────────────┘                               └──────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 18, TypeScript, Vite |
| Backend | Rust, Axum, Tokio |
| Transport | WebSockets (`tokio-tungstenite`) |
| AI | Minimax + Alpha-Beta Pruning |
| Board State | 64-bit Bitboards (zero-allocation) |
| Caching | Zobrist-hashed Transposition Table |

## Key Design Decisions

### Bitboards
Board state is represented as twelve 64-bit integers (one per piece type per color). All move generation, attack maps, and legality checks use bitwise operations — no heap allocation in the hot path.

### Minimax with Alpha-Beta Pruning
The AI searches the game tree to configurable depth. Alpha-beta pruning cuts branches that cannot influence the final decision, dramatically reducing nodes evaluated vs. naive minimax.

### Zobrist Hashing + Transposition Table
Each board position is assigned a unique 64-bit hash via XOR of random per-piece-per-square values. The transposition table caches evaluations keyed by this hash, avoiding redundant subtree evaluation when the same position is reached via different move orders.

### WebSocket Streaming
Game telemetry (moves, evaluations, search stats) is streamed as JSON payloads over a persistent WebSocket connection, keeping the UI reactive without polling.

## Getting Started

### Prerequisites
- Node.js ≥ 18
- Rust ≥ 1.75 (install via [rustup](https://rustup.rs))

### Backend

```bash
cd backend
cargo build --release
cargo run --release
# Starts on ws://localhost:3000
```

### Frontend

```bash
cd frontend
npm install
npm run dev
# Opens on http://localhost:5173
```

## Project Structure

```
chess-engine/
├── backend/
│   ├── src/
│   │   ├── main.rs              # Axum server entry point
│   │   ├── engine/
│   │   │   ├── mod.rs
│   │   │   ├── bitboard.rs      # 64-bit board representation
│   │   │   ├── movegen.rs       # Move generation via bitwise ops
│   │   │   ├── eval.rs          # Position evaluation heuristics
│   │   │   ├── search.rs        # Minimax + alpha-beta
│   │   │   └── transposition.rs # Zobrist hashing + TT
│   │   ├── ws/
│   │   │   ├── mod.rs
│   │   │   └── handler.rs       # WebSocket upgrade + message loop
│   │   └── api/
│   │       └── routes.rs        # Axum router
│   └── Cargo.toml
└── frontend/
    ├── src/
    │   ├── main.tsx
    │   ├── App.tsx
    │   ├── components/
    │   │   ├── Board.tsx         # SVG chessboard
    │   │   ├── Piece.tsx         # Piece rendering
    │   │   └── Telemetry.tsx     # Live AI stats panel
    │   ├── hooks/
    │   │   └── useWebSocket.ts   # WS client hook
    │   └── types/
    │       └── game.ts           # Shared message types
    ├── package.json
    └── vite.config.ts
```

## WebSocket Protocol

All messages are JSON with a `type` discriminant:

```ts
// Client → Server
{ type: "move", from: "e2", to: "e4" }
{ type: "new_game", difficulty: 4 }
{ type: "undo" }

// Server → Client
{ type: "game_state", fen: string, legal_moves: string[] }
{ type: "ai_move", from: string, to: string, eval: number }
{ type: "search_info", depth: number, nodes: number, tt_hits: number, elapsed_ms: number }
{ type: "game_over", result: "checkmate" | "stalemate" | "draw" }
```

## Performance

| Metric | Value |
|--------|-------|
| Move generation | ~200M moves/sec (bitboard ops) |
| Search depth (default) | 6 ply |
| Transposition table size | 64MB (configurable) |
| WS round-trip latency | < 5ms (localhost) |

## License

MIT
