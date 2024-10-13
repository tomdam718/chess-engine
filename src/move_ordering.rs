use cozy_chess::{Move, Piece};

use crate::position::Position;

pub fn mvv_lva(mv: Move, pos: &Position) -> i32 {
    8 * pos.board.piece_on(mv.to).unwrap_or(Piece::Pawn) as i32
        - pos.board.piece_on(mv.from).unwrap_or(Piece::Pawn) as i32
}
