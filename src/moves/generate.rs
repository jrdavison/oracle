use crate::bitboards::{self, Bitboard};
use crate::magic_bitboards::{
    BISHOP_BLOCKERS_LOOKUP, DIAGONAL_MASKS, KING_MASKS, KNIGHT_MASKS, ORTHOGONAL_MASKS, PAWN_ATTACK_MASKS,
    ROOK_BLOCKERS_LOOKUP,
};
use crate::position::Position;
use crate::utils::{Color, Direction, Piece, PieceType, Rank, Square};
use std::ops::BitOr;

#[derive(Default)]
pub struct ComputedMoves {
    pub valid_moves: Bitboard,
    pub attacks: Bitboard,
}

impl BitOr for ComputedMoves {
    type Output = ComputedMoves;

    fn bitor(self, rhs: ComputedMoves) -> ComputedMoves {
        ComputedMoves {
            valid_moves: self.valid_moves | rhs.valid_moves,
            attacks: self.attacks | rhs.attacks,
        }
    }
}

pub fn compute_valid_moves(pos: &mut Position, color: Color) {
    let mut attacks = 0;
    for sq in Square::iter() {
        let piece = pos.board[sq];
        let piece_type = Piece::type_of(piece);
        let mut computed_moves = ComputedMoves::default();

        /*
        only compute moves for pieces of the correct color.
        Valid moves and attacks will be reset back to 0 for the enemy color
        */
        // if color == Piece::color_of(piece) {
        // let p_start = Instant::now();
        match piece_type {
            PieceType::Pawn => computed_moves = compute_pawn_moves(pos, sq),
            PieceType::Knight => computed_moves = compute_knight_moves(sq),
            PieceType::Rook => computed_moves = compute_rook_moves(pos, sq),
            PieceType::Bishop => computed_moves = compute_bishop_moves(pos, sq),
            PieceType::Queen => computed_moves = compute_rook_moves(pos, sq) | compute_bishop_moves(pos, sq),
            PieceType::King => computed_moves = compute_king_moves(sq),
            _ => {}
        }
        // }

        /*
        TODO: check if move puts king in check (diagonal and horizontal pins)

        we will need to copy the position and then use that to simulate the move. Then we check if the king is in
        check
        */

        computed_moves.valid_moves &= !pos.bitboards.get_checkers(color);
        pos.bitboards.set_valid_moves(sq, computed_moves.valid_moves);
        attacks |= computed_moves.attacks;
        // println!(
        //     "Computed moves for {:?} in {:.4} ms",
        //     piece,
        //     p_start.elapsed().as_secs_f64() * 1000.0
        // );
    }

    pos.bitboards.set_attacks(color, attacks);
}

fn compute_pawn_moves(pos: &Position, sq: Square) -> ComputedMoves {
    let mut valid_moves = 0;

    let piece = pos.board[sq];
    let color = Piece::color_of(piece);
    let forward = Direction::forward_direction(color);

    let mut target_sq = sq;
    for i in 0..2 {
        // only allow double move from starting rank
        if i == 1 && (Rank::relative_rank(color, Square::rank_of(sq)) != Rank::Rank2) {
            break;
        }

        target_sq = target_sq + forward;
        if (target_sq != Square::Count) && !pos.bitboards.is_checkers_sq_set(Color::Both, target_sq) {
            bitboards::set_bit(&mut valid_moves, target_sq);
        } else {
            break;
        }
    }

    let mut enemy_checkers = pos.bitboards.get_checkers(!color);
    if pos.en_passant_square != Square::Count {
        bitboards::set_bit(&mut enemy_checkers, pos.en_passant_square);
    }
    let attacks = PAWN_ATTACK_MASKS[color as usize][sq as usize] & enemy_checkers;
    valid_moves |= attacks;

    // TODO: promotion

    ComputedMoves { valid_moves, attacks }
}

fn compute_knight_moves(sq: Square) -> ComputedMoves {
    let valid_moves = KNIGHT_MASKS[sq as usize];
    ComputedMoves {
        valid_moves,
        attacks: valid_moves,
    }
}

fn compute_rook_moves(pos: &Position, sq: Square) -> ComputedMoves {
    let move_mask = ORTHOGONAL_MASKS[sq as usize];
    let blocker_key = pos.bitboards.get_checkers(Color::Both) & move_mask;
    let valid_moves = *ROOK_BLOCKERS_LOOKUP[sq as usize]
        .get(&blocker_key)
        .unwrap_or(&Bitboard::default());

    ComputedMoves {
        valid_moves,
        attacks: valid_moves,
    }
}

fn compute_bishop_moves(pos: &Position, sq: Square) -> ComputedMoves {
    let diagonal_mask = DIAGONAL_MASKS[sq as usize];
    let blocker_key = pos.bitboards.get_checkers(Color::Both) & diagonal_mask;
    let attacks = *BISHOP_BLOCKERS_LOOKUP[sq as usize]
        .get(&blocker_key)
        .unwrap_or(&Bitboard::default());

    ComputedMoves {
        valid_moves: attacks,
        attacks,
    }
}

fn compute_king_moves(sq: Square) -> ComputedMoves {
    // TODO: castling
    let valid_moves = KING_MASKS[sq as usize];
    ComputedMoves {
        valid_moves,
        attacks: valid_moves,
    }
}
