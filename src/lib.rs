pub mod move_ordering;
pub mod nnue;
pub mod position;
pub mod search;
pub mod uci;

use std::{fmt, iter::Peekable, str::SplitWhitespace};

use cozy_chess::{Board, Color, File, Move, Piece, Square};
use position::Position;
use uci::Uci;

const MAX_DEPTH: u8 = 100;

pub struct Tsunami {
    pub pos: Position,
}

impl Tsunami {
    pub fn new(fen: &str) -> Self {
        Self {
            pos: Position::from_fen(fen),
        }
    }
}

impl Default for Tsunami {
    fn default() -> Self {
        Self {
            pos: Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        }
    }
}

impl Uci for Tsunami {
    fn go(&mut self, args: &mut Peekable<SplitWhitespace>) {
        let mut wtime: u128 = 10000;
        let mut btime: u128 = 10000;
        let mut move_time: Option<u128> = None;
        while let Some(arg) = args.next() {
            match arg {
                "perft" => {
                    let depth: u8 = args.next().unwrap().parse().unwrap_or(1);
                    let now = std::time::Instant::now();
                    let nodes = crate::perft(&self.pos.board, depth);
                    let elapsed = now.elapsed().as_secs_f64();
                    self.send(&format!("Nodes: {}", nodes));
                    self.send(&format!(
                        "mnps: {}",
                        ((nodes as f64 / elapsed) / 1_000_000.0).floor()
                    ));
                    return;
                }
                "wtime" => {
                    wtime = args.next().unwrap().parse().unwrap();
                }
                "btime" => {
                    btime = args.next().unwrap().parse().unwrap();
                }
                "winc" => {
                    let _winc: u64 = args.next().unwrap().parse().unwrap();
                }
                "binc" => {
                    let _binc: u64 = args.next().unwrap().parse().unwrap();
                }
                "movetime" => {
                    move_time = Some(args.next().unwrap().parse().unwrap());
                }
                _ => {
                    self.unknown();
                }
            }
        }

        // Basic time management: use 1% of the time left per move
        // TODO: Better time management
        let time_left_millis = match self.pos.board.side_to_move() {
            Color::White => wtime,
            Color::Black => btime,
        };

        if let Some(move_time) = move_time {
            crate::search::think(&mut self.pos, MAX_DEPTH, move_time);
            return;
        }

        crate::search::think(&mut self.pos, MAX_DEPTH, time_left_millis / 100);
    }

    fn display_eval(&mut self) {
        self.send(&format!("Eval: {}cp", self.pos.eval()));
    }

    fn set_position(&mut self, args: &mut Peekable<SplitWhitespace>) {
        match args.next() {
            Some("startpos") => {
                self.pos =
                    Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
            }
            Some("fen") => {
                let mut fen = String::new();
                for arg in args.by_ref() {
                    if arg == "moves" {
                        break;
                    }
                    fen.push_str(arg);
                    fen.push(' ');
                }
                self.pos = Position::from_fen(fen.trim_end());
            }
            _ => {
                self.unknown();
            }
        }

        // Play the moves
        for mv in args.by_ref() {
            if mv == "moves" {
                continue;
            }

            let move_to_play = mv.parse().unwrap();
            self.pos
                .make_move(from_uci_castling(&self.pos.board, move_to_play));
        }
    }

    fn display(&self) {
        self.send(&format!("{}", self));
        self.send(&format!("FEN: {}", self.pos.board));
    }
}

impl fmt::Display for Tsunami {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n+---+---+---+---+---+---+---+---+")?;

        // Iterate over the squares of the board
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = Square::index(rank * 8 + file);
                let pc = self.pos.board.piece_on(sq);

                // Print the char representation of the piece, or a blank space
                match pc {
                    Some(pc) => {
                        write!(
                            f,
                            "| {} ",
                            match self.pos.board.color_on(sq) {
                                Some(color) => match color {
                                    Color::White => format!("{}", pc).to_uppercase(),
                                    Color::Black => format!("{}", pc),
                                },
                                None => String::from("?"),
                            }
                        )?;
                    }
                    None => {
                        write!(f, "|   ")?;
                    }
                }
            }
            writeln!(f, "| {} \n+---+---+---+---+---+---+---+---+", rank + 1)?;
        }
        writeln!(f, "  a   b   c   d   e   f   g   h ")?;

        Ok(())
    }
}

/// Count the number of possible positions at a given depth
/// Implementation from cozy-chess
fn perft(board: &Board, depth: u8) -> u64 {
    let mut nodes = 0;
    match depth {
        0 => nodes += 1,
        1 => {
            board.generate_moves(|moves| {
                nodes += moves.len() as u64;
                false
            });
        }
        _ => {
            board.generate_moves(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.play_unchecked(mv);
                    let child_nodes = perft(&board, depth - 1);
                    nodes += child_nodes;
                }
                false
            });
        }
    }
    nodes
}

/// Convert a move from UCI format to the format cozy-chess uses
/// Implementation from cozy-chess
fn from_uci_castling(board: &Board, mut mv: Move) -> Move {
    if mv.from.file() == File::E && board.piece_on(mv.from) == Some(Piece::King) {
        if mv.to.file() == File::G {
            mv.to = Square::new(File::H, mv.to.rank());
        } else if mv.to.file() == File::C {
            mv.to = Square::new(File::A, mv.to.rank());
        }
    }
    mv
}
