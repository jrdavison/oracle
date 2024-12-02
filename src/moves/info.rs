use crate::utils::{Direction, MoveType, Piece, PieceType, Rank, Square};
use slint::SharedString;

#[derive(Clone, Copy, Debug, Default)]
pub struct MoveInfo {
    pub move_type: MoveType,
    pub from: Square,
    pub to: Square,
    pub moved_piece: Piece,
    pub captured_piece: Piece,
    pub capture_piece_sq: Square,
    pub halfmove_clock: i32,
}

impl MoveInfo {
    pub fn new(from: Square, to: Square, board: &[Piece; Square::Count as usize], halfmove_clock: i32) -> MoveInfo {
        let moved_piece = board[from as usize];
        let move_type;
        let mut captured_piece = board[to as usize];
        let mut capture_piece_sq = Square::Count;

        match Piece::type_of(moved_piece) {
            PieceType::Pawn => {
                let color = Piece::color_of(moved_piece);
                let from_rank = Square::rank_of(from);
                let from_file = Square::file_of(from);
                let to_file = Square::file_of(to);

                let relative_from_rank = Rank::relative_rank(color, from_rank);
                if relative_from_rank == Rank::Rank7 {
                    move_type = MoveType::Promotion;
                    capture_piece_sq = to;
                } else if relative_from_rank == Rank::Rank2 {
                    move_type = MoveType::TwoSquarePush;
                } else if captured_piece != Piece::Empty {
                    move_type = MoveType::Capture;
                    capture_piece_sq = to;
                } else if from_file != to_file {
                    move_type = MoveType::EnPassant;
                    capture_piece_sq = to + Direction::forward_direction(!color);
                    captured_piece = board[capture_piece_sq];
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
            halfmove_clock,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.move_type != MoveType::Invalid
    }

    pub fn to_algebraic_notation(&self) -> SharedString {
        let to_square = format!("{:?}", self.to).to_lowercase();
        let piece_type = Piece::type_of(self.moved_piece);

        let move_string = match self.move_type {
            MoveType::Quiet | MoveType::TwoSquarePush => {
                format!("{}{}", piece_type.to_string(), to_square)
            }
            MoveType::Capture | MoveType::EnPassant => {
                if piece_type == PieceType::Pawn {
                    format!("{}x{}", Square::file_of(self.from).to_string(), to_square)
                } else {
                    format!("{}x{}", piece_type.to_string(), to_square)
                }
            }
            MoveType::Promotion => {
                if self.captured_piece != Piece::Empty {
                    format!("{}x{}={}", Square::file_of(self.from).to_string(), to_square, PieceType::Queen.to_string())
                } else {
                    format!("{}={}", to_square, PieceType::Queen.to_string())
                }
            }
            _ => "not handled".into(),
        };

        // TODO: handle check/checkmate/ambiguous moves

        SharedString::from(move_string)
    }
}
