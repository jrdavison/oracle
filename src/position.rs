use crate::bitboards::{self, Bitboards};
use crate::moves::compute;
use crate::moves::info::MoveInfo;
use crate::utils::{CastlingRights, Color, Direction, File, MoveType, Piece, PieceType, Rank, Square};
use num_traits::FromPrimitive;
use std::time::{Duration, Instant};

pub struct Position {
    pub active_squares: [Vec<Square>; Color::Both as usize],
    pub bitboards: Bitboards,
    pub board: [Piece; Square::Count as usize],
    pub castling_rights: CastlingRights,
    pub en_passant_sq: Square,
    pub king_squares: [Square; Color::Both as usize],
    pub side_to_move: Color,

    move_history: Vec<MoveInfo>,
    redo_history: Vec<MoveInfo>,

    total_compute_time: Duration,
    total_moves: u32,

    fullmove_count: i32,
    halfmove_clock: i32,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            active_squares: [Vec::new(), Vec::new()],
            bitboards: Bitboards::default(),
            board: [Piece::Empty; Square::Count as usize],
            castling_rights: CastlingRights::default(),
            en_passant_sq: Square::Count,
            king_squares: [Square::Count; Color::Both as usize],

            move_history: Vec::new(),
            redo_history: Vec::new(),

            total_compute_time: Duration::default(),
            total_moves: 0,

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

    pub fn avg_compute_time(&self) -> String {
        format!("{:?}", self.total_compute_time / self.total_moves)
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

    pub fn king_in_check(&self, color: Color) -> bool {
        let enemy_attacks = self.bitboards.get_attacks(!color);
        let king_sq = self.king_squares[color as usize];
        bitboards::is_bit_set(enemy_attacks, king_sq)
    }

    pub fn valid_move(&self, from: Square, to: Square) -> bool {
        if !Square::is_valid(from as i8) || !Square::is_valid(to as i8) {
            return false;
        }
        let piece = self.board[from as usize];
        if Piece::color_of(piece) != self.side_to_move {
            return false;
        }
        self.bitboards.is_valid_move(from, to)
    }

    pub fn compute_valid_moves(&mut self) {
        let start = Instant::now();
        compute::compute_valid_moves(self);
        let delta = start.elapsed();
        self.total_compute_time += delta;
        self.total_moves += 1;
        println!("Time to compute valid moves: {:?}", delta);
    }

    pub fn move_piece(&mut self, from: Square, to: Square, clear_redo: bool) -> MoveInfo {
        let start = Instant::now();
        if !self.valid_move(from, to) {
            return MoveInfo::default();
        }

        let move_info = MoveInfo::new(self, from, to);
        let moved_piece_color = Piece::color_of(move_info.moved_piece);
        let moved_piece_type = Piece::type_of(move_info.moved_piece);

        // en passant only valid for one move
        self.en_passant_sq = Square::Count;

        // clear the old square first
        self.remove_piece(move_info.from);

        // handle special moves
        match move_info.move_type {
            MoveType::TwoSquarePush => {
                let enemy_forward = Direction::forward_direction(!moved_piece_color);
                self.en_passant_sq = move_info.to + enemy_forward;
            }
            MoveType::EnPassant | MoveType::Capture => {
                self.remove_piece(move_info.capture_piece_sq);
            }
            MoveType::Promotion => {
                // TODO: give user option to choose promotion piece
                self.board[move_info.to as usize] = Piece::from(PieceType::Queen, moved_piece_color);
                if Piece::color_of(move_info.captured_piece) != Color::Both {
                    self.remove_piece(move_info.capture_piece_sq);
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

                let rook = self.board[rook_from as usize];
                self.add_piece(rook_to, rook);
                self.remove_piece(rook_from);
            }
            MoveType::Quiet | MoveType::Invalid => {}
        }

        // move the piece to the new square
        self.add_piece(move_info.to, move_info.moved_piece);

        if moved_piece_type == PieceType::King {
            self.king_squares[moved_piece_color as usize] = move_info.to;
            let rights_to_unset = match moved_piece_color {
                Color::White => CastlingRights::WhiteCastling,
                Color::Black => CastlingRights::BlackCastling,
                _ => CastlingRights::default(),
            };
            self.castling_rights.unset_castling_rights(rights_to_unset);
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

        println!("Time to make move: {:?}", start.elapsed());

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
            self.castling_rights = last_move.castling_rights;
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

    pub fn valid_moves(&self) -> Vec<MoveInfo> {
        let mut moves = Vec::new();
        for sq in self.active_squares[self.side_to_move as usize].iter() {
            let valid_moves = self.bitboards.get_valid_moves(*sq);
            for to in Square::iter() {
                if bitboards::is_bit_set(valid_moves, to) {
                    let move_info = MoveInfo::new(self, *sq, to);
                    moves.push(move_info);
                }
            }
        }
        moves
    }

    fn remove_piece(&mut self, sq: Square) {
        let piece = self.board[sq as usize];
        let color = Piece::color_of(piece);
        self.board[sq as usize] = Piece::Empty;
        self.bitboards.unset_checkers(color, sq);
        self.active_squares[color as usize].retain(|&s| s != sq);
    }

    fn add_piece(&mut self, sq: Square, piece: Piece) {
        let color = Piece::color_of(piece);
        self.board[sq as usize] = piece;
        self.bitboards.set_checkers(color, sq);
        self.active_squares[color as usize].push(sq);
    }
}

pub fn count_valid_moves(pos: &mut Position, ply: u32) -> u32 {
    if ply == 0 {
        return 1;
    }

    pos.compute_valid_moves();
    let moves = pos.valid_moves();
    if ply == 1 {
        return moves.len() as u32;
    }

    let mut nodes = 0;
    for mv in moves {
        let made = pos.move_piece(mv.from, mv.to, false);
        if made.move_type == MoveType::Invalid {
            continue;
        }
        nodes += count_valid_moves(pos, ply - 1);
        let _ = pos.undo_move();
    }

    nodes
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

                position.active_squares[color as usize].push(sq);

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
