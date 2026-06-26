/// Bitboard-based chess board representation.
///
/// Each piece type and color occupies one u64. A set bit at index `i`
/// means that piece is on square `i` (index 0 = a1, 63 = h8).
/// All move generation and attack map queries use bitwise operations
/// with zero heap allocation in the hot path.

pub type Bitboard = u64;

// ── Bit manipulation helpers ──────────────────────────────────────────────────

#[inline(always)]
pub fn lsb(bb: Bitboard) -> u32 {
    bb.trailing_zeros()
}

#[inline(always)]
pub fn pop_lsb(bb: &mut Bitboard) -> u32 {
    let sq = lsb(*bb);
    *bb &= *bb - 1;
    sq
}

#[inline(always)]
pub fn sq_bb(sq: u32) -> Bitboard {
    1u64 << sq
}

#[inline(always)]
pub fn popcount(bb: Bitboard) -> u32 {
    bb.count_ones()
}

// ── File / rank masks ─────────────────────────────────────────────────────────

pub const FILE_A: Bitboard = 0x0101_0101_0101_0101;
pub const FILE_H: Bitboard = 0x8080_8080_8080_8080;
pub const RANK_1: Bitboard = 0x0000_0000_0000_00FF;
pub const RANK_8: Bitboard = 0xFF00_0000_0000_0000;
pub const RANK_2: Bitboard = RANK_1 << 8;
pub const RANK_7: Bitboard = RANK_8 >> 8;

// ── Color / piece indices ─────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn flip(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

// ── Board state ───────────────────────────────────────────────────────────────

/// Full board state stored as 12 bitboards (6 piece types × 2 colors).
/// Castling rights packed into a u8; en passant square as Option<u32>.
#[derive(Clone, Debug)]
pub struct Board {
    /// pieces[color][piece]
    pub pieces: [[Bitboard; 6]; 2],
    pub side_to_move: Color,
    /// Bits: 0=WK, 1=WQ, 2=BK, 3=BQ
    pub castling_rights: u8,
    pub en_passant: Option<u32>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Board {
    /// Return the starting position.
    pub fn startpos() -> Self {
        let mut b = Board {
            pieces: [[0u64; 6]; 2],
            side_to_move: Color::White,
            castling_rights: 0b1111,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        };

        // White pieces
        b.pieces[0][Piece::Pawn as usize] = 0x0000_0000_0000_FF00;
        b.pieces[0][Piece::Knight as usize] = 0x0000_0000_0000_0042;
        b.pieces[0][Piece::Bishop as usize] = 0x0000_0000_0000_0024;
        b.pieces[0][Piece::Rook as usize] = 0x0000_0000_0000_0081;
        b.pieces[0][Piece::Queen as usize] = 0x0000_0000_0000_0008;
        b.pieces[0][Piece::King as usize] = 0x0000_0000_0000_0010;

        // Black pieces
        b.pieces[1][Piece::Pawn as usize] = 0x00FF_0000_0000_0000;
        b.pieces[1][Piece::Knight as usize] = 0x4200_0000_0000_0000;
        b.pieces[1][Piece::Bishop as usize] = 0x2400_0000_0000_0000;
        b.pieces[1][Piece::Rook as usize] = 0x8100_0000_0000_0000;
        b.pieces[1][Piece::Queen as usize] = 0x0800_0000_0000_0000;
        b.pieces[1][Piece::King as usize] = 0x1000_0000_0000_0000;

        b
    }

    #[inline]
    pub fn occupied(&self) -> Bitboard {
        self.color_bb(Color::White) | self.color_bb(Color::Black)
    }

    #[inline]
    pub fn color_bb(&self, c: Color) -> Bitboard {
        self.pieces[c as usize].iter().fold(0, |acc, &bb| acc | bb)
    }

    #[inline]
    pub fn piece_bb(&self, c: Color, p: Piece) -> Bitboard {
        self.pieces[c as usize][p as usize]
    }

    /// Return which piece (if any) sits on `sq` for color `c`.
    pub fn piece_on(&self, c: Color, sq: u32) -> Option<Piece> {
        let mask = sq_bb(sq);
        for p in 0..6usize {
            if self.pieces[c as usize][p] & mask != 0 {
                return Some(match p {
                    0 => Piece::Pawn,
                    1 => Piece::Knight,
                    2 => Piece::Bishop,
                    3 => Piece::Rook,
                    4 => Piece::Queen,
                    _ => Piece::King,
                });
            }
        }
        None
    }
}

// ── Square utilities ──────────────────────────────────────────────────────────

pub fn sq_name(sq: u32) -> String {
    let file = (b'a' + (sq % 8) as u8) as char;
    let rank = (b'1' + (sq / 8) as u8) as char;
    format!("{}{}", file, rank)
}

pub fn sq_from_name(name: &str) -> Option<u32> {
    let bytes = name.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let file = bytes[0].checked_sub(b'a')? as u32;
    let rank = bytes[1].checked_sub(b'1')? as u32;
    if file < 8 && rank < 8 {
        Some(rank * 8 + file)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_piece_counts() {
        let b = Board::startpos();
        assert_eq!(popcount(b.piece_bb(Color::White, Piece::Pawn)), 8);
        assert_eq!(popcount(b.piece_bb(Color::Black, Piece::Pawn)), 8);
        assert_eq!(popcount(b.occupied()), 32);
    }

    #[test]
    fn sq_round_trip() {
        for sq in 0..64u32 {
            let name = sq_name(sq);
            assert_eq!(sq_from_name(&name), Some(sq));
        }
    }
}
