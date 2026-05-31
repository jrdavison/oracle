use super::compute::{KINGSIDE_CASTLE_SQUARES, QUEENSIDE_CASTLE_SQUARES};
use crate::position::Position;
use crate::utils::{CastlingRights, Direction, MoveType, Piece, PieceType, Rank, Square};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
}

// Enough for reachable orthodox chess positions; arbitrary FENs can exceed this.
const MAX_MOVES: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct MoveList {
    pub moves: [Move; MAX_MOVES],
    pub len: usize,
}

impl Default for MoveList {
    fn default() -> MoveList {
        MoveList {
            moves: [Move::default(); MAX_MOVES],
            len: 0,
        }
    }
}

impl MoveList {
    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn push(&mut self, mv: Move) {
        debug_assert!(self.len < self.moves.len());
        self.moves[self.len] = mv;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = Move> + '_ {
        self.moves[..self.len].iter().copied()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MoveInfo {
    pub move_type: MoveType,
    pub from: Square,
    pub to: Square,
    pub moved_piece: Piece,
    pub captured_piece: Piece,
    pub capture_piece_sq: Square,
    pub en_passant_sq: Square,
    pub castling_rights: CastlingRights,
    pub fullmove_count: i32,
    pub halfmove_clock: i32,
}

impl MoveInfo {
    pub fn new(position: &Position, from: Square, to: Square) -> MoveInfo {
        let move_type;
        let moved_piece = position.board[from as usize];
        let moved_piece_color = Piece::color_of(moved_piece);
        let moved_piece_type = Piece::type_of(moved_piece);

        let mut captured_piece = position.board[to as usize];
        let mut capture_piece_sq = Square::Count; // save square for en passant captures

        match moved_piece_type {
            PieceType::Pawn => {
                let color = Piece::color_of(moved_piece);
                let from_rank = Square::rank_of(from);
                let to_rank = Square::rank_of(to);
                let from_file = Square::file_of(from);
                let to_file = Square::file_of(to);

                let relative_from_rank = Rank::relative_rank(color, from_rank);
                let relative_to_rank = Rank::relative_rank(color, to_rank);
                if relative_from_rank == Rank::Rank7 {
                    move_type = MoveType::Promotion;
                    capture_piece_sq = to;
                } else if relative_from_rank == Rank::Rank2 && relative_to_rank == Rank::Rank4 {
                    move_type = MoveType::TwoSquarePush;
                } else if from_file != to_file {
                    if captured_piece == Piece::Empty {
                        move_type = MoveType::EnPassant;
                        capture_piece_sq = to + Direction::forward_direction(!color);
                        captured_piece = position.board[capture_piece_sq as usize];
                    } else {
                        move_type = MoveType::Capture;
                        capture_piece_sq = to;
                    }
                } else {
                    move_type = MoveType::Quiet;
                }
            }
            PieceType::King => {
                if captured_piece != Piece::Empty {
                    move_type = MoveType::Capture;
                    capture_piece_sq = to;
                } else if to == KINGSIDE_CASTLE_SQUARES[moved_piece_color as usize]
                    || to == QUEENSIDE_CASTLE_SQUARES[moved_piece_color as usize]
                {
                    move_type = MoveType::Castle;
                } else {
                    move_type = MoveType::Quiet;
                }
            }
            _ => {
                if captured_piece != Piece::Empty {
                    move_type = MoveType::Capture;
                    capture_piece_sq = to;
                } else {
                    move_type = MoveType::Quiet;
                }
            }
        }

        MoveInfo {
            move_type,
            from,
            to,
            moved_piece,
            captured_piece,
            capture_piece_sq,
            en_passant_sq: position.en_passant_sq,
            castling_rights: position.castling_rights,
            fullmove_count: position.fullmove_count(),
            halfmove_clock: position.halfmove_clock(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.move_type != MoveType::Invalid
    }
}
