/// Static position evaluation.
///
/// Returns a centipawn score from White's perspective.
/// Positive = White advantage, negative = Black advantage.

use super::bitboard::*;
use super::movegen::{bishop_attacks, knight_attacks, rook_attacks};

// ── Piece values (centipawns) ─────────────────────────────────────────────────

pub const PAWN_VAL:   i32 = 100;
pub const KNIGHT_VAL: i32 = 320;
pub const BISHOP_VAL: i32 = 330;
pub const ROOK_VAL:   i32 = 500;
pub const QUEEN_VAL:  i32 = 900;
pub const KING_VAL:   i32 = 20_000;

pub const PIECE_VALUES: [i32; 6] = [
    PAWN_VAL, KNIGHT_VAL, BISHOP_VAL, ROOK_VAL, QUEEN_VAL, KING_VAL,
];

// ── Piece-square tables (from White's perspective, rank 1 = index 0) ──────────

#[rustfmt::skip]
const PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const KNIGHT_PST: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_PST: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const KING_MG_PST: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

const PSTS: [&[i32; 64]; 6] = [
    &PAWN_PST,
    &KNIGHT_PST,
    &BISHOP_PST,
    &ROOK_PST,
    &PAWN_PST, // queen uses pawn table as placeholder
    &KING_MG_PST,
];

// ── Evaluation ────────────────────────────────────────────────────────────────

pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0i32;

    for c in 0..2usize {
        let sign = if c == 0 { 1 } else { -1 };

        for p in 0..6usize {
            let mut bb = board.pieces[c][p];
            while bb != 0 {
                let sq = pop_lsb(&mut bb) as usize;
                // Flip square for Black so PST is always from the piece's own perspective.
                let pst_sq = if c == 0 { sq } else { sq ^ 56 };
                score += sign * (PIECE_VALUES[p] + PSTS[p][pst_sq]);
            }
        }

        // Mobility bonus: count squares attacked by our pieces.
        let occ = board.occupied();
        let mob = mobility_score(board, if c == 0 { Color::White } else { Color::Black }, occ);
        score += sign * mob;

        // Bishop pair bonus
        if popcount(board.pieces[c][Piece::Bishop as usize]) >= 2 {
            score += sign * 30;
        }
    }

    score
}

fn mobility_score(board: &Board, c: Color, occ: Bitboard) -> i32 {
    let our = board.color_bb(c);
    let mut mob = 0i32;

    let mut knights = board.piece_bb(c, Piece::Knight);
    while knights != 0 {
        let sq = pop_lsb(&mut knights);
        mob += popcount(knight_attacks(sq) & !our) as i32 * 4;
    }

    let mut bishops = board.piece_bb(c, Piece::Bishop);
    while bishops != 0 {
        let sq = pop_lsb(&mut bishops);
        mob += popcount(bishop_attacks(sq, occ) & !our) as i32 * 3;
    }

    let mut rooks = board.piece_bb(c, Piece::Rook);
    while rooks != 0 {
        let sq = pop_lsb(&mut rooks);
        mob += popcount(rook_attacks(sq, occ) & !our) as i32 * 2;
    }

    mob
}

pub const CHECKMATE_SCORE: i32 = 100_000;
pub const DRAW_SCORE: i32 = 0;
