use std::time::Instant;

use crate::{move_ordering::mvv_lva, position::Position};
use cozy_chess::Move;

const CHECKMATE: i32 = 100000;
const DRAW: i32 = 0;

#[derive(Clone, Copy, Debug)]
struct SearchInfo {
    pub best_move: Option<Move>,
    pub depth: u8,
    pub nodes: u64,
    pub eval: i32,
}

/// The main search algorithm is negamax with alpha-beta pruning
/// Negamax is a variant of minimax
fn negamax(
    mut alpha: i32,
    beta: i32,
    depth: u8,
    pos: &mut Position,
    ply: u32,
    search_info: &mut SearchInfo,
) -> i32 {
    // Visiting one more node
    search_info.nodes += 1;

    // Reached a leaf node, return the evaluation
    if depth == 0 {
        return pos.eval();
    }

    // An average position has ~32 moves.
    // TODO: It might be better to use a bigger capacity here anyway
    let mut moves: Vec<Move> = Vec::with_capacity(32);
    let mut captures = Vec::with_capacity(32);
    let enemy_pieces = pos.board.colors(!pos.board.side_to_move());
    pos.board.generate_moves(|mv| {
        let mut capture_moves = mv;

        // Filter out non-captures
        capture_moves.to &= enemy_pieces;
        captures.extend(capture_moves);

        // These are all the legal moves
        moves.extend(mv);
        false
    });

    // TODO: cozy_chess has a function for this
    if moves.is_empty() {
        return if !pos.board.checkers().is_empty() {
            // No moves and in check, checkmate
            // We want to return the score relative to the current ply so that
            // We find checkmate in the least moves possible
            -CHECKMATE + ply as i32
        } else {
            // No moves and not in check, stalemate
            DRAW
        };
    }

    // Sort the moves by MVV-LVA
    // This is for more efficient alpha-beta pruning
    // TODO: Score the moves instead of sorting them.
    // We waste time sorting moves that we will never search
    sort_moves(&mut moves, pos);

    let mut best_score = -CHECKMATE;

    for mv in moves {
        let mut new_pos = pos.clone();
        new_pos.make_move(mv);

        // Check for a 3-fold repetition
        // We do this by checking if our new position has been seen before
        if pos.is_repetition(new_pos.board.hash()) {
            return DRAW;
        }
        let score = -negamax(-beta, -alpha, depth - 1, &mut new_pos, ply + 1, search_info);
        best_score = best_score.max(score);

        if score > alpha {
            alpha = score;
        }

        // Fail-hard beta cutoff
        if alpha >= beta {
            break;
        }
    }

    best_score
}

/// Sorts the moves by MVV-LVA
fn sort_moves(moves: &mut [Move], pos: &Position) {
    let mut scores = Vec::with_capacity(moves.len());
    for mv in moves.iter() {
        scores.push(mvv_lva(*mv, pos));
    }

    // TODO: More efficient sorting
    for i in 0..moves.len() {
        let mut best_score = scores[i];
        let mut best_index = i;
        for j in i + 1..moves.len() {
            if scores[j] > best_score {
                best_score = scores[j];
                best_index = j;
            }
        }

        if best_index != i {
            moves.swap(i, best_index);
            scores.swap(i, best_index);
        }
    }
}

/// The main entry point for the search
/// UCI output
pub fn think(pos: &mut Position, depth: u8, time_limit_millis: u128) {
    let mut candidate_move: Option<Move> = None;
    let start_time = Instant::now();

    // Iterative deepening
    for d in 1..=depth {
        // We are out of time
        if Instant::now().duration_since(start_time).as_millis() >= time_limit_millis {
            println!("bestmove {}", candidate_move.unwrap());
            return;
        }

        let search_info = best_move(pos, d);
        candidate_move = Some(search_info.best_move.unwrap());
        println!(
            "info depth {} score cp {} nodes {} pv {}",
            search_info.depth,
            search_info.eval,
            search_info.nodes,
            candidate_move.unwrap()
        );
    }

    // We reached max depth
    println!("bestmove {}", candidate_move.unwrap());
}

/// Get the best move for the current position using negamax
fn best_move(pos: &mut Position, depth: u8) -> SearchInfo {
    let mut best_move = None;
    let mut best_eval = -CHECKMATE;

    let mut moves = Vec::with_capacity(32);
    pos.board.generate_moves(|mv| {
        moves.extend(mv);
        false
    });

    let mut search_info = SearchInfo {
        best_move,
        depth,
        nodes: 0,
        eval: best_eval,
    };

    for mv in moves {
        let mut new_pos = pos.clone();
        new_pos.make_move(mv);
        let score = -negamax(
            -CHECKMATE,
            CHECKMATE,
            depth - 1,
            &mut new_pos,
            1,
            &mut search_info,
        );
        if score > best_eval {
            best_eval = score;
            best_move = Some(mv);
        }
    }

    search_info.best_move = best_move;
    search_info.eval = best_eval;
    search_info
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    // The engine should be able to find checkmate regardless of the evaluation function
    fn test_checkmate() {
        // Mate in 2
        let mut pos =
            Position::from_fen("r4rk1/p1pb3p/2p5/3p2pN/3Qp1Pq/4P2P/PPP2PK1/R5R1 b - - 1 21");
        let search_info = best_move(&mut pos, 4);
        assert_eq!(search_info.best_move.unwrap(), "f8f2".parse().unwrap());
        // 3 ply to mate
        assert_eq!(search_info.eval, CHECKMATE - 3);
    }
}
