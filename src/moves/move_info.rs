use crate::types::{Direction, MoveType, Piece, PieceType, Rank, Square};

#[derive(Clone, Copy, Debug, Default)]
pub struct MoveInfo {
    pub move_type: MoveType,
    pub from: Square,
    pub to: Square,
    pub moved_piece: Piece,
    pub captured_piece: Piece,
    pub capture_piece_sq: Square,
}

impl MoveInfo {
    pub fn new(from: Square, to: Square, board: &[Piece; Square::Count as usize]) -> MoveInfo {
        let moved_piece = board[from as usize];
        let move_type;
        let mut captured_piece = board[to as usize];
        let mut capture_piece_sq = Square::Count;

        match Piece::type_of(moved_piece) {
            PieceType::Pawn => {
                let color = Piece::color_of(moved_piece);
                let from_rank = Square::rank_of(from);
                let from_file = Square::file_of(from);
                let to_rank = Square::rank_of(to);
                let to_file = Square::file_of(to);

                if Rank::relative_rank(color, from_rank) == Rank::Rank2 {
                    move_type = MoveType::TwoSquarePush;
                } else if captured_piece != Piece::Empty {
                    move_type = MoveType::Capture;
                    capture_piece_sq = to;
                } else if from_file != to_file {
                    move_type = MoveType::EnPassant;
                    capture_piece_sq = to + Direction::forward_direction(!color);
                    captured_piece = board[capture_piece_sq];
                } else if Rank::relative_rank(color, to_rank) == Rank::Rank8 {
                    // TODO: promotion
                    move_type = MoveType::Promotion;
                } else {
                    move_type = MoveType::Quiet;
                }
            }
            PieceType::King => {
                // TODO: castling moves
                if captured_piece != Piece::Empty {
                    move_type = MoveType::Capture;
                    capture_piece_sq = to;
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
        }
    }

    pub fn is_valid(&self) -> bool {
        self.move_type != MoveType::Invalid
    }
}
