use std::io::{self, Write};

use chess::chess::board::Board;
use chess::chess::board_state::BoardState;
use chess::chess::logic_base::{ChessLogic, GameResult};
use chess::chess::moves::MoveInstruction;
use chess::chess::piece::Color;

/// Tiny dependency-free xorshift64 PRNG so we don't need the `rand` crate
/// just to pick a random legal move for the AI side.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_index(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }
}

fn seed_from_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xDEAD_BEEF_CAFE_F00D)
}

/// Print any pending game-over result and return true if the game ended.
fn report_if_over(board: &Board, ply: u32) -> bool {
    match board.game_result() {
        GameResult::Checkmate(winner) => {
            println!("Checkmate! {winner:?} wins after {ply} ply.");
            true
        }
        GameResult::Stalemate => {
            println!("Stalemate — draw after {ply} ply.");
            true
        }
        GameResult::DrawByFiftyMoveRule => {
            println!("Draw by fifty-move rule after {ply} ply.");
            true
        }
        GameResult::DrawByInsufficientMaterial => {
            println!("Draw by insufficient material after {ply} ply.");
            true
        }
        GameResult::Ongoing => false,
    }
}

/// Read one line of input from stdin, trimmed. Returns None on EOF.
fn read_line(prompt: &str) -> Option<String> {
    print!("{prompt}");
    io::stdout().flush().ok();

    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => None, // EOF (e.g. piped input ran out, or Ctrl-D)
        Ok(_) => Some(line.trim().to_string()),
        Err(_) => None,
    }
}

fn main() {
    let mut board = Board::new();
    let mut rng = Rng::new(seed_from_time());
    let mut ply: u32 = 0;

    println!("You are White. Enter moves like \"e2e4\" or a promotion like \"e7e8q\".");
    println!("Type \"moves\" to list your legal moves, or \"quit\" to exit.\n");
    println!("{board}");

    loop {
        if report_if_over(&board, ply) {
            break;
        }

        match board.get_side_to_move() {
            Color::White => {
                // --- Human turn ---
                let legal_moves = board.generate_legal_moves();
                // Should be unreachable since report_if_over() already
                // caught checkmate/stalemate, but guard anyway.
                if legal_moves.is_empty() {
                    println!("No legal moves available; stopping.");
                    break;
                }

                let Some(input) = read_line("Your move: ") else {
                    println!("\nInput closed, exiting.");
                    break;
                };

                if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                if input.eq_ignore_ascii_case("moves") {
                    let list: Vec<String> = legal_moves
                        .iter()
                        .map(|mv| format!("{}{}", mv.from(), mv.to()))
                        .collect();
                    println!("Legal moves: {}", list.join(", "));
                    continue;
                }

                let instr: MoveInstruction = match input.parse() {
                    Ok(instr) => instr,
                    Err(err) => {
                        println!("{err}");
                        continue;
                    }
                };

                let mv = match board.resolve_move(instr) {
                    Ok(mv) => mv,
                    Err(err) => {
                        println!("{err}");
                        continue;
                    }
                };

                board.make_move(mv);
                ply += 1;
                println!("{board}");
            }

            Color::Black => {
                // --- AI turn: uniformly random legal move ---
                let legal_moves = board.generate_legal_moves();
                if legal_moves.is_empty() {
                    println!("No legal moves available; stopping.");
                    break;
                }

                let mv = legal_moves[rng.next_index(legal_moves.len())];
                board.make_move(mv);
                ply += 1;

                println!("AI plays: {}{}", mv.from(), mv.to());
                println!("{board}");
            }
        }
    }
}