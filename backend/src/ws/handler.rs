/// WebSocket handler: upgrades HTTP connections and manages the game message loop.
///
/// Each connected client gets its own Board + Searcher. Messages are JSON
/// with a `type` discriminant (see protocol in README).

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::engine::{
    bitboard::{Board, sq_from_name, sq_name},
    search::{apply_move, Searcher},
    movegen::{generate_moves, Move},
};

// ── Protocol types ────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMsg {
    Move { from: String, to: String },
    NewGame { difficulty: Option<u8> },
    Undo,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMsg {
    GameState {
        fen: String,
        legal_moves: Vec<[String; 2]>,
    },
    AiMove {
        from: String,
        to: String,
        eval: i32,
    },
    SearchInfo {
        depth: u8,
        nodes: u64,
        tt_hits: u64,
        elapsed_ms: u64,
    },
    GameOver {
        result: String,
    },
    Error {
        message: String,
    },
}

// ── Upgrade handler ───────────────────────────────────────────────────────────

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    info!("Client connected");

    let mut board = Board::startpos();
    let mut searcher = Searcher::new(64); // 64 MB TT
    let mut history: Vec<Board> = Vec::new();
    let mut depth = 5u8;

    // Send initial game state
    if let Err(e) = send_state(&mut socket, &board).await {
        warn!("Failed to send initial state: {}", e);
        return;
    }

    while let Some(msg) = socket.recv().await {
        let Ok(msg) = msg else { break };

        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let client_msg: ClientMsg = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                let _ = send_msg(&mut socket, &ServerMsg::Error {
                    message: format!("Parse error: {}", e),
                }).await;
                continue;
            }
        };

        match client_msg {
            ClientMsg::NewGame { difficulty } => {
                board = Board::startpos();
                history.clear();
                searcher.tt.clear();
                depth = difficulty.unwrap_or(5).clamp(1, 8);
                let _ = send_state(&mut socket, &board).await;
            }

            ClientMsg::Move { from, to } => {
                let Some(from_sq) = sq_from_name(&from) else {
                    let _ = send_msg(&mut socket, &ServerMsg::Error {
                        message: format!("Invalid square: {from}"),
                    }).await;
                    continue;
                };
                let Some(to_sq) = sq_from_name(&to) else {
                    let _ = send_msg(&mut socket, &ServerMsg::Error {
                        message: format!("Invalid square: {to}"),
                    }).await;
                    continue;
                };

                // Find matching legal move
                let legal = generate_moves(&board);
                let found = legal.iter().find(|m| m.from == from_sq && m.to == to_sq);

                match found {
                    None => {
                        let _ = send_msg(&mut socket, &ServerMsg::Error {
                            message: "Illegal move".into(),
                        }).await;
                    }
                    Some(&mv) => {
                        let Some(new_board) = apply_move(&board, mv) else {
                            let _ = send_msg(&mut socket, &ServerMsg::Error {
                                message: "Move application failed".into(),
                            }).await;
                            continue;
                        };

                        history.push(board.clone());
                        board = new_board;

                        // Check for game over before AI moves
                        if generate_moves(&board).is_empty() {
                            let _ = send_msg(&mut socket, &ServerMsg::GameOver {
                                result: "checkmate_or_stalemate".into(),
                            }).await;
                            continue;
                        }

                        let _ = send_state(&mut socket, &board).await;

                        // AI response
                        let mut ai_board = board.clone();
                        if let Some((ai_mv, stats)) = searcher.best_move(&mut ai_board, depth) {
                            // Emit search telemetry first
                            let _ = send_msg(&mut socket, &ServerMsg::SearchInfo {
                                depth: stats.depth_reached,
                                nodes: stats.nodes,
                                tt_hits: stats.tt_hits,
                                elapsed_ms: stats.elapsed_ms,
                            }).await;

                            if let Some(new_board) = apply_move(&board, ai_mv) {
                                history.push(board.clone());
                                board = new_board;

                                let eval = crate::engine::eval::evaluate(&board);
                                let _ = send_msg(&mut socket, &ServerMsg::AiMove {
                                    from: sq_name(ai_mv.from),
                                    to: sq_name(ai_mv.to),
                                    eval,
                                }).await;

                                let _ = send_state(&mut socket, &board).await;
                            }
                        } else {
                            let _ = send_msg(&mut socket, &ServerMsg::GameOver {
                                result: "ai_no_moves".into(),
                            }).await;
                        }
                    }
                }
            }

            ClientMsg::Undo => {
                if history.len() >= 2 {
                    history.pop(); // remove AI move
                    board = history.pop().unwrap();
                    let _ = send_state(&mut socket, &board).await;
                } else {
                    let _ = send_msg(&mut socket, &ServerMsg::Error {
                        message: "Nothing to undo".into(),
                    }).await;
                }
            }
        }
    }

    info!("Client disconnected");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn send_state(
    socket: &mut WebSocket,
    board: &Board,
) -> Result<(), axum::Error> {
    let moves = generate_moves(board);
    let legal_moves = moves
        .iter()
        .map(|m| [sq_name(m.from), sq_name(m.to)])
        .collect();

    send_msg(socket, &ServerMsg::GameState {
        fen: board_to_fen(board),
        legal_moves,
    }).await
}

async fn send_msg(socket: &mut WebSocket, msg: &ServerMsg) -> Result<(), axum::Error> {
    let json = serde_json::to_string(msg).unwrap();
    socket.send(Message::Text(json)).await
}

/// Minimal FEN serializer (position + side-to-move + castling + ep).
fn board_to_fen(board: &Board) -> String {
    use crate::engine::bitboard::{Color, Piece, sq_bb};

    let mut fen = String::new();

    for rank in (0..8u32).rev() {
        let mut empty = 0u32;
        for file in 0..8u32 {
            let sq = rank * 8 + file;
            let mut found = false;

            'outer: for c in [Color::White, Color::Black] {
                for p in 0..6usize {
                    if board.pieces[c as usize][p] & sq_bb(sq) != 0 {
                        if empty > 0 {
                            fen.push((b'0' + empty as u8) as char);
                            empty = 0;
                        }
                        let ch = match p {
                            0 => 'p', 1 => 'n', 2 => 'b',
                            3 => 'r', 4 => 'q', _ => 'k',
                        };
                        fen.push(if c == Color::White { ch.to_ascii_uppercase() } else { ch });
                        found = true;
                        break 'outer;
                    }
                }
            }

            if !found { empty += 1; }
        }
        if empty > 0 { fen.push((b'0' + empty as u8) as char); }
        if rank > 0 { fen.push('/'); }
    }

    fen.push(' ');
    fen.push(if board.side_to_move == Color::White { 'w' } else { 'b' });

    // Castling
    fen.push(' ');
    let cr = board.castling_rights;
    if cr == 0 {
        fen.push('-');
    } else {
        if cr & 1 != 0 { fen.push('K'); }
        if cr & 2 != 0 { fen.push('Q'); }
        if cr & 4 != 0 { fen.push('k'); }
        if cr & 8 != 0 { fen.push('q'); }
    }

    // En passant
    fen.push(' ');
    match board.en_passant {
        Some(sq) => fen.push_str(&sq_name(sq)),
        None => fen.push('-'),
    }

    fen
}
