use crate::bitboards;
use crate::moves::compute::{KINGSIDE_CASTLE_SQUARES, QUEENSIDE_CASTLE_SQUARES};
use crate::moves::info::{Move, MoveInfo};
use crate::position::Position;
use crate::utils::{File, MoveType, Piece, PieceType, Rank, Square};

#[derive(Clone, Debug, Default)]
pub struct GameMove {
    pub info: MoveInfo,
    pub notation: String,
}

pub struct GameState {
    pub position: Position,
    move_history: Vec<GameMove>,
    redo_history: Vec<GameMove>,
}

impl GameState {
    pub fn new(fen: &str) -> GameState {
        GameState {
            position: Position::new(fen),
            move_history: Vec::new(),
            redo_history: Vec::new(),
        }
    }

    pub fn move_history(&self) -> &[GameMove] {
        &self.move_history
    }

    pub fn redo_history(&self) -> &[GameMove] {
        &self.redo_history
    }

    pub fn last_move(&self) -> MoveInfo {
        self.move_history.last().map(|mv| mv.info).unwrap_or_default()
    }

    pub fn play_move(&mut self, from: Square, to: Square) -> Option<MoveInfo> {
        let mv = Move { from, to };
        if !self.position.is_legal_move(mv.from, mv.to) {
            return None;
        }

        let move_preview = MoveInfo::new(&self.position, mv.from, mv.to);
        let notation = algebraic_notation(&move_preview, &self.position);
        let move_info = self.position.move_piece(mv, true)?;
        self.move_history.push(GameMove {
            info: move_info,
            notation,
        });
        self.redo_history.clear();
        Some(move_info)
    }

    pub fn undo_move(&mut self) -> bool {
        if let Some(last_move) = self.move_history.pop() {
            self.position.undo_move(last_move.info);
            self.redo_history.push(last_move);
            true
        } else {
            false
        }
    }

    pub fn redo_move(&mut self) -> bool {
        if let Some(last_move) = self.redo_history.pop() {
            let mv = Move {
                from: last_move.info.from,
                to: last_move.info.to,
            };
            if let Some(move_info) = self.position.move_piece(mv, true) {
                self.move_history.push(GameMove {
                    info: move_info,
                    notation: last_move.notation,
                });
                true
            } else {
                self.redo_history.push(last_move);
                false
            }
        } else {
            false
        }
    }
}

fn algebraic_notation(info: &MoveInfo, position: &Position) -> String {
    let piece_identifier = disambiguate_move(info, position);
    let to_square = format!("{:?}", info.to).to_lowercase();

    match info.move_type {
        MoveType::Quiet | MoveType::TwoSquarePush => [piece_identifier, to_square].join(""),
        MoveType::Capture | MoveType::EnPassant => [piece_identifier, to_square].join("x"),
        MoveType::Promotion => {
            if info.captured_piece != Piece::Empty {
                format!(
                    "{}x{}={}",
                    Square::file_of(info.from).make_notation_string(),
                    to_square,
                    PieceType::Queen.make_notation_string()
                )
            } else {
                format!("{}={}", to_square, PieceType::Queen.make_notation_string())
            }
        }
        MoveType::Castle => {
            if info.to == KINGSIDE_CASTLE_SQUARES[Piece::color_of(info.moved_piece) as usize] {
                "O-O".into()
            } else if info.to == QUEENSIDE_CASTLE_SQUARES[Piece::color_of(info.moved_piece) as usize] {
                "O-O-O".into()
            } else {
                "not handled".into()
            }
        }
        MoveType::Invalid => "not handled".into(),
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

    if bitboards::is_bit_set(common_moves, info.to) && piece_sqs.len() > 1 {
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
        Square::file_of(info.from).make_notation_string().to_string()
    } else {
        piece_type.make_notation_string().to_string()
    }
}
