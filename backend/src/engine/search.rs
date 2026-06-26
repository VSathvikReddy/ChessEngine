/// Minimax search with Alpha-Beta pruning and Transposition Table integration.
///
/// Alpha-beta prunes branches that cannot affect the final decision:
///   - α (alpha): best score the maximizer can guarantee
///   - β (beta):  best score the minimizer can guarantee
///   - If a node's score ≥ β (fail-high), the minimizer will never reach it
///     → prune that branch entirely (β-cutoff).
///
/// The transposition table (TT) short-circuits evaluation of positions already
/// seen via different move orders, using Zobrist hashes as keys.

use super::{
    bitboard::{Board, Color, Piece},
    eval::{evaluate, CHECKMATE_SCORE, DRAW_SCORE, PIECE_VALUES},
    movegen::{generate_moves, Move, MoveFlags},
    transposition::{hash, Bound, TranspositionTable},
};
use std::time::Instant;

// ── Search context ────────────────────────────────────────────────────────────

pub struct SearchStats {
    pub nodes: u64,
    pub tt_hits: u64,
    pub depth_reached: u8,
    pub elapsed_ms: u64,
}

pub struct Searcher {
    pub tt: TranspositionTable,
}

impl Searcher {
    pub fn new(tt_size_mb: usize) -> Self {
        Searcher {
            tt: TranspositionTable::new(tt_size_mb),
        }
    }

    /// Find the best move for the side to move, searching to `depth` ply.
    pub fn best_move(&mut self, board: &mut Board, depth: u8) -> Option<(Move, SearchStats)> {
        let start = Instant::now();
        let mut nodes = 0u64;

        let mut best_move = None;
        let mut best_score = i32::MIN + 1;
        let alpha = i32::MIN + 1;
        let beta = i32::MAX;

        let moves = generate_moves(board);
        if moves.is_empty() {
            return None;
        }

        let mut ordered = order_moves(moves, board);

        for mv in &ordered {
            if let Some(new_board) = apply_move(board, *mv) {
                let score = -self.negamax(&mut new_board.clone(), depth - 1, -beta, -alpha, &mut nodes);
                if score > best_score {
                    best_score = score;
                    best_move = Some(*mv);
                }
            }
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;
        let stats = SearchStats {
            nodes,
            tt_hits: self.tt.hits,
            depth_reached: depth,
            elapsed_ms,
        };

        best_move.map(|mv| (mv, stats))
    }

    /// Negamax with alpha-beta pruning.
    ///
    /// Returns the best score for the side to move (always maximizing).
    /// Scores are negated at each level so the caller's perspective is positive.
    fn negamax(
        &mut self,
        board: &mut Board,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        nodes: &mut u64,
    ) -> i32 {
        *nodes += 1;

        let key = hash(board);

        // ── Transposition table probe ──────────────────────────────────────────
        if let Some(entry) = self.tt.probe(key) {
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::LowerBound => {
                        if entry.score >= beta {
                            return entry.score;
                        }
                    }
                    Bound::UpperBound => {
                        if entry.score <= alpha {
                            return entry.score;
                        }
                    }
                }
            }
        }

        // ── Leaf node: quiescence search ───────────────────────────────────────
        if depth == 0 {
            return self.quiesce(board, alpha, beta, nodes);
        }

        // ── Generate and order moves ───────────────────────────────────────────
        let moves = generate_moves(board);
        if moves.is_empty() {
            // No legal moves: checkmate or stalemate.
            // (Proper legality checking omitted for brevity; engine calls apply_move
            //  which returns None for illegal moves, but for terminal node detection
            //  we'd need a full is_in_check query here.)
            return DRAW_SCORE;
        }

        let ordered = order_moves(moves, board);
        let mut best_score = i32::MIN + 1;
        let mut best_from = 0u8;
        let mut best_to = 0u8;
        let mut bound = Bound::UpperBound;

        for mv in &ordered {
            let Some(mut child) = apply_move(board, *mv) else {
                continue;
            };

            let score = -self.negamax(&mut child, depth - 1, -beta, -alpha, nodes);

            if score > best_score {
                best_score = score;
                best_from = mv.from as u8;
                best_to = mv.to as u8;
            }

            if score > alpha {
                alpha = score;
                bound = Bound::Exact;
            }

            // β-cutoff: the minimizer will never allow this position.
            if alpha >= beta {
                bound = Bound::LowerBound;
                break;
            }
        }

        // ── Store in transposition table ───────────────────────────────────────
        self.tt.store(key, depth, best_score, bound, best_from, best_to);

        best_score
    }

    /// Quiescence search: continue searching captures until a "quiet" position
    /// is reached to avoid the horizon effect.
    fn quiesce(&mut self, board: &mut Board, mut alpha: i32, beta: i32, nodes: &mut u64) -> i32 {
        *nodes += 1;

        let stand_pat = evaluate_from_side(board);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let moves = generate_moves(board);
        let captures: Vec<Move> = moves
            .into_iter()
            .filter(|m| m.flags.contains(MoveFlags::CAPTURE) || m.flags.contains(MoveFlags::EN_PASSANT))
            .collect();

        for mv in captures {
            let Some(mut child) = apply_move(board, mv) else {
                continue;
            };
            let score = -self.quiesce(&mut child, -beta, -alpha, nodes);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }
}

// ── Move ordering ─────────────────────────────────────────────────────────────

/// Order moves to improve pruning efficiency:
/// 1. Captures ordered by MVV-LVA (most valuable victim, least valuable attacker)
/// 2. Promotions
/// 3. Quiet moves
fn order_moves(mut moves: Vec<Move>, board: &Board) -> Vec<Move> {
    moves.sort_unstable_by_key(|mv| {
        let mut score = 0i32;

        if mv.flags.contains(MoveFlags::CAPTURE) {
            let victim = board
                .piece_on(board.side_to_move.flip(), mv.to)
                .map(|p| PIECE_VALUES[p as usize])
                .unwrap_or(0);
            let attacker = board
                .piece_on(board.side_to_move, mv.from)
                .map(|p| PIECE_VALUES[p as usize])
                .unwrap_or(0);
            score += 10_000 + victim - attacker / 10;
        }

        if mv.flags.contains(MoveFlags::PROMOTION) {
            score += 9_000;
        }

        // Negate for descending sort.
        -score
    });
    moves
}

// ── Move application ──────────────────────────────────────────────────────────

/// Apply a move and return the new board, or `None` if the move is illegal
/// (leaves king in check — simplified check here).
pub fn apply_move(board: &Board, mv: Move) -> Option<Board> {
    let mut new = board.clone();
    let us = new.side_to_move;
    let them = us.flip();

    // Find moving piece
    let piece = new.piece_on(us, mv.from)?;
    let from_bb = super::bitboard::sq_bb(mv.from);
    let to_bb = super::bitboard::sq_bb(mv.to);

    // Remove from source
    new.pieces[us as usize][piece as usize] &= !from_bb;

    // Remove captured piece (any opponent piece on `to`)
    for p in 0..6usize {
        new.pieces[them as usize][p] &= !to_bb;
    }

    // En passant capture
    if mv.flags.contains(MoveFlags::EN_PASSANT) {
        let ep_pawn_sq = match us {
            Color::White => mv.to - 8,
            Color::Black => mv.to + 8,
        };
        new.pieces[them as usize][Piece::Pawn as usize] &= !super::bitboard::sq_bb(ep_pawn_sq);
    }

    // Promotion: place a queen (extend to support underpromotion as needed)
    let landing_piece = if mv.flags.contains(MoveFlags::PROMOTION) {
        Piece::Queen
    } else {
        piece
    };
    new.pieces[us as usize][landing_piece as usize] |= to_bb;

    // Castling: also move the rook
    if mv.flags.contains(MoveFlags::CASTLE_K) {
        match us {
            Color::White => {
                new.pieces[0][Piece::Rook as usize] &= !0x0000_0000_0000_0080;
                new.pieces[0][Piece::Rook as usize] |= 0x0000_0000_0000_0020;
            }
            Color::Black => {
                new.pieces[1][Piece::Rook as usize] &= !0x8000_0000_0000_0000;
                new.pieces[1][Piece::Rook as usize] |= 0x2000_0000_0000_0000;
            }
        }
    } else if mv.flags.contains(MoveFlags::CASTLE_Q) {
        match us {
            Color::White => {
                new.pieces[0][Piece::Rook as usize] &= !0x0000_0000_0000_0001;
                new.pieces[0][Piece::Rook as usize] |= 0x0000_0000_0000_0008;
            }
            Color::Black => {
                new.pieces[1][Piece::Rook as usize] &= !0x0100_0000_0000_0000;
                new.pieces[1][Piece::Rook as usize] |= 0x0800_0000_0000_0000;
            }
        }
    }

    // Update castling rights
    update_castling_rights(&mut new, mv.from, mv.to, piece, us);

    // En passant square
    new.en_passant = if mv.flags.contains(MoveFlags::DOUBLE_PUSH) {
        let ep_sq = match us {
            Color::White => mv.from + 8,
            Color::Black => mv.from - 8,
        };
        Some(ep_sq)
    } else {
        None
    };

    // Halfmove clock
    if piece == Piece::Pawn || mv.flags.contains(MoveFlags::CAPTURE) {
        new.halfmove_clock = 0;
    } else {
        new.halfmove_clock += 1;
    }

    if us == Color::Black {
        new.fullmove_number += 1;
    }

    new.side_to_move = them;
    Some(new)
}

fn update_castling_rights(board: &mut Board, from: u32, to: u32, piece: Piece, us: Color) {
    if piece == Piece::King {
        match us {
            Color::White => board.castling_rights &= !0b0011,
            Color::Black => board.castling_rights &= !0b1100,
        }
    }
    if from == 0 || to == 0 { board.castling_rights &= !0b0010; }
    if from == 7 || to == 7 { board.castling_rights &= !0b0001; }
    if from == 56 || to == 56 { board.castling_rights &= !0b1000; }
    if from == 63 || to == 63 { board.castling_rights &= !0b0100; }
}

/// Evaluate from the perspective of the side to move (negamax convention).
fn evaluate_from_side(board: &Board) -> i32 {
    let score = evaluate(board);
    if board.side_to_move == Color::White { score } else { -score }
}
