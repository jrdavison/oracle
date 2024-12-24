use crate::bitboards::{self, Bitboard, LOOKUP_TABLES};
use crate::position::Position;
use crate::utils::{CastlingRights, Color, Direction, Piece, PieceType, Rank, Square};
use std::ops::BitOr;

const KINGSIDE_CASTLE_MASKS: [Bitboard; Color::Both as usize] = [
    0b01110000,       // white back rank
    0b01110000 << 56, // black back rank
];
const QUEENSIDE_CASTLE_MASKS: [Bitboard; Color::Both as usize] = [
    0b00011110,       // white back rank
    0b00011110 << 56, // black back rank
];
const KINGSIDE_CASTLE_SQUARE: [Square; Color::Both as usize] = [Square::G1, Square::G8];
const QUEENSIDE_CASTLE_SQUARE: [Square; Color::Both as usize] = [Square::C1, Square::C8];

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

pub fn compute_valid_moves(pos: &mut Position, turn_color: Color) {
    let mut attacks = [Bitboard::default(); Color::Both as usize];
    for sq in Square::iter() {
        let piece = pos.board[sq as usize];
        let piece_type = Piece::type_of(piece);
        if piece_type != PieceType::Empty {
            let piece_color = Piece::color_of(piece);

            let mut computed_moves = match piece_type {
                PieceType::Pawn => compute_pawn_moves(pos, sq, piece_color),
                PieceType::Knight => compute_knight_moves(sq),
                PieceType::Rook => compute_rook_moves(pos, sq),
                PieceType::Bishop => compute_bishop_moves(pos, sq),
                PieceType::Queen => compute_rook_moves(pos, sq) | compute_bishop_moves(pos, sq),
                _ => ComputedMoves::default(),
            };

            /*
            TODO: check if move puts king in check (diagonal and horizontal pins)

            we will need to copy the position and then use that to simulate the move. Then we check if the king is in
            check. We can probably do this without having to recalculate the moves for the entire board though...
            */

            computed_moves.valid_moves &= !pos.bitboards.get_checkers(piece_color);
            pos.bitboards.set_valid_moves(sq, computed_moves.valid_moves);
            attacks[piece_color as usize] |= computed_moves.attacks;
        }
    }
    let enemy_color = !turn_color;
    let enemy_king_moves = compute_king_moves(pos, enemy_color);
    pos.bitboards
        .set_valid_moves(pos.king_squares[enemy_color as usize], enemy_king_moves.valid_moves);
    attacks[enemy_color as usize] |= enemy_king_moves.attacks;
    pos.bitboards.set_attacks(enemy_color, attacks[enemy_color as usize]);

    let friendly_king_moves = compute_king_moves(pos, turn_color);
    pos.bitboards
        .set_valid_moves(pos.king_squares[turn_color as usize], friendly_king_moves.valid_moves);
    attacks[turn_color as usize] |= friendly_king_moves.attacks;
    pos.bitboards.set_attacks(turn_color, attacks[turn_color as usize]);
}

fn compute_pawn_moves(pos: &Position, sq: Square, color: Color) -> ComputedMoves {
    let mut valid_moves = 0;
    let forward = Direction::forward_direction(color);

    let mut target_sq = sq;
    let start_rank = Rank::relative_rank(color, Square::rank_of(sq));
    if start_rank == Rank::Rank2 {
        for _ in 0..2 {
            target_sq = target_sq + forward;
            if target_sq == Square::Count || pos.bitboards.is_checkers_sq_set(Color::Both, target_sq) {
                break;
            }
            valid_moves = bitboards::set_bit(valid_moves, target_sq);
        }
    } else {
        target_sq = target_sq + forward;
        if target_sq != Square::Count && !pos.bitboards.is_checkers_sq_set(Color::Both, target_sq) {
            valid_moves = bitboards::set_bit(valid_moves, target_sq);
        }
    }

    let enemy_checkers = pos.bitboards.get_checkers(!color);
    let attacks = LOOKUP_TABLES.get_pawn_attack_mask(color, sq);
    valid_moves |= attacks & bitboards::set_bit(enemy_checkers, pos.en_passant_sq);

    ComputedMoves { valid_moves, attacks }
}

fn compute_knight_moves(sq: Square) -> ComputedMoves {
    let attacks = LOOKUP_TABLES.get_knight_mask(sq);
    ComputedMoves {
        attacks,
        valid_moves: attacks,
    }
}

fn compute_rook_moves(pos: &Position, sq: Square) -> ComputedMoves {
    let move_mask = LOOKUP_TABLES.get_orthogonal_mask(sq);
    let blocker_key = pos.bitboards.get_checkers(Color::Both) & move_mask;
    let attacks = LOOKUP_TABLES.get_rook_mask(sq, blocker_key);

    ComputedMoves {
        attacks,
        valid_moves: attacks,
    }
}

fn compute_bishop_moves(pos: &Position, sq: Square) -> ComputedMoves {
    let diagonal_mask = LOOKUP_TABLES.get_diagonal_mask(sq);
    let blocker_key = pos.bitboards.get_checkers(Color::Both) & diagonal_mask;
    let attacks = LOOKUP_TABLES.get_bishop_mask(sq, blocker_key);

    ComputedMoves {
        valid_moves: attacks,
        attacks,
    }
}

fn compute_king_moves(pos: &Position, color: Color) -> ComputedMoves {
    let sq = pos.king_squares[color as usize];
    let enemy_attacks = pos.bitboards.get_attacks(!color);
    let friendly_pieces = pos.bitboards.get_checkers(color);

    let attacks = LOOKUP_TABLES.get_king_mask(sq);
    let mut valid_moves = attacks & !friendly_pieces & !enemy_attacks;

    if pos.castling_rights != CastlingRights::NoCastling {
        let kingside_castle_mask = KINGSIDE_CASTLE_MASKS[color as usize];
        let kingside_castle_sq = KINGSIDE_CASTLE_SQUARE[color as usize];

        let kingside = if color == Color::White {
            CastlingRights::WhiteOO
        } else {
            CastlingRights::BlackOO
        };
        if pos.castling_rights & kingside != CastlingRights::NoCastling
            && (kingside_castle_mask & !friendly_pieces & !enemy_attacks) != 0
        {
            valid_moves = bitboards::set_bit(valid_moves, kingside_castle_sq);
        }

        let queenside_castle_mask = QUEENSIDE_CASTLE_MASKS[color as usize];
        let queenside_castle_sq = QUEENSIDE_CASTLE_SQUARE[color as usize];
        let queenside = if color == Color::White {
            CastlingRights::WhiteOOO
        } else {
            CastlingRights::BlackOOO
        };
        if pos.castling_rights & queenside != CastlingRights::NoCastling
            && (queenside_castle_mask & !friendly_pieces & !enemy_attacks) != 0
        {
            valid_moves = bitboards::set_bit(valid_moves, queenside_castle_sq);
        }
    }

    // TODO: castling

    ComputedMoves { attacks, valid_moves }
}
