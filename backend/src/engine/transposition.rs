/// Zobrist hashing and transposition table.
///
/// Each board position maps to a unique 64-bit hash computed by XOR-ing
/// random per-piece-per-square values. The transposition table caches
/// search evaluations keyed by this hash, short-circuiting redundant
/// subtree evaluation for transposed positions.

use super::bitboard::{Board, Color, Piece, sq_bb, pop_lsb};
use std::sync::OnceLock;

// ── Random tables ─────────────────────────────────────────────────────────────

struct ZobristKeys {
    /// [color][piece][square]
    piece: [[[u64; 64]; 6]; 2],
    side_to_move: u64,
    castling: [u64; 16],
    en_passant: [u64; 8], // file index
}

static ZOBRIST: OnceLock<ZobristKeys> = OnceLock::new();

fn keys() -> &'static ZobristKeys {
    ZOBRIST.get_or_init(|| {
        // Deterministic PRNG seeded with a fixed value for reproducibility.
        let mut state: u64 = 0xDEAD_BEEF_CAFE_1234;
        let mut next = || -> u64 {
            // xoshiro-style shift
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };

        let mut piece = [[[0u64; 64]; 6]; 2];
        for c in 0..2 {
            for p in 0..6 {
                for sq in 0..64 {
                    piece[c][p][sq] = next();
                }
            }
        }
        ZobristKeys {
            piece,
            side_to_move: next(),
            castling: std::array::from_fn(|_| next()),
            en_passant: std::array::from_fn(|_| next()),
        }
    })
}

/// Compute a full Zobrist hash for a board position (O(pieces)).
pub fn hash(board: &Board) -> u64 {
    let k = keys();
    let mut h: u64 = 0;

    for c in 0..2usize {
        for p in 0..6usize {
            let mut bb = board.pieces[c][p];
            while bb != 0 {
                let sq = pop_lsb(&mut bb) as usize;
                h ^= k.piece[c][p][sq];
            }
        }
    }

    if board.side_to_move == Color::Black {
        h ^= k.side_to_move;
    }

    h ^= k.castling[board.castling_rights as usize];

    if let Some(ep_sq) = board.en_passant {
        h ^= k.en_passant[(ep_sq % 8) as usize];
    }

    h
}

/// Incrementally update a hash after moving a piece from `sq` to `to`.
/// Caller supplies the piece and color; captures handled separately.
#[inline]
pub fn update_piece(h: &mut u64, c: Color, p: Piece, from: u32, to: u32) {
    let k = keys();
    *h ^= k.piece[c as usize][p as usize][from as usize];
    *h ^= k.piece[c as usize][p as usize][to as usize];
}

#[inline]
pub fn toggle_side(h: &mut u64) {
    *h ^= keys().side_to_move;
}

#[inline]
pub fn update_castling(h: &mut u64, old: u8, new: u8) {
    let k = keys();
    *h ^= k.castling[old as usize];
    *h ^= k.castling[new as usize];
}

// ── Transposition table ───────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bound {
    Exact,
    LowerBound, // alpha cutoff (fail-low)
    UpperBound, // beta  cutoff (fail-high)
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u8,
    pub score: i32,
    pub bound: Bound,
    pub best_from: u8,
    pub best_to: u8,
}

/// Fixed-capacity transposition table. Indexed by `key % capacity`.
/// No dynamic allocation after construction; entries are overwritten by
/// depth-preference replacement.
pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    capacity: usize,
    pub hits: u64,
    pub stores: u64,
}

impl TranspositionTable {
    /// Create a table with `size_mb` megabytes of capacity.
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let capacity = (size_mb * 1024 * 1024) / entry_size;
        TranspositionTable {
            entries: vec![None; capacity],
            capacity,
            hits: 0,
            stores: 0,
        }
    }

    pub fn probe(&mut self, key: u64) -> Option<&TTEntry> {
        let idx = (key as usize) % self.capacity;
        if let Some(entry) = &self.entries[idx] {
            if entry.key == key {
                self.hits += 1;
                return self.entries[idx].as_ref();
            }
        }
        None
    }

    /// Store an entry, preferring entries at greater depth.
    pub fn store(&mut self, key: u64, depth: u8, score: i32, bound: Bound, from: u8, to: u8) {
        let idx = (key as usize) % self.capacity;
        let replace = match &self.entries[idx] {
            None => true,
            Some(e) => depth >= e.depth || e.key != key,
        };
        if replace {
            self.stores += 1;
            self.entries[idx] = Some(TTEntry {
                key,
                depth,
                score,
                bound,
                best_from: from,
                best_to: to,
            });
        }
    }

    pub fn clear(&mut self) {
        self.entries.iter_mut().for_each(|e| *e = None);
        self.hits = 0;
        self.stores = 0;
    }

    pub fn hit_rate(&self) -> f64 {
        let probes = self.hits + (self.stores - self.hits).max(0);
        if probes == 0 { 0.0 } else { self.hits as f64 / probes as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::bitboard::Board;

    #[test]
    fn startpos_hash_stable() {
        let b = Board::startpos();
        assert_eq!(hash(&b), hash(&b));
    }

    #[test]
    fn tt_store_and_probe() {
        let mut tt = TranspositionTable::new(1);
        tt.store(0xCAFE_BABE, 4, 100, Bound::Exact, 12, 28);
        let entry = tt.probe(0xCAFE_BABE).expect("entry should exist");
        assert_eq!(entry.score, 100);
        assert_eq!(entry.depth, 4);
    }
}
