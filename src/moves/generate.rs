use super::info::{Move, MoveList};
use crate::position::Position;
use crate::utils::{MoveType, Square};
use num_traits::FromPrimitive;

pub fn count_legal_moves(pos: &mut Position, ply: u32) -> u64 {
    if ply == 0 {
        return 1;
    }

    pos.compute_legal_moves();
    let mut moves = MoveList::default();
    generate_moves(pos, &mut moves);
    if ply == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;
    for mv in moves.iter() {
        let undo = pos.move_piece(mv);
        if undo.move_type == MoveType::Invalid {
            panic!("generated invalid move: {:?} -> {:?}", mv.from, mv.to);
        }
        nodes += count_legal_moves(pos, ply - 1);
        pos.undo_move(undo);
    }

    nodes
}

pub fn generate_moves(pos: &Position, out: &mut MoveList) {
    out.clear();
    let mut pieces = pos.bitboards.get_checkers(pos.side_to_move());
    while pieces != 0 {
        let sq = Square::from_u8(pieces.trailing_zeros() as u8).unwrap_or_default();
        pieces &= pieces - 1;

        let mut targets = pos.legal_destinations_from(sq);
        while targets != 0 {
            let to = Square::from_u8(targets.trailing_zeros() as u8).unwrap_or_default();
            targets &= targets - 1;
            out.push(Move { from: sq, to });
        }
    }
}
