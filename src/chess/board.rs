use std::fmt;

use super::piece::{Class, Color, Piece};
use super::moves::Tile;
use super::board_state::{Bitboard,BoardState,CastlingState};

#[derive(Clone)]
pub struct Board {
    boards: [Bitboard; 12],
    mailbox: [Piece; 64],
    side_to_move: Color,
    
    en_passant: Option<Tile>,
    castling_state: CastlingState,

    halfmove_clock: u16,
    fullmove_number: u16,
}

impl Board {
    pub fn new() -> Self {
        let mut boards: [Bitboard; 12] = [0; 12];

        // --- White Pieces (Rank 1 and 2) ---
        boards[0] = 0x0000_0000_0000_FF00; // Pawns
        boards[1] = 0x0000_0000_0000_0042; // Knights (b1, g1)
        boards[2] = 0x0000_0000_0000_0024; // Bishops (c1, f1)
        boards[3] = 0x0000_0000_0000_0081; // Rooks (a1, h1)
        boards[4] = 0x0000_0000_0000_0008; // Queen (d1)
        boards[5] = 0x0000_0000_0000_0010; // King (e1)

        // --- Black Pieces (Rank 7 and 8) ---
        boards[6]  = 0x00FF_0000_0000_0000; // Pawns
        boards[7]  = 0x4200_0000_0000_0000; // Knights (b8, g8)
        boards[8]  = 0x2400_0000_0000_0000; // Bishops (c8, f8)
        boards[9]  = 0x8100_0000_0000_0000; // Rooks (a8, h8)
        boards[10] = 0x0800_0000_0000_0000; // Queen (d8)
        boards[11] = 0x1000_0000_0000_0000; // King (e8)

        let mut mailbox = [Piece::None; 64];

        for piece_idx in 0..12 {
            let mut bitboard = boards[piece_idx];
            
            while bitboard != 0 {
                let sq_idx = bitboard.trailing_zeros() as usize; 
                
                mailbox[sq_idx] = unsafe { std::mem::transmute(piece_idx as u8) }; 
                
                // Clear the lowest set bit
                bitboard &= bitboard - 1; 
            }
        }

        Self {
            boards,
            mailbox,
            side_to_move: Color::White,
            en_passant: None,
            castling_state: CastlingState::ALL,

            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}

impl BoardState for Board {
    #[inline(always)]
    fn get_piece(&self, tile: Tile) -> Piece {
        self.mailbox[tile.get_idx()]
    }

    #[inline(always)]
    fn set_piece(&mut self, tile: Tile, piece: Piece) {
        let idx = tile.get_idx();
        let mask: Bitboard = 1 << idx;

        let old = self.mailbox[idx];
        if !old.is_none() {
            self.boards[old as usize] &= !mask;
        }

        if !piece.is_none() {
            self.boards[piece as usize] |= mask;
        }

        self.mailbox[idx] = piece;
    }

    #[inline(always)]
    fn get_bitboard(&self, class: Class, color: Color) -> Bitboard {
        self.boards[Piece::new(class, color) as usize]
    }

    #[inline(always)]
    fn get_occupied(&self, color: Color) -> Bitboard {
        // Explicitly ORing the bitboards is zero-cost at runtime and avoids looping
        self.get_bitboard(Class::Pawn, color) |
        self.get_bitboard(Class::Knight, color) |
        self.get_bitboard(Class::Bishop, color) |
        self.get_bitboard(Class::Rook, color) |
        self.get_bitboard(Class::Queen, color) |
        self.get_bitboard(Class::King, color)
    }

    #[inline(always)]
    fn get_all_occupied(&self) -> Bitboard {
        self.boards.iter().fold(0, |acc, &b| acc | b)
    }

    #[inline(always)]
    fn get_side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline(always)]
    fn pass_player_turn(&mut self) {
        self.side_to_move = self.side_to_move.opposite();
    }

    #[inline(always)]
    fn get_en_passant(&self) -> Option<Tile> {
        self.en_passant
    }

    #[inline(always)]
    fn set_en_passant(&mut self, tile: Tile) {
        self.en_passant = Some(tile);
    }

    #[inline(always)]
    fn clear_en_passants(&mut self) {
        self.en_passant = None;
    }

    #[inline(always)]
    fn get_castling_state(&self) -> CastlingState{
        self.castling_state
    }

    #[inline(always)]
    fn set_castling_state(&mut self, castling_state: CastlingState) -> (){
        self.castling_state = castling_state;
    }

    #[inline(always)]
    fn get_halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    #[inline(always)]
    fn set_halfmove_clock(&mut self, val: u16) {
        self.halfmove_clock = val;
    }

    #[inline(always)]
    fn get_fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    #[inline(always)]
    fn set_fullmove_number(&mut self, val: u16) {
        self.fullmove_number = val;
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{} ", rank + 1)?;

            for tile in (rank * 8)..(rank * 8 + 8) {
                let piece = self.mailbox[tile];
                let c = if piece.is_none() { '.' } else { piece.to_char() };
                write!(f, "{} ", c)?;
            }

            writeln!(f)?;
        }
        writeln!(f, "  a b c d e f g h")
    }
}