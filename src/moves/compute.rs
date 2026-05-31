use crate::bitboards::{self, Bitboard, LOOKUP_TABLES};
use crate::position::Position;
use crate::utils::{CastlingRights, Color, Direction, Piece, PieceType, Rank, Square};
use num_traits::FromPrimitive;
use std::ops::BitOr;

pub const KINGSIDE_CASTLE_SQUARES: [Square; Color::Both as usize] = [Square::G1, Square::G8];
pub const QUEENSIDE_CASTLE_SQUARES: [Square; Color::Both as usize] = [Square::C1, Square::C8];

const KINGSIDE_CASTLE_MASKS: [Bitboard; Color::Both as usize] = [
    0b01110000,       // white back rank
    0b01110000 << 56, // black back rank
];
const QUEENSIDE_CASTLE_MASKS: [Bitboard; Color::Both as usize] = [
    0b00011110,       // white back rank
    0b00011110 << 56, // black back rank
];

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

pub fn compute_valid_moves(pos: &mut Position) {
    let friendly_color = pos.side_to_move;
    let (pinned_masks, check_mask) = compute_pin_and_check_masks(pos, friendly_color);

    let compute_moves_for_color = |pos: &mut Position, color: Color| {
        let mut attacks = Bitboard::default();
        for sq in &pos.occupied_squares[color as usize] {
            let piece = pos.board[*sq as usize];
            let piece_type = Piece::type_of(piece);
            let piece_color = Piece::color_of(piece);

            // psuedo-legal moves
            let mut computed_moves = match piece_type {
                PieceType::Pawn => compute_pawn_moves(pos, *sq, piece_color),
                PieceType::Knight => compute_knight_moves(*sq),
                PieceType::Rook => compute_rook_moves(pos, *sq),
                PieceType::Bishop => compute_bishop_moves(pos, *sq),
                PieceType::Queen => compute_rook_moves(pos, *sq) | compute_bishop_moves(pos, *sq),
                PieceType::King => compute_king_moves(pos, color),
                _ => ComputedMoves::default(),
            };

            // can't capture own pieces
            computed_moves.valid_moves &= !pos.bitboards.get_checkers(piece_color);

            if color == friendly_color && piece_type != PieceType::King {
                computed_moves.valid_moves &= check_mask;
                computed_moves.valid_moves &= pinned_masks[*sq as usize];
            }

            pos.bitboards.set_valid_moves(*sq, computed_moves.valid_moves);
            attacks |= computed_moves.attacks;
        }
        pos.bitboards.set_attacks(color, attacks);
    };

    compute_moves_for_color(pos, !pos.side_to_move); // enemy moves first
    compute_moves_for_color(pos, pos.side_to_move); // then friendly moves to handle checks/pins
}

fn compute_pawn_moves(pos: &Position, sq: Square, color: Color) -> ComputedMoves {
    let mut valid_moves = 0;
    let forward = Direction::forward_direction(color);
    let start_rank = Rank::relative_rank(color, Square::rank_of(sq));

    // Check single or double forward moves
    let mut target_sq = sq + forward;
    if target_sq != Square::Count && !pos.bitboards.is_checkers_sq_set(Color::Both, target_sq) {
        valid_moves = bitboards::set_bit(valid_moves, target_sq);

        if start_rank == Rank::Rank2 {
            target_sq = target_sq + forward;
            if target_sq != Square::Count && !pos.bitboards.is_checkers_sq_set(Color::Both, target_sq) {
                valid_moves = bitboards::set_bit(valid_moves, target_sq);
            }
        }
    }

    let enemy_checkers = pos.bitboards.get_checkers(!color);
    let attack_mask = LOOKUP_TABLES.get_pawn_attack_mask(color, sq);
    let en_passant_bit = if pos.en_passant_sq != Square::Count {
        bitboards::set_bit(0, pos.en_passant_sq)
    } else {
        0
    };

    valid_moves |= attack_mask & (enemy_checkers | en_passant_bit);

    ComputedMoves {
        valid_moves,
        attacks: attack_mask,
    }
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
        // use this wiki for test cases: https://en.wikipedia.org/wiki/Castling
        let friendly_pieces_no_king = bitboards::clear_bit(friendly_pieces, sq);

        let kingside_castle_mask = KINGSIDE_CASTLE_MASKS[color as usize];
        let kingside_castle_sq = KINGSIDE_CASTLE_SQUARES[color as usize];
        let kingside_rights_mask = if color == Color::White {
            CastlingRights::WhiteOO
        } else {
            CastlingRights::BlackOO
        };
        let kingside_rights = pos.castling_rights & kingside_rights_mask;
        let kingside_blockers = kingside_castle_mask & (friendly_pieces_no_king | enemy_attacks);
        if (kingside_rights != CastlingRights::NoCastling) && (kingside_blockers == 0) {
            valid_moves = bitboards::set_bit(valid_moves, kingside_castle_sq);
        }

        let queenside_castle_mask = QUEENSIDE_CASTLE_MASKS[color as usize];
        let queenside_castle_sq = QUEENSIDE_CASTLE_SQUARES[color as usize];
        let queenside_rights_mask = if color == Color::White {
            CastlingRights::WhiteOOO
        } else {
            CastlingRights::BlackOOO
        };
        let queenside_rights = pos.castling_rights & queenside_rights_mask;
        let queenside_blockers = queenside_castle_mask & (friendly_pieces_no_king | enemy_attacks);
        if (queenside_rights != CastlingRights::NoCastling) && (queenside_blockers == 0) {
            valid_moves = bitboards::set_bit(valid_moves, queenside_castle_sq);
        }
    }

    ComputedMoves { attacks, valid_moves }
}

fn compute_pin_and_check_masks(pos: &Position, color: Color) -> ([Bitboard; Square::Count as usize], Bitboard) {
    let mut pin_masks = [u64::MAX; Square::Count as usize];
    let mut check_mask = u64::MAX;
    let mut num_checks = 0u32;

    let occupancy = pos.bitboards.get_checkers(Color::Both);
    let king_sq = pos.king_squares[color as usize];
    let friendly_pieces = pos.bitboards.get_checkers(color);
    let enemy_pieces = pos.bitboards.get_checkers(!color);

    let enemy_rook_queens = enemy_pieces
        & (piece_type_mask(pos, PieceType::Rook) | piece_type_mask(pos, PieceType::Queen));
    let enemy_bishop_queens = enemy_pieces
        & (piece_type_mask(pos, PieceType::Bishop) | piece_type_mask(pos, PieceType::Queen));
    let enemy_knights = enemy_pieces & piece_type_mask(pos, PieceType::Knight);
    let enemy_pawns = enemy_pieces & piece_type_mask(pos, PieceType::Pawn);

    let rook_attackers = slider_attackers(king_sq, occupancy, enemy_rook_queens, true);
    let bishop_attackers = slider_attackers(king_sq, occupancy, enemy_bishop_queens, false);
    let mut sliders = rook_attackers | bishop_attackers;

    while sliders != 0 {
        let pinner_idx = sliders.trailing_zeros() as usize;
        sliders &= sliders - 1;

        let pinner_sq = Square::from_u8(pinner_idx as u8).unwrap_or_default();
        let between = squares_between(pos, king_sq, pinner_sq);
        let blockers = between & occupancy;

        if blockers == 0 {
            num_checks += 1;
            check_mask = bitboards::set_bit(0, pinner_sq);
        } else if blockers.count_ones() == 1 && (blockers & friendly_pieces) != 0 {
            let pinned_idx = blockers.trailing_zeros() as usize;
            pin_masks[pinned_idx] = between | bitboards::set_bit(0, pinner_sq);
        }
    }

    let knight_checks = LOOKUP_TABLES.get_knight_mask(king_sq) & enemy_knights;
    if knight_checks != 0 {
        num_checks += knight_checks.count_ones();
        check_mask = knight_checks;
    }

    let pawn_checks = LOOKUP_TABLES.get_pawn_attack_mask(color, king_sq) & enemy_pawns;
    if pawn_checks != 0 {
        num_checks += pawn_checks.count_ones();
        check_mask = pawn_checks;
    }

    if num_checks >= 2 {
        // double check, only the king can move in this case
        check_mask = 0;
    } else if num_checks == 0 {
        check_mask = u64::MAX;
    }

    (pin_masks, check_mask)
}

fn piece_type_mask(pos: &Position, piece_type: PieceType) -> Bitboard {
    let mut mask = 0;
    for sq in Square::iter() {
        if Piece::type_of(pos.board[sq as usize]) == piece_type {
            mask = bitboards::set_bit(mask, sq);
        }
    }
    mask
}

fn slider_attackers(king_sq: Square, occ: Bitboard, candidates: Bitboard, rook_like: bool) -> Bitboard {
    let move_mask = if rook_like {
        LOOKUP_TABLES.get_orthogonal_mask(king_sq)
    } else {
        LOOKUP_TABLES.get_diagonal_mask(king_sq)
    };
    let blockers = occ & move_mask;
    let attacks = if rook_like {
        LOOKUP_TABLES.get_rook_mask(king_sq, blockers)
    } else {
        LOOKUP_TABLES.get_bishop_mask(king_sq, blockers)
    };
    attacks & candidates
}

fn squares_between(_pos: &Position, a: Square, b: Square) -> Bitboard {
    let ray_ab = LOOKUP_TABLES.get_rook_mask(a, 0) & LOOKUP_TABLES.get_rook_mask(b, 0);
    if ray_ab != 0 {
        return ray_ab;
    }

    LOOKUP_TABLES.get_bishop_mask(a, 0) & LOOKUP_TABLES.get_bishop_mask(b, 0)
}
