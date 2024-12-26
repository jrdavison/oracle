use crate::bitboards::Bitboards;
use crate::moves::compute;
use crate::moves::info::MoveInfo;
use crate::utils::{CastlingRights, Color, Direction, File, MoveType, Piece, PieceType, Rank, Square};
use num_traits::FromPrimitive;
use std::time::{Duration, Instant};

pub struct Position {
    pub bitboards: Bitboards,
    pub board: [Piece; Square::Count as usize],
    pub en_passant_sq: Square,
    pub king_squares: [Square; Color::Both as usize],
    pub castling_rights: CastlingRights,

    move_history: Vec<MoveInfo>,
    redo_history: Vec<MoveInfo>,

    compute_time: Duration,
    fullmove_count: i32,
    halfmove_clock: i32,
    side_to_move: Color,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            bitboards: Bitboards::default(),
            board: [Piece::Empty; Square::Count as usize],
            en_passant_sq: Square::Count,
            king_squares: [Square::Count; Color::Both as usize],
            castling_rights: CastlingRights::default(),

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

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn fullmove_count(&self) -> i32 {
        self.fullmove_count
    }

    pub fn halfmove_clock(&self) -> i32 {
        self.halfmove_clock
    }

    pub fn compute_time(&self) -> String {
        format!("{:?}", self.compute_time)
    }

    pub fn en_passant_sq(&self) -> String {
        if self.en_passant_sq == Square::Count {
            "-".into()
        } else {
            format!("{:?}", self.en_passant_sq)
        }
    }

    pub fn move_history(&self) -> Vec<MoveInfo> {
        self.move_history.clone()
    }

    pub fn redo_history(&self) -> Vec<MoveInfo> {
        self.redo_history.clone()
    }

    pub fn last_move(&self) -> MoveInfo {
        self.move_history.last().cloned().unwrap_or_default()
    }

    pub fn valid_move(&self, from: Square, to: Square) -> bool {
        let piece = self.board[from as usize];
        if Piece::color_of(piece) != self.side_to_move {
            return false;
        }
        self.bitboards.is_valid_move(from, to)
    }

    pub fn compute_valid_moves(&mut self, color: Color) {
        let start = Instant::now();
        compute::compute_valid_moves(self, color);
        self.compute_time = start.elapsed();
    }

    pub fn move_piece(&mut self, from: Square, to: Square, clear_redo: bool) -> MoveInfo {
        if !self.valid_move(from, to) {
            return MoveInfo::default();
        }

        let move_info = MoveInfo::new(self, from, to);
        let moved_piece_color = Piece::color_of(move_info.moved_piece);
        let moved_piece_type = Piece::type_of(move_info.moved_piece);

        // en passant only valid for one move
        self.en_passant_sq = Square::Count;

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
                self.en_passant_sq = move_info.to + enemy_forward;
            }
            MoveType::EnPassant => {
                let capture_color = Piece::color_of(move_info.captured_piece);
                self.board[move_info.capture_piece_sq as usize] = Piece::Empty;
                self.bitboards.unset_checkers(capture_color, move_info.capture_piece_sq);
            }
            MoveType::Promotion => {
                // TODO: give user option to choose promotion piece
                self.board[move_info.to as usize] = Piece::from(PieceType::Queen, moved_piece_color);
                if Piece::color_of(move_info.captured_piece) != Color::Both {
                    self.bitboards
                        .unset_checkers(Piece::color_of(move_info.captured_piece), move_info.capture_piece_sq);
                }
            }
            MoveType::Castle => {
                let (rook_from, rook_to) = match move_info.to {
                    Square::G1 => (Square::H1, Square::F1),
                    Square::C1 => (Square::A1, Square::D1),
                    Square::G8 => (Square::H8, Square::F8),
                    Square::C8 => (Square::A8, Square::D8),
                    _ => (Square::Count, Square::Count),
                };

                self.board[rook_to as usize] = self.board[rook_from as usize];
                self.board[rook_from as usize] = Piece::Empty;
                self.bitboards.unset_checkers(moved_piece_color, rook_from);
                self.bitboards.set_checkers(moved_piece_color, rook_to);
            }
            MoveType::Quiet | MoveType::Invalid => {}
        }

        if moved_piece_type == PieceType::King {
            self.king_squares[moved_piece_color as usize] = move_info.to;
            let rights_to_unset = match moved_piece_color {
                Color::White => CastlingRights::WhiteCastling,
                Color::Black => CastlingRights::BlackCastling,
                _ => CastlingRights::default(),
            };
            self.castling_rights.unset_castling_rights(rights_to_unset);
            // TODO: get this working again if we redo moves
        }

        if moved_piece_type == PieceType::Rook {
            // if rook is moved from starting square, unset castling rights
            let rights_to_unset = match (moved_piece_color, move_info.from) {
                (Color::White, Square::A1) => CastlingRights::WhiteOOO,
                (Color::White, Square::H1) => CastlingRights::WhiteOO,
                (Color::Black, Square::A8) => CastlingRights::BlackOOO,
                (Color::Black, Square::H8) => CastlingRights::BlackOO,
                _ => CastlingRights::default(),
            };
            self.castling_rights.unset_castling_rights(rights_to_unset);
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

                    if last_move.captured_piece != Piece::Empty {
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
                    self.en_passant_sq = last_move.to;
                }
                MoveType::Castle => {
                    let (rook_from, rook_to) = match last_move.to {
                        Square::G1 => (Square::H1, Square::F1),
                        Square::C1 => (Square::A1, Square::D1),
                        Square::G8 => (Square::H8, Square::F8),
                        Square::C8 => (Square::A8, Square::D8),
                        _ => (Square::Count, Square::Count),
                    };

                    // reset king
                    self.board[last_move.from as usize] = last_move.moved_piece;
                    self.board[last_move.to as usize] = Piece::Empty;
                    self.bitboards.set_checkers(color, last_move.from);
                    self.bitboards.unset_checkers(color, last_move.to);

                    // reset rook
                    self.board[rook_from as usize] = self.board[rook_to as usize];
                    self.board[rook_to as usize] = Piece::Empty;
                    self.bitboards.set_checkers(color, rook_from);
                    self.bitboards.unset_checkers(color, rook_to);
                }
                MoveType::Invalid => panic!("Invalid move"),
            }

            let moved_piece_type = Piece::type_of(last_move.moved_piece);
            if moved_piece_type == PieceType::King {
                self.king_squares[color as usize] = last_move.from;
            }

            self.side_to_move = !self.side_to_move;
            self.en_passant_sq = last_move.en_passant_sq;
            self.halfmove_clock = last_move.halfmove_clock;
            self.fullmove_count = last_move.fullmove_count;

            self.redo_history.push(last_move);
            true
        } else {
            false
        }
    }

    pub fn redo_move(&mut self) -> bool {
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
                let sq = Square::from(file, rank);
                let piece_type = PieceType::from_char(c);
                position.board[sq as usize] = Piece::from(piece_type, color);
                position.bitboards.set_checkers(color, sq);

                if piece_type == PieceType::King {
                    position.king_squares[color as usize] = sq;
                }

                file = file + 1u8;
            }
        }
    }

    position.side_to_move = match fen_parts.next().unwrap_or("w") {
        "w" => Color::White,
        "b" => Color::Black,
        _ => panic!("Invalid side to move"),
    };

    let castling = fen_parts.next().unwrap_or("-");
    let mut castling_mask = 0u8;
    for c in castling.chars() {
        match c {
            'K' => castling_mask |= CastlingRights::WhiteOO as u8,
            'Q' => castling_mask |= CastlingRights::WhiteOOO as u8,
            'k' => castling_mask |= CastlingRights::BlackOO as u8,
            'q' => castling_mask |= CastlingRights::BlackOOO as u8,
            _ => {}
        }
    }
    position.castling_rights = CastlingRights::from_u8(castling_mask).unwrap_or_default();

    let en_passant_str = fen_parts.next().unwrap_or("-").to_lowercase();
    position.en_passant_sq = Square::from_string(&en_passant_str);

    position.halfmove_clock = fen_parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0);
    position.fullmove_count = fen_parts.next().unwrap_or("1").parse::<i32>().unwrap_or(1);

    position
}
