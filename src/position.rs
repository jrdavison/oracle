use crate::bitboards::Bitboards;
use crate::moves::generate;
use crate::moves::info::MoveInfo;
use crate::utils::{Color, Direction, File, MoveType, Piece, PieceType, Rank, Square};
use num_traits::ToPrimitive;
use std::time::{Duration, Instant};

pub struct Position {
    pub board: [Piece; Square::Count as usize],
    pub bitboards: Bitboards,
    pub en_passant_square: Square,

    move_history: Vec<MoveInfo>,
    redo_history: Vec<MoveInfo>,

    compute_time: Duration,
    fullmove_count: i32,
    halfmove_clock: i32,
    side_to_move: Color,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            board: [Piece::Empty; Square::Count as usize],
            bitboards: Bitboards::default(),
            en_passant_square: Square::Count,
            move_history: Vec::new(),
            redo_history: Vec::new(),
            compute_time: Duration::default(),
            fullmove_count: 1,
            halfmove_clock: 0,
            side_to_move: Color::White,
        }
    }
}

impl Position {
    pub fn new(fen: &str) -> Position {
        init_from_fen(fen)
    }

    pub fn board_i32(&self) -> Vec<i32> {
        self.board.iter().map(|&piece| Piece::to_i32(&piece).unwrap()).collect()
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn halfmove_clock(&self) -> i32 {
        self.halfmove_clock
    }

    pub fn compute_time(&self) -> String {
        format!("{:?}", self.compute_time)
    }

    pub fn en_passant_square(&self) -> String {
        if self.en_passant_square == Square::Count {
            "-".into()
        } else {
            format!("{:?}", self.en_passant_square)
        }
    }

    pub fn move_history(&self) -> Vec<MoveInfo> {
        self.move_history.clone()
    }

    pub fn last_move(&self) -> MoveInfo {
        self.move_history.last().cloned().unwrap_or_default()
    }

    pub fn valid_move(&self, from: Square, to: Square) -> bool {
        let piece = self.board[from];
        if Piece::color_of(piece) != self.side_to_move {
            return false;
        }
        self.bitboards.is_valid_move(from, to)
    }

    pub fn compute_valid_moves(&mut self, color: Color) {
        let start = Instant::now();
        generate::compute_valid_moves(self, color);
        self.compute_time = start.elapsed();
    }

    pub fn move_piece(&mut self, from: Square, to: Square, clear_redo: bool) -> MoveInfo {
        if !self.valid_move(from, to) {
            return MoveInfo::default();
        }

        let move_info = MoveInfo::new(self, from, to);
        let moved_piece_color = Piece::color_of(move_info.moved_piece);

        // en passant only valid for one move
        self.en_passant_square = Square::Count;

        // move piece
        self.board[move_info.to as usize] = move_info.moved_piece;
        self.board[move_info.from as usize] = Piece::Empty;
        self.bitboards.unset_checkers(moved_piece_color, move_info.from);
        self.bitboards.set_checkers(moved_piece_color, move_info.to);

        match move_info.move_type {
            MoveType::Capture => {
                let capture_color = Piece::color_of(move_info.captured_piece);
                self.bitboards.unset_checkers(capture_color, move_info.capture_piece_sq);
            }
            MoveType::TwoSquarePush => {
                let enemy_forward = Direction::forward_direction(!moved_piece_color);
                self.en_passant_square = move_info.to + enemy_forward;
            }
            MoveType::EnPassant => {
                let capture_color = Piece::color_of(move_info.captured_piece);
                self.board[move_info.capture_piece_sq as usize] = Piece::Empty;
                self.bitboards.unset_checkers(capture_color, move_info.capture_piece_sq);
            }
            MoveType::Promotion => {
                // TODO: give user option to choose promotion piece
                self.board[move_info.to as usize] = Piece::make_piece(PieceType::Queen, moved_piece_color);
                if Piece::color_of(move_info.captured_piece) != Color::Both {
                    self.bitboards
                        .unset_checkers(Piece::color_of(move_info.captured_piece), move_info.capture_piece_sq);
                }
            }
            MoveType::Invalid => panic!("Invalid move"),
            MoveType::Quiet => {}
        }

        if self.side_to_move == Color::Black {
            self.fullmove_count += 1;
        }

        if move_info.move_type == MoveType::Capture || Piece::type_of(move_info.moved_piece) == PieceType::Pawn {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        self.side_to_move = !self.side_to_move;
        self.move_history.push(move_info.clone());

        // clear redo history if move is not a redo
        if clear_redo {
            self.redo_history.clear();
        }

        move_info
    }

    pub fn undo_move(&mut self) -> bool {
        if let Some(last_move) = self.move_history.pop() {
            let color = Piece::color_of(last_move.moved_piece);
            match last_move.move_type {
                MoveType::Quiet | MoveType::TwoSquarePush | MoveType::Capture | MoveType::Promotion => {
                    self.board[last_move.from as usize] = last_move.moved_piece;
                    self.board[last_move.to as usize] = last_move.captured_piece;
                    self.bitboards.set_checkers(color, last_move.from);
                    self.bitboards.unset_checkers(color, last_move.to);

                    if last_move.move_type == MoveType::Capture {
                        self.bitboards.set_checkers(!color, last_move.capture_piece_sq);
                    }
                }
                MoveType::EnPassant => {
                    let color = Piece::color_of(last_move.moved_piece);
                    self.board[last_move.from as usize] = last_move.moved_piece;
                    self.board[last_move.to as usize] = Piece::Empty;
                    self.board[last_move.capture_piece_sq as usize] = last_move.captured_piece;
                    self.bitboards.set_checkers(color, last_move.from);
                    self.bitboards.unset_checkers(color, last_move.to);
                    self.bitboards.set_checkers(!color, last_move.capture_piece_sq);
                    self.en_passant_square = last_move.to;
                }
                MoveType::Invalid => panic!("Invalid move"),
            }

            self.side_to_move = !self.side_to_move;

            // reset clocks
            if color == Color::Black {
                self.fullmove_count -= 1;
            }
            if last_move.move_type == MoveType::Capture || Piece::type_of(last_move.moved_piece) == PieceType::Pawn {
                self.halfmove_clock = last_move.halfmove_clock;
            } else {
                self.halfmove_clock -= 1;
            }

            self.redo_history.push(last_move);
            true
        } else {
            false
        }
    }

    pub fn redo_move(&mut self) -> bool {
        // TODO: bug in redoing promotion with capture moves
        if let Some(last_move) = self.redo_history.pop() {
            self.move_piece(last_move.from, last_move.to, false);
            true
        } else {
            false
        }
    }
}

fn init_from_fen(fen: &str) -> Position {
    /*
    More info about fen notation: https://www.chess.com/terms/fen-chess
    */
    let mut fen_parts = fen.split_whitespace();

    let mut position = Position::default();
    let mut file = File::FileA;
    let mut rank = Rank::Rank8;

    let pieces = fen_parts.next().unwrap_or("");
    for c in pieces.chars() {
        match c {
            '/' => {
                rank = rank - 1;
                file = File::FileA;
            }
            c if c.is_ascii_digit() => {
                let c_digit = c.to_digit(10).expect("Expected digit");
                file = file + (c_digit as u8);
            }
            _ => {
                let color = if c.is_uppercase() { Color::White } else { Color::Black };
                let sq = Square::make_square(file, rank);
                let piece_type = PieceType::make_piece_type(c);
                position.board[sq as usize] = Piece::make_piece(piece_type, color);
                position.bitboards.set_checkers(color, sq);
                file = file + 1u8;
            }
        }
    }

    position.side_to_move = match fen_parts.next().unwrap_or("w") {
        "w" => Color::White,
        "b" => Color::Black,
        _ => panic!("Invalid side to move"),
    };

    // TODO: castling rights
    let _ = fen_parts.next().unwrap_or("-");

    // TODO: en passant square
    let _ = fen_parts.next().unwrap_or("-");

    position.halfmove_clock = fen_parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0);
    position.fullmove_count = fen_parts.next().unwrap_or("1").parse::<i32>().unwrap_or(1);

    position
}
