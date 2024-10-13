use cozy_chess::{Board, Move, Piece, Square};

use crate::nnue::{Accumulator, Network};

pub const SEE_VALS: [i32; 6] = [100, 450, 450, 650, 1250, 0];

#[derive(Clone, Debug)]
pub struct Position {
    // cozy_chess board
    pub board: Board,
    // NNUE accumulators
    pub acc: [Accumulator; 2],
    // Stores hashes of previous positions
    pub repeition_history: Vec<u64>,
}

impl Position {
    pub fn from_fen(fen: &str) -> Self {
        let board = Board::from_fen(fen, false).unwrap();

        let mut pos = Self {
            board,
            acc: [Accumulator::default(), Accumulator::default()],
            repeition_history: Vec::new(),
        };

        pos.update_acc();
        pos
    }

    /// Inserts a new hash into the repetition history
    fn update_repetitions(&mut self) {
        let hash = self.board.hash();
        self.repeition_history.push(hash);
    }

    /// If the current position has already been reached twice before, it is a repetition
    pub fn is_repetition(&self, hash: u64) -> bool {
        // Check if the current position has been seen twice before
        self.repeition_history
            .iter()
            .filter(|&n| *n == hash)
            .count()
            >= 2
    }

    /// Goes through every square to update the accumulators
    fn update_acc(&mut self) {
        self.acc = [Accumulator::default(), Accumulator::default()];
        // Update accumulators
        for sq in 0..64 {
            if let Some(pc) = self.board.piece_on(Square::index(sq)) {
                let color = self.board.color_on(Square::index(sq)).unwrap();
                self.toggle::<true>(color as usize, pc as usize, sq);
            }
        }
    }

    /// Makes a move on the board
    /// Also updates the accumulators and the repetition history
    pub fn make_move(&mut self, mv: Move) {
        // If it is a capture, update the accumulator for the captured piece
        if self.board.piece_on(mv.to).is_some() {
            self.toggle::<false>(
                self.board.side_to_move() as usize ^ 1,
                self.board.piece_on(mv.to).unwrap() as usize,
                mv.to as usize,
            );
        }

        // If it is a promotion, update the accumulator for the promoted piece
        if mv.promotion.is_some() {
            self.toggle::<false>(
                self.board.side_to_move() as usize,
                Piece::Pawn as usize,
                mv.to as usize,
            );
            self.toggle::<true>(
                self.board.side_to_move() as usize,
                mv.promotion.unwrap() as usize,
                mv.to as usize,
            );

            self.board.play_unchecked(mv);
            self.update_repetitions();

            return;
        }

        // If it is a castling move, update the accumulator for the rook and king
        if self.board.color_on(mv.to) == self.board.color_on(mv.from) {
            self.board.play_unchecked(mv);

            // Just reset the accumulators
            // TODO: This is quick but should be changed to incrementally update accumulators
            self.update_acc();
            self.update_repetitions();

            return;
        }

        // Update the accumulators
        self.toggle::<false>(
            self.board.side_to_move() as usize,
            self.board.piece_on(mv.from).unwrap() as usize,
            mv.from as usize,
        );

        self.toggle::<true>(
            self.board.side_to_move() as usize,
            self.board.piece_on(mv.from).unwrap() as usize,
            mv.to as usize,
        );

        self.board.play_unchecked(mv);

        // Update repetitions history
        self.update_repetitions();
    }

    fn toggle<const ADD: bool>(&mut self, side: usize, pc: usize, sq: usize) {
        // Find the index to update in the accumulator
        let start = 384 * side + 64 * pc + sq;
        self.acc[0].update::<ADD>(start);

        let start = 384 * (side ^ 1) + 64 * pc + (sq ^ 56);
        self.acc[1].update::<ADD>(start);
    }

    /// The NNUE evaluation of the current position
    pub fn eval(&self) -> i32 {
        let boys = &self.acc[self.board.side_to_move() as usize];
        let opps = &self.acc[self.board.side_to_move() as usize ^ 1];
        let eval = Network::out(boys, opps);
        self.scale(eval)
    }

    /// Scales the evaluation based on the material on the board
    fn scale(&self, eval: i32) -> i32 {
        let mut mat = self.board.pieces(Piece::Knight).len() as i32
            * SEE_VALS[Piece::Knight as usize]
            + self.board.pieces(Piece::Bishop).len() as i32 * SEE_VALS[Piece::Bishop as usize]
            + self.board.pieces(Piece::Rook).len() as i32 * SEE_VALS[Piece::Rook as usize]
            + self.board.pieces(Piece::Queen).len() as i32 * SEE_VALS[Piece::Queen as usize];

        mat = 700 + mat / 32;

        eval * mat / 1024
    }
}
