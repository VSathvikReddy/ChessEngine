use super::piece::{Piece,Color};
use super::moves::{Move, MoveInstruction,Tile};
use super::board_state::{BoardState, Bitboard,CastlingState};


#[derive(Debug, Clone, Copy)]
pub struct UnmakeInfo {
    pub captured: Piece,
    pub captured_ep_tile: Option<Tile>,
    pub prev_castling_rights: CastlingState,
    pub prev_en_passant: Option<Tile>,

    pub prev_halfmove_clock: u16, // Halfmove clock (for 50-move rule) before the move.
    pub prev_fullmove_number: u16, // Fullmove number before the move (only increments after Black moves).
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Ongoing,
    Checkmate(Color), // winning color
    Stalemate,
    DrawByFiftyMoveRule,
    DrawByInsufficientMaterial,
}


pub trait ChessLogic: BoardState {
    // --- Move generation ---
    fn generate_pseudo_legal_moves(&self) -> Vec<Move>;
    fn generate_legal_moves(&self) -> Vec<Move>;
    fn generate_moves_for_tile(&self, tile: Tile) -> Vec<Move>;

    // --- Attack / check detection ---
    fn is_square_attacked(&self, tile: Tile, by_color: Color) -> bool;
    fn get_attackers_of(&self, tile: Tile, by_color: Color) -> Bitboard;
    fn is_in_check(&self, color: Color) -> bool;
    fn king_tile(&self, color: Color) -> Tile;

    // --- Move application ---
    fn make_move(&mut self, mv: Move) -> UnmakeInfo;
    fn unmake_move(&mut self, mv: Move, info: UnmakeInfo);

    // --- Move construction / validation ---
    fn resolve_move(&self, instr: MoveInstruction) -> Result<Move, String>;
    // (turns a raw from/to/promotion into a validated internal Move w/ correct flag)

    fn is_legal(&self, mv: Move) -> bool;

    // --- Game state queries ---
    fn is_checkmate(&self) -> bool;
    fn is_stalemate(&self) -> bool;
    fn is_draw_by_insufficient_material(&self) -> bool;
    fn is_draw_by_fifty_move_rule(&self) -> bool;
    fn game_result(&self) -> GameResult;

    // --- Perft / debugging ---
    fn perft(&mut self, depth: u32) -> u64;
}