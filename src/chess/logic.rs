use super::piece::{Class, Color, Piece};
use super::moves::{Move,MoveFlag,Tile, MoveInstruction};

// ---------------------------------------------------------------------
// Bitboard attack helpers (plain ray-casting for now; can be replaced
// with magic bitboards / PEXT later without touching the trait surface)
// ---------------------------------------------------------------------

use super::board_state::{BoardState,Bitboard, CastlingRights};
use super::logic_base::{UnmakeInfo, ChessLogic, GameResult};

const FILE_A: Bitboard = 0x0101_0101_0101_0101;
const FILE_B: Bitboard = FILE_A << 1;
const FILE_G: Bitboard = FILE_A << 6;
const FILE_H: Bitboard = FILE_A << 7;
const FILE_AB: Bitboard = FILE_A | FILE_B;
const FILE_GH: Bitboard = FILE_G | FILE_H;

#[inline(always)]
fn knight_attacks(sq: usize) -> Bitboard {
    let b: Bitboard = 1 << sq;
    ((b << 17) & !FILE_A)
        | ((b << 15) & !FILE_H)
        | ((b << 10) & !FILE_AB)
        | ((b << 6) & !FILE_GH)
        | ((b >> 17) & !FILE_H)
        | ((b >> 15) & !FILE_A)
        | ((b >> 10) & !FILE_GH)
        | ((b >> 6) & !FILE_AB)
}

#[inline(always)]
fn king_attacks(sq: usize) -> Bitboard {
    let b: Bitboard = 1 << sq;
    (b << 8)
        | (b >> 8)
        | ((b << 1) & !FILE_A)
        | ((b >> 1) & !FILE_H)
        | ((b << 9) & !FILE_A)
        | ((b << 7) & !FILE_H)
        | ((b >> 7) & !FILE_A)
        | ((b >> 9) & !FILE_H)
}

#[inline(always)]
fn pawn_attacks(sq: usize, color: Color) -> Bitboard {
    let b: Bitboard = 1 << sq;
    match color {
        Color::White => ((b << 7) & !FILE_H) | ((b << 9) & !FILE_A),
        Color::Black => ((b >> 7) & !FILE_A) | ((b >> 9) & !FILE_H),
    }
}

/// Ray-cast in each (file_delta, rank_delta) direction until an occupied
/// square (inclusive) or the board edge is hit.
fn sliding_attacks(sq: usize, occupied: Bitboard, deltas: &[(i8, i8)]) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let file = (sq % 8) as i8;
    let rank = (sq / 8) as i8;

    for &(df, dr) in deltas {
        let mut f = file + df;
        let mut r = rank + dr;
        while (0..8).contains(&f) && (0..8).contains(&r) {
            let idx = (r * 8 + f) as usize;
            attacks |= 1 << idx;
            if occupied & (1 << idx) != 0 {
                break;
            }
            f += df;
            r += dr;
        }
    }
    attacks
}

#[inline(always)]
fn bishop_attacks(sq: usize, occupied: Bitboard) -> Bitboard {
    sliding_attacks(sq, occupied, &[(1, 1), (1, -1), (-1, 1), (-1, -1)])
}

#[inline(always)]
fn rook_attacks(sq: usize, occupied: Bitboard) -> Bitboard {
    sliding_attacks(sq, occupied, &[(1, 0), (-1, 0), (0, 1), (0, -1)])
}

#[inline(always)]
fn queen_attacks(sq: usize, occupied: Bitboard) -> Bitboard {
    bishop_attacks(sq, occupied) | rook_attacks(sq, occupied)
}

/// Squares set in `bb`, as an iterator of tile indices.
fn iter_bits(mut bb: Bitboard) -> impl Iterator<Item = usize> {
    std::iter::from_fn(move || {
        if bb == 0 {
            None
        } else {
            let idx = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            Some(idx)
        }
    })
}


impl<T: BoardState + Clone> ChessLogic for T {
    fn generate_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let side = self.get_side_to_move();
        let own_occupied = self.get_occupied(side);
        let enemy_occupied = self.get_occupied(side.opposite());
        let all_occupied = self.get_all_occupied();

        // --- Pawns ---
        let pawns = self.get_bitboard(Class::Pawn, side);
        let (push_dir, start_rank, promo_rank): (i32, i32, i32) = match side {
            Color::White => (8, 1, 7),
            Color::Black => (-8, 6, 0),
        };

        for from_idx in iter_bits(pawns) {
            // let (file,rank) = Tile::from_idx(from_idx as u8).get_file_rank();
            let from = Tile::from_idx(from_idx as u8);
            let file = (from_idx % 8) as i32;
            let rank = (from_idx / 8) as i32;

            // Single push
            let one_idx = from_idx as i32 + push_dir;
            if (0..64).contains(&one_idx) && self.get_piece(Tile::from_idx(one_idx as u8)).is_none() {
                let to = Tile::from_idx(one_idx as u8);
                let to_rank = one_idx / 8;
                push_pawn_move(&mut moves, from, to, to_rank == promo_rank, false);

                // Double push
                if rank == start_rank {
                    let two_idx = from_idx as i32 + push_dir * 2;
                    if self.get_piece(Tile::from_idx(two_idx as u8)).is_none() {
                        moves.push(Move::new(from, Tile::from_idx(two_idx as u8), MoveFlag::DoublePawnPush));
                    }
                }
            }

            // Captures (incl. en passant)
            for (df, cap_dir) in [(-1, push_dir - 1), (1, push_dir + 1)] {
                let target_file = file + df;
                if !(0..8).contains(&target_file) {
                    continue;
                }
                let target_idx = from_idx as i32 + cap_dir;
                if !(0..64).contains(&target_idx) {
                    continue;
                }
                let to = Tile::from_idx(target_idx as u8);
                let to_rank = target_idx / 8;

                if let Some(ep) = self.get_en_passant() {
                    if ep == to {
                        moves.push(Move::new(from, to, MoveFlag::EpCapture));
                        continue;
                    }
                }

                let target_piece = self.get_piece(to);
                if !target_piece.is_none() && target_piece.color() != side {
                    push_pawn_move(&mut moves, from, to, to_rank == promo_rank, true);
                }
            }
        }

        // --- Knights ---
        for from_idx in iter_bits(self.get_bitboard(Class::Knight, side)) {
            let attacks = knight_attacks(from_idx) & !own_occupied;
            push_stepper_moves(self, &mut moves, from_idx, attacks, enemy_occupied);
        }

        // --- Bishops ---
        for from_idx in iter_bits(self.get_bitboard(Class::Bishop, side)) {
            let attacks = bishop_attacks(from_idx, all_occupied) & !own_occupied;
            push_stepper_moves(self, &mut moves, from_idx, attacks, enemy_occupied);
        }

        // --- Rooks ---
        for from_idx in iter_bits(self.get_bitboard(Class::Rook, side)) {
            let attacks = rook_attacks(from_idx, all_occupied) & !own_occupied;
            push_stepper_moves(self, &mut moves, from_idx, attacks, enemy_occupied);
        }

        // --- Queens ---
        for from_idx in iter_bits(self.get_bitboard(Class::Queen, side)) {
            let attacks = queen_attacks(from_idx, all_occupied) & !own_occupied;
            push_stepper_moves(self, &mut moves, from_idx, attacks, enemy_occupied);
        }

        // --- King (non-castling) ---
        for from_idx in iter_bits(self.get_bitboard(Class::King, side)) {
            let attacks = king_attacks(from_idx) & !own_occupied;
            push_stepper_moves(self, &mut moves, from_idx, attacks, enemy_occupied);
        }

        // --- Castling ---
        generate_castling_moves(self, &mut moves, side, all_occupied);

        moves
    }

    fn generate_legal_moves(&self) -> Vec<Move> {
        let side = self.get_side_to_move();
        let pseudo = self.generate_pseudo_legal_moves();
        let mut legal = Vec::with_capacity(pseudo.len());

        // Cloning gives us a genuinely owned, mutable scratch board to
        // make/unmake on, so legality checks never need unsafe aliasing
        // tricks. This does cost a clone per call; if that ever shows up
        // in profiling, consider a make/unmake-based check that only
        // clones once per generate_legal_moves call (already the case
        // here) or caching attack info.
        let mut scratch = self.clone();

        for mv in pseudo {
            let info = scratch.make_move(mv);
            let king_safe = !scratch.is_in_check(side);
            scratch.unmake_move(mv, info);
            if king_safe {
                legal.push(mv);
            }
        }

        legal
    }

    fn generate_moves_for_tile(&self, tile: Tile) -> Vec<Move> {
        self.generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.from() == tile)
            .collect()
    }

    fn is_square_attacked(&self, tile: Tile, by_color: Color) -> bool {
        self.get_attackers_of(tile, by_color) != 0
    }

    fn get_attackers_of(&self, tile: Tile, by_color: Color) -> Bitboard {
        let idx = tile.get_idx();
        let occupied = self.get_all_occupied();

        let mut attackers: Bitboard = 0;
        attackers |= pawn_attacks(idx, by_color.opposite()) & self.get_bitboard(Class::Pawn, by_color);
        attackers |= knight_attacks(idx) & self.get_bitboard(Class::Knight, by_color);
        attackers |= king_attacks(idx) & self.get_bitboard(Class::King, by_color);
        attackers |= bishop_attacks(idx, occupied)
            & (self.get_bitboard(Class::Bishop, by_color) | self.get_bitboard(Class::Queen, by_color));
        attackers |= rook_attacks(idx, occupied)
            & (self.get_bitboard(Class::Rook, by_color) | self.get_bitboard(Class::Queen, by_color));

        attackers
    }

    fn is_in_check(&self, color: Color) -> bool {
        self.is_square_attacked(self.king_tile(color), color.opposite())
    }

    fn king_tile(&self, color: Color) -> Tile {
        let bb = self.get_bitboard(Class::King, color);
        debug_assert!(bb != 0, "king missing from board for {:?}", color);
        Tile::from_idx(bb.trailing_zeros() as u8)
    }

    fn make_move(&mut self, mv: Move) -> UnmakeInfo {
        let from = mv.from();
        let to = mv.to();
        let flag = mv.flag();
        let moving_piece = self.get_piece(from);
        let side = moving_piece.color();

        let prev_castling_rights = self.get_castling_state();
        let prev_en_passant = self.get_en_passant();
        let prev_halfmove_clock = self.get_halfmove_clock();
        let prev_fullmove_number = self.get_fullmove_number();

        let mut captured = self.get_piece(to);
        let mut captured_ep_tile = None;

        if flag == MoveFlag::EpCapture {
            let cap_idx = if side == Color::White { to.get_idx() - 8 } else { to.get_idx() + 8 };
            let cap_tile = Tile::from_idx(cap_idx as u8);
            captured = self.get_piece(cap_tile);
            captured_ep_tile = Some(cap_tile);
            self.set_piece(cap_tile, Piece::None);
        }

        self.set_piece(from, Piece::None);

        let placed_piece = match flag {
            MoveFlag::KnightPromo | MoveFlag::KnightPromoCapture => Piece::new(Class::Knight, side),
            MoveFlag::BishopPromo | MoveFlag::BishopPromoCapture => Piece::new(Class::Bishop, side),
            MoveFlag::RookPromo | MoveFlag::RookPromoCapture => Piece::new(Class::Rook, side),
            MoveFlag::QueenPromo | MoveFlag::QueenPromoCapture => Piece::new(Class::Queen, side),
            _ => moving_piece,
        };
        self.set_piece(to, placed_piece);

        match flag {
            MoveFlag::KingCastle => match side {
                Color::White => relocate_rook(self, 7, 5),
                Color::Black => relocate_rook(self, 63, 61),
            },
            MoveFlag::QueenCastle => match side {
                Color::White => relocate_rook(self, 0, 3),
                Color::Black => relocate_rook(self, 56, 59),
            },
            _ => {}
        }

        // Castling rights updates
        match moving_piece {
            Piece::WhiteKing => {
                block_castling(self, CastlingRights::WhiteKing);
                block_castling(self, CastlingRights::WhiteQueen);
            }
            Piece::BlackKing => {
                block_castling(self, CastlingRights::BlackKing);
                block_castling(self, CastlingRights::BlackQueen);
            }
            _ => {}
        }
        let (fi, ti) = (from.get_idx(), to.get_idx());
        if fi == 0 || ti == 0 { block_castling(self, CastlingRights::WhiteQueen); }
        if fi == 7 || ti == 7 { block_castling(self, CastlingRights::WhiteKing); }
        if fi == 56 || ti == 56 { block_castling(self, CastlingRights::BlackQueen); }
        if fi == 63 || ti == 63 { block_castling(self, CastlingRights::BlackKing); }

        // En passant target
        self.clear_en_passants();
        if flag == MoveFlag::DoublePawnPush {
            let ep_idx = if side == Color::White { from.get_idx() + 8 } else { from.get_idx() - 8 };
            self.set_en_passant(Tile::from_idx(ep_idx as u8));
        }

        // Halfmove clock
        let is_capture = !captured.is_none() || flag == MoveFlag::EpCapture;
        if moving_piece.class() == Class::Pawn || is_capture {
            self.set_halfmove_clock(0);
        } else {
            self.set_halfmove_clock(prev_halfmove_clock + 1);
        }

        // Fullmove number increments after Black's move
        if side == Color::Black {
            self.set_fullmove_number(prev_fullmove_number + 1);
        }

        self.pass_player_turn();

        UnmakeInfo {
            captured,
            captured_ep_tile,
            prev_castling_rights,
            prev_en_passant,
            prev_halfmove_clock,
            prev_fullmove_number,
        }
    }

    fn unmake_move(&mut self, mv: Move, info: UnmakeInfo) {
        self.pass_player_turn(); // back to the side that made the move

        let from = mv.from();
        let to = mv.to();
        let flag = mv.flag();
        let side = self.get_side_to_move();

        let is_promo = matches!(
            flag,
            MoveFlag::KnightPromo | MoveFlag::KnightPromoCapture |
            MoveFlag::BishopPromo | MoveFlag::BishopPromoCapture |
            MoveFlag::RookPromo | MoveFlag::RookPromoCapture |
            MoveFlag::QueenPromo | MoveFlag::QueenPromoCapture
        );
        let original_piece = if is_promo { Piece::new(Class::Pawn, side) } else { self.get_piece(to) };

        self.set_piece(to, Piece::None);
        self.set_piece(from, original_piece);

        if flag == MoveFlag::EpCapture {
            if let Some(cap_tile) = info.captured_ep_tile {
                self.set_piece(cap_tile, info.captured);
            }
        } else {
            self.set_piece(to, info.captured);
        }

        match flag {
            MoveFlag::KingCastle => match side {
                Color::White => relocate_rook(self, 5, 7),
                Color::Black => relocate_rook(self, 61, 63),
            },
            MoveFlag::QueenCastle => match side {
                Color::White => relocate_rook(self, 3, 0),
                Color::Black => relocate_rook(self, 59, 56),
            },
            _ => {}
        }

        self.set_castling_state(info.prev_castling_rights);
        self.clear_en_passants();
        if let Some(ep) = info.prev_en_passant {
            self.set_en_passant(ep);
        }
        self.set_halfmove_clock(info.prev_halfmove_clock);
        self.set_fullmove_number(info.prev_fullmove_number);
    }

    fn resolve_move(&self, instr: MoveInstruction) -> Result<Move, String> {
        self.generate_legal_moves()
            .into_iter()
            .find(|mv| {
                mv.from() == instr.from
                    && mv.to() == instr.to
                    && promo_class_of(mv.flag()) == instr.promotion
            })
            .ok_or_else(|| format!(
                "Illegal move: {}{}{}",
                instr.from,
                instr.to,
                instr.promotion.map(|c| format!("{:?}", c)).unwrap_or_default()
            ))
    }

    fn is_legal(&self, mv: Move) -> bool {
        self.generate_legal_moves().contains(&mv)
    }

    fn is_checkmate(&self) -> bool {
        self.is_in_check(self.get_side_to_move()) && self.generate_legal_moves().is_empty()
    }

    fn is_stalemate(&self) -> bool {
        !self.is_in_check(self.get_side_to_move()) && self.generate_legal_moves().is_empty()
    }

    fn is_draw_by_insufficient_material(&self) -> bool {
        // Any pawn, rook, or queen on the board means there's still mating material.
        let heavy_or_pawns = self.get_bitboard(Class::Pawn, Color::White)
            | self.get_bitboard(Class::Pawn, Color::Black)
            | self.get_bitboard(Class::Rook, Color::White)
            | self.get_bitboard(Class::Rook, Color::Black)
            | self.get_bitboard(Class::Queen, Color::White)
            | self.get_bitboard(Class::Queen, Color::Black);
        if heavy_or_pawns != 0 {
            return false;
        }

        let minor_count = |c: Color| {
            self.get_bitboard(Class::Knight, c).count_ones() + self.get_bitboard(Class::Bishop, c).count_ones()
        };
        let white_minors = minor_count(Color::White);
        let black_minors = minor_count(Color::Black);

        // K vs K, K+minor vs K, are draws. K+2N vs K is not forced mate but
        // also not automatically declared here (rare edge case, treat as playable).
        (white_minors == 0 && black_minors == 0)
            || (white_minors == 1 && black_minors == 0 && self.get_bitboard(Class::Knight, Color::White).count_ones() <= 1)
            || (black_minors == 1 && white_minors == 0 && self.get_bitboard(Class::Knight, Color::Black).count_ones() <= 1)
    }

    fn is_draw_by_fifty_move_rule(&self) -> bool {
        self.get_halfmove_clock() >= 100
    }

    fn game_result(&self) -> GameResult {
        if self.is_checkmate() {
            return GameResult::Checkmate(self.get_side_to_move().opposite());
        }
        if self.is_stalemate() {
            return GameResult::Stalemate;
        }
        if self.is_draw_by_fifty_move_rule() {
            return GameResult::DrawByFiftyMoveRule;
        }
        if self.is_draw_by_insufficient_material() {
            return GameResult::DrawByInsufficientMaterial;
        }
        GameResult::Ongoing
    }

    fn perft(&mut self, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }
        let moves = self.generate_legal_moves();
        if depth == 1 {
            return moves.len() as u64;
        }
        let mut nodes = 0u64;
        for mv in moves {
            let info = self.make_move(mv);
            nodes += self.perft(depth - 1);
            self.unmake_move(mv, info);
        }
        nodes
    }
}

// ---------------------------------------------------------------------
// Free helper functions used by the blanket impl
// ---------------------------------------------------------------------

fn push_pawn_move(moves: &mut Vec<Move>, from: Tile, to: Tile, is_promo: bool, is_capture: bool) {
    if is_promo {
        let flags = if is_capture {
            [MoveFlag::KnightPromoCapture, MoveFlag::BishopPromoCapture, MoveFlag::RookPromoCapture, MoveFlag::QueenPromoCapture]
        } else {
            [MoveFlag::KnightPromo, MoveFlag::BishopPromo, MoveFlag::RookPromo, MoveFlag::QueenPromo]
        };
        for flag in flags {
            moves.push(Move::new(from, to, flag));
        }
    } else {
        let flag = if is_capture { MoveFlag::Capture } else { MoveFlag::Quiet };
        moves.push(Move::new(from, to, flag));
    }
}

fn push_stepper_moves<T: BoardState + ?Sized>(
    board: &T,
    moves: &mut Vec<Move>,
    from_idx: usize,
    targets: Bitboard,
    enemy_occupied: Bitboard,
) {
    let from = Tile::from_idx(from_idx as u8);
    for to_idx in iter_bits(targets) {
        let to = Tile::from_idx(to_idx as u8);
        let flag = if (1u64 << to_idx) & enemy_occupied != 0 {
            MoveFlag::Capture
        } else {
            MoveFlag::Quiet
        };
        let _ = board; // board only needed if we later special-case something
        moves.push(Move::new(from, to, flag));
    }
}

fn relocate_rook<T: BoardState + ?Sized>(board: &mut T, from_idx: u8, to_idx: u8) {
    let rook = board.get_piece(Tile::from_idx(from_idx));
    board.set_piece(Tile::from_idx(from_idx), Piece::None);
    board.set_piece(Tile::from_idx(to_idx), rook);
}

fn generate_castling_moves<T: BoardState + ChessLogic + ?Sized>(
    board: &T,
    moves: &mut Vec<Move>,
    side: Color,
    all_occupied: Bitboard,
) {
    if board.is_in_check(side) {
        return;
    }

    let castling_state = board.get_castling_state();

    match side {
        Color::White => {
            if castling_state.contains(CastlingRights::WhiteKing)
                && (all_occupied & ((1 << 5) | (1 << 6))) == 0
                && !board.is_square_attacked(Tile::from_idx(5), Color::Black)
                && !board.is_square_attacked(Tile::from_idx(6), Color::Black)
            {
                moves.push(Move::new(Tile::from_idx(4), Tile::from_idx(6), MoveFlag::KingCastle));
            }
            if castling_state.contains(CastlingRights::WhiteQueen)
                && (all_occupied & ((1 << 1) | (1 << 2) | (1 << 3))) == 0
                && !board.is_square_attacked(Tile::from_idx(3), Color::Black)
                && !board.is_square_attacked(Tile::from_idx(2), Color::Black)
            {
                moves.push(Move::new(Tile::from_idx(4), Tile::from_idx(2), MoveFlag::QueenCastle));
            }
        }
        Color::Black => {
            if castling_state.contains(CastlingRights::BlackKing)
                && (all_occupied & ((1 << 61) | (1 << 62))) == 0
                && !board.is_square_attacked(Tile::from_idx(61), Color::White)
                && !board.is_square_attacked(Tile::from_idx(62), Color::White)
            {
                moves.push(Move::new(Tile::from_idx(60), Tile::from_idx(62), MoveFlag::KingCastle));
            }
            if castling_state.contains(CastlingRights::BlackQueen)
                && (all_occupied & ((1 << 57) | (1 << 58) | (1 << 59))) == 0
                && !board.is_square_attacked(Tile::from_idx(59), Color::White)
                && !board.is_square_attacked(Tile::from_idx(58), Color::White)
            {
                moves.push(Move::new(Tile::from_idx(60), Tile::from_idx(58), MoveFlag::QueenCastle));
            }
        }
    }
}

/// Removes a single castling right from the board's current castling state.
fn block_castling<T: BoardState + ?Sized>(board: &mut T, right: CastlingRights) {
    let mut state = board.get_castling_state();
    state.remove(right);
    board.set_castling_state(state);
}

fn promo_class_of(flag: MoveFlag) -> Option<Class> {
    match flag {
        MoveFlag::KnightPromo | MoveFlag::KnightPromoCapture => Some(Class::Knight),
        MoveFlag::BishopPromo | MoveFlag::BishopPromoCapture => Some(Class::Bishop),
        MoveFlag::RookPromo | MoveFlag::RookPromoCapture => Some(Class::Rook),
        MoveFlag::QueenPromo | MoveFlag::QueenPromoCapture => Some(Class::Queen),
        _ => None,
    }
}


