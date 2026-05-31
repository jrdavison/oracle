use super::compute::{KINGSIDE_CASTLE_SQUARES, QUEENSIDE_CASTLE_SQUARES};
use crate::bitboards;
use crate::position::Position;
use crate::utils::{CastlingRights, Direction, File, MoveType, Piece, PieceType, Rank, Square};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
}

pub type UndoInfo = MoveInfo;

#[derive(Clone, Copy, Debug)]
pub struct MoveList {
    pub moves: [Move; 256],
    pub len: usize,
}

impl Default for MoveList {
    fn default() -> MoveList {
        MoveList {
            moves: [Move::default(); 256],
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

#[derive(Clone, Debug, Default)]
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
    pub notation: String,
}

impl MoveInfo {
    pub fn new(position: &Position, from: Square, to: Square) -> MoveInfo {
        MoveInfo::from_position(position, from, to, true)
    }

    pub fn new_without_notation(position: &Position, from: Square, to: Square) -> MoveInfo {
        MoveInfo::from_position(position, from, to, false)
    }

    fn from_position(position: &Position, from: Square, to: Square, include_notation: bool) -> MoveInfo {
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

        let mut new_move = MoveInfo {
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
            notation: String::default(),
        };
        if include_notation {
            new_move.set_algebraic_notation(position);
        }

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
                        Square::file_of(self.from).make_notation_string(),
                        to_square,
                        PieceType::Queen.make_notation_string()
                    )
                } else {
                    format!("{}={}", to_square, PieceType::Queen.make_notation_string())
                }
            }
            MoveType::Castle => {
                if self.to == KINGSIDE_CASTLE_SQUARES[Piece::color_of(self.moved_piece) as usize] {
                    "O-O".into()
                } else {
                    "O-O-O".into()
                }
            }
            _ => "not handled".into(),
        };

        // TODO: handle check/checkmate

        self.notation = move_string;
    }
}

fn disambiguate_move(info: &MoveInfo, position: &Position) -> String {
    let piece_type = Piece::type_of(info.moved_piece);
    let original_attack = position.bitboards.get_legal_moves(info.from);

    let mut common_moves = original_attack;
    let mut piece_sqs = vec![info.from];
    for sq in Square::iter() {
        let other_piece = position.board[sq as usize];
        if (sq != info.from && Piece::color_of(info.moved_piece) == Piece::color_of(other_piece))
            && (Piece::type_of(info.moved_piece) == Piece::type_of(other_piece))
        {
            let other_attack = position.bitboards.get_legal_moves(sq);
            let check_common_moves = original_attack & other_attack;
            if check_common_moves != 0 {
                piece_sqs.push(sq);
            }
            common_moves &= other_attack;
        }
    }

    let formatted_string = if bitboards::is_bit_set(common_moves, info.to) && piece_sqs.len() > 1 {
        let files = piece_sqs.iter().map(|&sq| Square::file_of(sq)).collect::<Vec<File>>();
        let ranks = piece_sqs.iter().map(|&sq| Square::rank_of(sq)).collect::<Vec<Rank>>();
        let files_are_same = files.iter().all(|&file| file == files[0]);
        let ranks_are_same = ranks.iter().all(|&rank| rank == ranks[0]);
        if !files_are_same {
            format!(
                "{}{}",
                piece_type.make_notation_string(),
                Square::file_of(info.from).make_notation_string()
            )
        } else if !ranks_are_same {
            format!(
                "{}{}",
                piece_type.make_notation_string(),
                Square::rank_of(info.from).make_notation_string()
            )
        } else {
            let from = format!("{:?}", info.from).to_lowercase();
            format!("{}{}", piece_type.make_notation_string(), from)
        }
    } else if piece_type == PieceType::Pawn
        && (info.move_type == MoveType::EnPassant || info.move_type == MoveType::Capture)
    {
        // add file of pawns automatically during captures (and en passant)
        Square::file_of(info.from).make_notation_string().to_string()
    } else {
        piece_type.make_notation_string().to_string()
    };

    formatted_string
}
