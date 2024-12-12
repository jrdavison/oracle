use crate::bitboards::{self, Bitboard};
use crate::magic_bitboards::LOOKUP_TABLES;
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
        let piece = pos.board[sq as usize];
        let piece_type = Piece::type_of(piece);
        let mut computed_moves = ComputedMoves::default();

        match piece_type {
            PieceType::Pawn => computed_moves = compute_pawn_moves(pos, sq),
            PieceType::Knight => computed_moves = compute_knight_moves(sq),
            PieceType::Rook => computed_moves = compute_rook_moves(pos, sq),
            PieceType::Bishop => computed_moves = compute_bishop_moves(pos, sq),
            PieceType::Queen => computed_moves = compute_rook_moves(pos, sq) | compute_bishop_moves(pos, sq),
            PieceType::King => computed_moves = compute_king_moves(sq),
            _ => {}
        }

        /*
        TODO: check if move puts king in check (diagonal and horizontal pins)

        we will need to copy the position and then use that to simulate the move. Then we check if the king is in
        check. We can probably do this without having to recalculate the moves for the entire board though...
        */

        computed_moves.valid_moves &= !pos.bitboards.get_checkers(color);
        pos.bitboards.set_valid_moves(sq, computed_moves.valid_moves);
        attacks |= computed_moves.attacks;
    }

    pos.bitboards.set_attacks(color, attacks);
}

fn compute_pawn_moves(pos: &Position, sq: Square) -> ComputedMoves {
    let mut valid_moves = 0;

    let piece = pos.board[sq as usize];
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
            valid_moves = bitboards::set_bit(valid_moves, target_sq);
        } else {
            break;
        }
    }

    let mut enemy_checkers = pos.bitboards.get_checkers(!color);
    if pos.en_passant_square != Square::Count {
        enemy_checkers = bitboards::set_bit(enemy_checkers, pos.en_passant_square);
    }
    let attacks = LOOKUP_TABLES.pawn_attack_masks[color as usize][sq as usize] & enemy_checkers;
    valid_moves |= attacks;

    ComputedMoves { valid_moves, attacks }
}

fn compute_knight_moves(sq: Square) -> ComputedMoves {
    let attacks = LOOKUP_TABLES.knight_masks[sq as usize];
    ComputedMoves {
        attacks,
        valid_moves: attacks,
    }
}

fn compute_rook_moves(pos: &Position, sq: Square) -> ComputedMoves {
    let move_mask = LOOKUP_TABLES.orthogonal_masks[sq as usize];
    let blocker_key = pos.bitboards.get_checkers(Color::Both) & move_mask;
    let attacks = LOOKUP_TABLES.rook_blockers_lookup[sq as usize].get(&blocker_key);

    ComputedMoves {
        attacks,
        valid_moves: attacks,
    }
}

fn compute_bishop_moves(pos: &Position, sq: Square) -> ComputedMoves {
    let diagonal_mask = LOOKUP_TABLES.diagonal_masks[sq as usize];
    let blocker_key = pos.bitboards.get_checkers(Color::Both) & diagonal_mask;
    let attacks = LOOKUP_TABLES.bishop_blockers_lookup[sq as usize].get(&blocker_key);

    ComputedMoves {
        valid_moves: attacks,
        attacks,
    }
}

fn compute_king_moves(sq: Square) -> ComputedMoves {
    // TODO: castling
    let attacks = LOOKUP_TABLES.king_masks[sq as usize];
    ComputedMoves {
        attacks,
        valid_moves: attacks,
    }
}
