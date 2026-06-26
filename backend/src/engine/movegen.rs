/// Pseudo-legal move generation using bitboard attack maps.
///
/// All sliding-piece attack generation uses classical hyperbola quintessence
/// (o^(o-2r) trick) — branch-free and allocation-free.

use super::bitboard::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: u32,
    pub to: u32,
    pub flags: MoveFlags,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct MoveFlags: u8 {
        const QUIET      = 0b0000_0000;
        const CAPTURE    = 0b0000_0001;
        const EN_PASSANT = 0b0000_0010;
        const PROMOTION  = 0b0000_0100;
        const CASTLE_K   = 0b0000_1000;
        const CASTLE_Q   = 0b0001_0000;
        const DOUBLE_PUSH= 0b0010_0000;
    }
}

/// Generate all pseudo-legal moves for the side to move.
/// Caller is responsible for legality filtering (king in check after move).
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(64);
    let us = board.side_to_move;
    let them = us.flip();
    let occ = board.occupied();
    let our_pieces = board.color_bb(us);
    let their_pieces = board.color_bb(them);

    // Pawns
    pawn_moves(board, us, occ, their_pieces, &mut moves);

    // Knights
    let mut knights = board.piece_bb(us, Piece::Knight);
    while knights != 0 {
        let sq = pop_lsb(&mut knights);
        let attacks = knight_attacks(sq) & !our_pieces;
        push_quiet_and_captures(sq, attacks, their_pieces, &mut moves);
    }

    // Bishops
    let mut bishops = board.piece_bb(us, Piece::Bishop);
    while bishops != 0 {
        let sq = pop_lsb(&mut bishops);
        let attacks = bishop_attacks(sq, occ) & !our_pieces;
        push_quiet_and_captures(sq, attacks, their_pieces, &mut moves);
    }

    // Rooks
    let mut rooks = board.piece_bb(us, Piece::Rook);
    while rooks != 0 {
        let sq = pop_lsb(&mut rooks);
        let attacks = rook_attacks(sq, occ) & !our_pieces;
        push_quiet_and_captures(sq, attacks, their_pieces, &mut moves);
    }

    // Queens
    let mut queens = board.piece_bb(us, Piece::Queen);
    while queens != 0 {
        let sq = pop_lsb(&mut queens);
        let attacks = (bishop_attacks(sq, occ) | rook_attacks(sq, occ)) & !our_pieces;
        push_quiet_and_captures(sq, attacks, their_pieces, &mut moves);
    }

    // King
    let king_sq = lsb(board.piece_bb(us, Piece::King));
    let king_attacks = king_attacks(king_sq) & !our_pieces;
    push_quiet_and_captures(king_sq, king_attacks, their_pieces, &mut moves);

    // Castling
    castling_moves(board, us, occ, &mut moves);

    moves
}

// ── Attack maps ───────────────────────────────────────────────────────────────

pub fn knight_attacks(sq: u32) -> Bitboard {
    let bb = sq_bb(sq);
    let l1 = (bb >> 1) & !FILE_H;
    let r1 = (bb << 1) & !FILE_A;
    let h1 = l1 | r1;
    let l2 = (bb >> 2) & !(FILE_G | FILE_H);
    let r2 = (bb << 2) & !(FILE_A | FILE_B);
    let h2 = l2 | r2;
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
}

pub fn king_attacks(sq: u32) -> Bitboard {
    let bb = sq_bb(sq);
    let lr = ((bb >> 1) & !FILE_H) | ((bb << 1) & !FILE_A);
    let ring = bb | lr;
    (ring << 8) | (ring >> 8) | lr
}

/// Hyperbola quintessence for rank/file sliders.
#[inline]
fn sliding_attacks(sq: u32, occ: Bitboard, mask: Bitboard) -> Bitboard {
    let o = occ & mask;
    let sq_bb = sq_bb(sq);
    let forward = o.wrapping_sub(sq_bb.wrapping_shl(1));
    let reverse = (o.swap_bytes().wrapping_sub((sq_bb.swap_bytes()).wrapping_shl(1))).swap_bytes();
    (forward ^ reverse) & mask
}

pub fn rook_attacks(sq: u32, occ: Bitboard) -> Bitboard {
    sliding_attacks(sq, occ, rank_mask(sq)) | sliding_attacks(sq, occ, file_mask(sq))
}

pub fn bishop_attacks(sq: u32, occ: Bitboard) -> Bitboard {
    sliding_attacks(sq, occ, diag_mask(sq)) | sliding_attacks(sq, occ, anti_diag_mask(sq))
}

fn rank_mask(sq: u32) -> Bitboard {
    RANK_1 << (8 * (sq / 8))
}

fn file_mask(sq: u32) -> Bitboard {
    FILE_A << (sq % 8)
}

fn diag_mask(sq: u32) -> Bitboard {
    // 64 diagonals: index by (rank - file + 7)
    let diag = (sq / 8) as i32 - (sq % 8) as i32 + 7;
    DIAGONALS[diag as usize]
}

fn anti_diag_mask(sq: u32) -> Bitboard {
    let anti = (sq / 8) as i32 + (sq % 8) as i32;
    ANTI_DIAGONALS[anti as usize]
}

const FILE_G: Bitboard = FILE_A << 6;
const FILE_B: Bitboard = FILE_A << 1;

// Pre-computed diagonal tables (generated at compile time via const fn).
const DIAGONALS: [Bitboard; 15] = compute_diagonals();
const ANTI_DIAGONALS: [Bitboard; 15] = compute_anti_diagonals();

const fn compute_diagonals() -> [Bitboard; 15] {
    let mut diags = [0u64; 15];
    let mut sq = 0usize;
    while sq < 64 {
        let d = (sq / 8) as i32 - (sq % 8) as i32 + 7;
        diags[d as usize] |= 1u64 << sq;
        sq += 1;
    }
    diags
}

const fn compute_anti_diagonals() -> [Bitboard; 15] {
    let mut anti = [0u64; 15];
    let mut sq = 0usize;
    while sq < 64 {
        let a = (sq / 8) as i32 + (sq % 8) as i32;
        anti[a as usize] |= 1u64 << sq;
        sq += 1;
    }
    anti
}

// ── Pawn moves ────────────────────────────────────────────────────────────────

fn pawn_moves(
    board: &Board,
    us: Color,
    occ: Bitboard,
    their_pieces: Bitboard,
    moves: &mut Vec<Move>,
) {
    let pawns = board.piece_bb(us, Piece::Pawn);
    let ep_bb = board.en_passant.map(sq_bb).unwrap_or(0);

    match us {
        Color::White => {
            // Single push
            let push1 = (pawns << 8) & !occ;
            // Double push from rank 2
            let push2 = ((push1 & RANK_3) << 8) & !occ;
            // Captures
            let cap_l = ((pawns & !FILE_A) << 7) & their_pieces;
            let cap_r = ((pawns & !FILE_H) << 9) & their_pieces;

            emit_pawn_moves(push1, 8, MoveFlags::QUIET, moves);
            emit_pawn_moves(push2, 16, MoveFlags::DOUBLE_PUSH, moves);
            emit_pawn_moves(cap_l, 7, MoveFlags::CAPTURE, moves);
            emit_pawn_moves(cap_r, 9, MoveFlags::CAPTURE, moves);

            // En passant
            if ep_bb != 0 {
                if ((pawns & !FILE_A) << 7) & ep_bb != 0 {
                    moves.push(Move { from: lsb(ep_bb) - 7, to: lsb(ep_bb), flags: MoveFlags::EN_PASSANT });
                }
                if ((pawns & !FILE_H) << 9) & ep_bb != 0 {
                    moves.push(Move { from: lsb(ep_bb) - 9, to: lsb(ep_bb), flags: MoveFlags::EN_PASSANT });
                }
            }
        }
        Color::Black => {
            let push1 = (pawns >> 8) & !occ;
            let push2 = ((push1 & RANK_6) >> 8) & !occ;
            let cap_l = ((pawns & !FILE_H) >> 7) & their_pieces;
            let cap_r = ((pawns & !FILE_A) >> 9) & their_pieces;

            emit_pawn_moves_back(push1, 8, MoveFlags::QUIET, moves);
            emit_pawn_moves_back(push2, 16, MoveFlags::DOUBLE_PUSH, moves);
            emit_pawn_moves_back(cap_l, 7, MoveFlags::CAPTURE, moves);
            emit_pawn_moves_back(cap_r, 9, MoveFlags::CAPTURE, moves);

            if ep_bb != 0 {
                if ((pawns & !FILE_H) >> 7) & ep_bb != 0 {
                    moves.push(Move { from: lsb(ep_bb) + 7, to: lsb(ep_bb), flags: MoveFlags::EN_PASSANT });
                }
                if ((pawns & !FILE_A) >> 9) & ep_bb != 0 {
                    moves.push(Move { from: lsb(ep_bb) + 9, to: lsb(ep_bb), flags: MoveFlags::EN_PASSANT });
                }
            }
        }
    }
}

const RANK_3: Bitboard = RANK_1 << 16;
const RANK_6: Bitboard = RANK_8 >> 16;

fn emit_pawn_moves(mut targets: Bitboard, offset: u32, flags: MoveFlags, moves: &mut Vec<Move>) {
    while targets != 0 {
        let to = pop_lsb(&mut targets);
        let from = to - offset;
        let f = if to >= 56 { flags | MoveFlags::PROMOTION } else { flags };
        moves.push(Move { from, to, flags: f });
    }
}

fn emit_pawn_moves_back(mut targets: Bitboard, offset: u32, flags: MoveFlags, moves: &mut Vec<Move>) {
    while targets != 0 {
        let to = pop_lsb(&mut targets);
        let from = to + offset;
        let f = if to < 8 { flags | MoveFlags::PROMOTION } else { flags };
        moves.push(Move { from, to, flags: f });
    }
}

// ── Castling ──────────────────────────────────────────────────────────────────

fn castling_moves(board: &Board, us: Color, occ: Bitboard, moves: &mut Vec<Move>) {
    match us {
        Color::White => {
            if board.castling_rights & 0b0001 != 0
                && occ & 0x0000_0000_0000_0060 == 0
            {
                moves.push(Move { from: 4, to: 6, flags: MoveFlags::CASTLE_K });
            }
            if board.castling_rights & 0b0010 != 0
                && occ & 0x0000_0000_0000_000E == 0
            {
                moves.push(Move { from: 4, to: 2, flags: MoveFlags::CASTLE_Q });
            }
        }
        Color::Black => {
            if board.castling_rights & 0b0100 != 0
                && occ & 0x6000_0000_0000_0000 == 0
            {
                moves.push(Move { from: 60, to: 62, flags: MoveFlags::CASTLE_K });
            }
            if board.castling_rights & 0b1000 != 0
                && occ & 0x0E00_0000_0000_0000 == 0
            {
                moves.push(Move { from: 60, to: 58, flags: MoveFlags::CASTLE_Q });
            }
        }
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn push_quiet_and_captures(from: u32, mut targets: Bitboard, their_pieces: Bitboard, moves: &mut Vec<Move>) {
    while targets != 0 {
        let to = pop_lsb(&mut targets);
        let flags = if sq_bb(to) & their_pieces != 0 {
            MoveFlags::CAPTURE
        } else {
            MoveFlags::QUIET
        };
        moves.push(Move { from, to, flags });
    }
}
