use crate::bitboards;
use crate::bitboards::{Bitboards, ComputedMoves};
use crate::move_info::MoveInfo;
use crate::utils::constants;
use crate::utils::helpers;
use crate::utils::types::{
    Bitboard, BlockersMoveDatabase, Color, Direction, File, MoveType, Piece, PieceType, Rank, SimpleMoveDatabase,
    Square,
};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

static KNIGHT_MOVES_DB: Lazy<SimpleMoveDatabase> = Lazy::new(|| helpers::load_simple_move_db("knight_moves.bin"));
static KING_MOVES_DB: Lazy<SimpleMoveDatabase> = Lazy::new(|| helpers::load_simple_move_db("king_moves.bin"));
static ROOK_MOVES_DB: Lazy<BlockersMoveDatabase> = Lazy::new(|| helpers::load_blockers_move_db("rook_moves.bin"));

pub struct Position {
    board: [Piece; Square::Count as usize],
    bitboards: Bitboards,
    en_passant_square: Square,
    move_history: Vec<MoveInfo>,

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

    pub fn fullmove_count(&self) -> i32 {
        self.fullmove_count
    }

    pub fn compute_time(&self) -> String {
        format!("{:?}", self.compute_time)
    }

    pub fn valid_move(&self, from: Square, to: Square) -> bool {
        let piece = self.board[from];
        if Piece::color_of(piece) != self.side_to_move {
            return false;
        }
        self.bitboards.is_valid_move(from, to)
    }

    fn compute_pawn_moves(&self, sq: Square) -> ComputedMoves {
        let mut valid_moves = 0;

        let piece = self.board[sq];
        let color = Piece::color_of(piece);
        let forward = Direction::forward_direction(color);

        // normal move
        let mut target_sq = sq;
        for i in 0..2 {
            // only allow double move from starting rank
            if i == 1 && (Rank::relative_rank(color, Square::rank_of(sq)) != Rank::Rank2) {
                break;
            }

            target_sq = target_sq + forward;
            if (target_sq != Square::Count) && !self.bitboards.is_checkers_sq_set(Color::Both, target_sq) {
                bitboards::set_bit(&mut valid_moves, target_sq);
            } else {
                break;
            }
        }

        // capture moves
        let mut attacks = 0; // pawns can only attack diagonally
        let target_sq_east = (sq + forward) + Direction::East;
        if target_sq_east != Square::Count {
            bitboards::set_bit(&mut attacks, sq);
            if self.bitboards.is_checkers_sq_set(!color, target_sq_east) {
                bitboards::set_bit(&mut valid_moves, target_sq_east);
            }
        }

        let target_sq_west = (sq + forward) + Direction::West;
        if target_sq_west != Square::Count {
            bitboards::set_bit(&mut attacks, sq);
            if self.bitboards.is_checkers_sq_set(!color, target_sq_west) {
                bitboards::set_bit(&mut valid_moves, target_sq_west);
            }
        }

        // en passant
        if self.en_passant_square != Square::Count {
            if target_sq_east == self.en_passant_square {
                bitboards::set_bit(&mut valid_moves, target_sq_east);
            } else if target_sq_west == self.en_passant_square {
                bitboards::set_bit(&mut valid_moves, target_sq_west);
            }
        }

        // TODO: promotion

        ComputedMoves {
            valid_moves,
            attacks: valid_moves,
        }
    }

    fn compute_knight_moves(&self, sq: Square) -> ComputedMoves {
        let valid_moves = KNIGHT_MOVES_DB[sq as usize];
        ComputedMoves {
            valid_moves,
            attacks: valid_moves,
        }
    }

    fn compute_rook_moves(&self, sq: Square) -> ComputedMoves {
        let rank = Square::rank_of(sq);
        let file = Square::file_of(sq);

        let h_mask = constants::HORIZONTAL_MASK << (rank as u64 * 8);
        let v_mask = constants::VERTICAL_MASK << (file as u64);
        let move_mask = (h_mask | v_mask) & !(1u64 << (sq as u64));

        let blocker_key = self.bitboards.get_checkers(Color::Both) & move_mask;
        let valid_moves = *ROOK_MOVES_DB[sq as usize]
            .get(&blocker_key)
            .unwrap_or(&Bitboard::default());

        ComputedMoves {
            valid_moves,
            attacks: valid_moves,
        }
    }

    fn compute_bishop_moves(&self, sq: Square) -> ComputedMoves {
        let mut valid_moves = 0;

        let mut target_square_ne = sq + Direction::North + Direction::East;
        while target_square_ne != Square::Count {
            if self.bitboards.is_checkers_sq_set(Color::Both, target_square_ne) {
                bitboards::set_bit(&mut valid_moves, target_square_ne);
                break;
            }
            bitboards::set_bit(&mut valid_moves, target_square_ne);
            target_square_ne = target_square_ne + Direction::North + Direction::East;
        }

        let mut target_square_nw = sq + Direction::North + Direction::West;
        while target_square_nw != Square::Count {
            bitboards::set_bit(&mut valid_moves, target_square_nw);
            if self.bitboards.is_checkers_sq_set(Color::Both, target_square_nw) {
                break;
            }
            target_square_nw = target_square_nw + Direction::North + Direction::West;
        }

        let mut target_square_se = sq + Direction::South + Direction::East;
        while target_square_se != Square::Count {
            bitboards::set_bit(&mut valid_moves, target_square_se);
            if self.bitboards.is_checkers_sq_set(Color::Both, target_square_se) {
                break;
            }
            target_square_se = target_square_se + Direction::South + Direction::East;
        }

        let mut target_square_sw = sq + Direction::South + Direction::West;
        while target_square_sw != Square::Count {
            bitboards::set_bit(&mut valid_moves, target_square_sw);
            if self.bitboards.is_checkers_sq_set(Color::Both, target_square_sw) {
                break;
            }
            target_square_sw = target_square_sw + Direction::South + Direction::West;
        }

        ComputedMoves {
            valid_moves,
            attacks: valid_moves,
        }
    }

    fn compute_king_moves(&self, sq: Square) -> ComputedMoves {
        // TODO: castling
        let valid_moves = KING_MOVES_DB[sq as usize];
        ComputedMoves {
            valid_moves,
            attacks: valid_moves,
        }
    }

    pub fn compute_valid_moves(&mut self, color: Color) {
        let start = Instant::now();

        let mut attacks = 0;
        for sq in Square::iter() {
            let piece = self.board[sq];
            let piece_type = Piece::type_of(piece);
            let mut computed_moves = ComputedMoves::default();

            /*
            only compute moves for pieces of the correct color.
            Valid moves and attacks will be reset back to 0 for the enemy color
            */
            if color == Piece::color_of(piece) {
                match piece_type {
                    PieceType::Pawn => computed_moves = self.compute_pawn_moves(sq),
                    PieceType::Knight => computed_moves = self.compute_knight_moves(sq),
                    PieceType::Rook => computed_moves = self.compute_rook_moves(sq),
                    PieceType::Bishop => computed_moves = self.compute_bishop_moves(sq),
                    PieceType::Queen => computed_moves = self.compute_rook_moves(sq) | self.compute_bishop_moves(sq),
                    PieceType::King => computed_moves = self.compute_king_moves(sq),
                    _ => {}
                }
            }

            /*
            TODO: check if move puts king in check (diagonal and horizontal pins)

            we will need to copy the position and then use that to simulate the move. Then we check if the king is in
            check
            */

            computed_moves.valid_moves &= !self.bitboards.get_checkers(color);
            self.bitboards.set_valid_moves(sq, computed_moves.valid_moves);
            attacks |= computed_moves.attacks;
        }

        self.bitboards.set_attacks(color, attacks);

        self.compute_time = start.elapsed();
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> MoveInfo {
        if !self.valid_move(from, to) {
            return MoveInfo::default(); // defaults to invalid move
        }

        // reset en passant square
        self.en_passant_square = Square::Count;

        let move_info = MoveInfo::new(from, to, &self.board);
        let moved_piece_color = Piece::color_of(move_info.moved_piece);

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
            MoveType::Promotion => println!("Promotion not handled"),
            MoveType::Invalid => panic!("Invalid move"),
            MoveType::Quiet => {}
        }

        if self.side_to_move == Color::Black {
            self.fullmove_count += 1;
        }

        self.side_to_move = !self.side_to_move;

        // TODO: count halfmoves

        self.move_history.push(move_info);
        move_info
    }

    pub fn undo_move(&mut self) -> bool {
        if let Some(last_move) = self.move_history.pop() {
            match last_move.move_type {
                MoveType::Quiet | MoveType::TwoSquarePush | MoveType::Capture => {
                    let color = Piece::color_of(last_move.moved_piece);
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
                }
                MoveType::Invalid => panic!("Invalid move"),
                _ => {
                    println!("Move not handled: {:?}", last_move.move_type);
                }
            }

            self.side_to_move = !self.side_to_move;
            if self.side_to_move == Color::Black {
                self.fullmove_count -= 1;
            }

            true
        } else {
            false
        }
    }
}

pub fn load_move_dbs() {
    Lazy::force(&KNIGHT_MOVES_DB);
    Lazy::force(&KING_MOVES_DB);
    Lazy::force(&ROOK_MOVES_DB);
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
                file = file + 1;
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
