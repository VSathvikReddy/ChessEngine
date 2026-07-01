use super::piece::{Class, Color, Piece};
use super::moves::Tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CastlingRights{
    WhiteKing   = 0b0001,
    WhiteQueen  = 0b0010,
    BlackKing   = 0b0100,
    BlackQueen  = 0b1000,
}
impl CastlingRights {
    pub const ALL: [CastlingRights; 4] = [
        CastlingRights::WhiteKing,
        CastlingRights::WhiteQueen,
        CastlingRights::BlackKing,
        CastlingRights::BlackQueen,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingState {
    mask: u8,
}

impl CastlingState {
    pub const NONE: Self = Self { mask: 0 };
    pub const ALL: Self = Self { mask: 0b1111 };

    #[inline(always)]
    pub fn new(mask: u8) -> Self {
        Self { mask: mask & 0b1111 }
    }

    #[inline(always)]
    pub fn contains(&self, right: CastlingRights) -> bool {
        (self.mask & (right as u8)) != 0
    }

    #[inline(always)]
    pub fn remove(&mut self, right: CastlingRights) {
        self.mask &= !(right as u8);
    }

    #[inline(always)]
    pub fn add(&mut self, right: CastlingRights) {
        self.mask |= right as u8;
    }

    #[inline(always)]
    pub fn as_u8(&self) -> u8 {
        self.mask
    }
}


pub type Bitboard = u64;
pub trait BoardState {
    fn get_piece(&self, tile: Tile) -> Piece;
    fn set_piece(&mut self, tile: Tile, piece: Piece) -> ();

    fn get_bitboard(&self, class: Class, color: Color) -> Bitboard;
    fn get_occupied(&self, color: Color) -> Bitboard;
    fn get_all_occupied(&self) -> Bitboard;

    fn get_side_to_move(&self) -> Color;
    fn pass_player_turn(&mut self) -> ();

    fn get_en_passant(&self) -> Option<Tile>;
    fn set_en_passant(&mut self, tile: Tile);
    fn clear_en_passants(&mut self);

    fn get_castling_state(&self) -> CastlingState;
    fn set_castling_state(&mut self, castling_state: CastlingState) -> ();
 
    fn get_halfmove_clock(&self) -> u16;
    fn set_halfmove_clock(&mut self, val: u16);
 
    fn get_fullmove_number(&self) -> u16;
    fn set_fullmove_number(&mut self, val: u16);
}