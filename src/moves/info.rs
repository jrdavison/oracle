use crate::bitboards;
use crate::position::Position;
use crate::utils::{Direction, File, MoveType, Piece, PieceType, Rank, Square};
use slint::SharedString;

#[derive(Clone, Debug, Default)]
pub struct MoveInfo {
    pub move_type: MoveType,
    pub from: Square,
    pub to: Square,
    pub moved_piece: Piece,
    pub captured_piece: Piece,
    pub capture_piece_sq: Square,
    pub halfmove_clock: i32,
    pub notation: SharedString,
}

impl MoveInfo {
    pub fn new(from: Square, to: Square, position: &Position) -> MoveInfo {
        let moved_piece = position.board[from as usize];
        let move_type;
        let mut captured_piece = position.board[to as usize];
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
                    captured_piece = position.board[capture_piece_sq];
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

        let mut new_move = MoveInfo {
            move_type,
            from,
            to,
            moved_piece,
            captured_piece,
            capture_piece_sq,
            halfmove_clock: position.halfmove_clock(),
            notation: SharedString::default(),
        };
        new_move.set_algebraic_notation(position);

        new_move
    }

    pub fn is_valid(&self) -> bool {
        self.move_type != MoveType::Invalid
    }

    fn set_algebraic_notation(&mut self, position: &Position) {
        let piece_identifier = disambiguate_move(self, position);
        let to_square = format!("{:?}", self.to).to_lowercase();

        let move_string = match self.move_type {
            MoveType::Quiet | MoveType::TwoSquarePush => [piece_identifier, to_square].join(""),
            MoveType::Capture | MoveType::EnPassant => [piece_identifier, to_square].join("x"),
            MoveType::Promotion => {
                if self.captured_piece != Piece::Empty {
                    format!(
                        "{}x{}={}",
                        Square::file_of(self.from).to_notation_string(),
                        to_square,
                        PieceType::Queen.to_notation_string()
                    )
                } else {
                    format!("{}={}", to_square, PieceType::Queen.to_notation_string())
                }
            }
            _ => "not handled".into(),
        };

        // TODO: handle check/checkmate

        self.notation = SharedString::from(move_string);
    }
}

fn disambiguate_move(info: &MoveInfo, position: &Position) -> String {
    let piece_type = Piece::type_of(info.moved_piece);
    let original_attack = position.bitboards.get_valid_moves(info.from);

    let mut common_moves = original_attack;
    let mut piece_sqs = vec![info.from];
    for sq in Square::iter() {
        let other_piece = position.board[sq as usize];
        let _test = Piece::color_of(other_piece);
        if (sq != info.to && Piece::color_of(info.moved_piece) == Piece::color_of(other_piece))
            && (Piece::type_of(info.moved_piece) == Piece::type_of(other_piece))
        {
            let other_attack = position.bitboards.get_valid_moves(sq);
            let check_common_moves = original_attack & other_attack;
            if check_common_moves != 0 {
                piece_sqs.push(sq);
            }
            common_moves &= other_attack;
        }
    }

    if bitboards::is_bit_set(&common_moves, info.to) {
        let files = piece_sqs.iter().map(|&sq| Square::file_of(sq)).collect::<Vec<File>>();
        let ranks = piece_sqs.iter().map(|&sq| Square::rank_of(sq)).collect::<Vec<Rank>>();
        let files_are_same = files.iter().all(|&file| file == files[0]);
        let ranks_are_same = ranks.iter().all(|&rank| rank == ranks[0]);
        if !files_are_same {
            format!("{}{}", piece_type.to_notation_string(), Square::file_of(info.from).to_notation_string())
        } else if !ranks_are_same {
            format!("{}{}", piece_type.to_notation_string(), Square::rank_of(info.from).to_notation_string())
        } else {
            let from = format!("{:?}", info.from).to_lowercase();
            format!("{}{}", piece_type.to_notation_string(), from)
        }
    } else {
        format!("{}", piece_type.to_notation_string())
    }
}
