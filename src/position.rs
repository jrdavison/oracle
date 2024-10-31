use crate::bitboards;
use crate::utils::constants;
use crate::utils::types::{
    Bitboard, Color, Direction, File, KnightMoveDatabase, Piece, PieceType, Rank, RookMoveDatabase, Square,
};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

static KNIGHT_MOVES_DB: Lazy<KnightMoveDatabase> = Lazy::new(|| bitboards::load_knight_move_db());
static ROOK_MOVES_DB: Lazy<RookMoveDatabase> = Lazy::new(|| bitboards::load_rook_move_db());

pub struct Bitboards {
    valid_moves: [Bitboard; Square::SquareNb as usize],
    checkers: [Bitboard; Color::ColorNb as usize],
}

impl Default for Bitboards {
    fn default() -> Self {
        Bitboards {
            valid_moves: [0; Square::SquareNb as usize],
            checkers: [0; Color::ColorNb as usize],
        }
    }
}

pub struct Position {
    board: [Piece; Square::SquareNb as usize],

    side_to_move: Color,
    halfmove_clock: i32,
    fullmove_count: i32,
    compute_time: Duration,

    bitboards: Bitboards,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            board: [Piece::NoPiece; Square::SquareNb as usize],
            side_to_move: Color::White,
            halfmove_clock: 0,
            fullmove_count: 1,
            compute_time: Duration::default(),
            bitboards: Bitboards::default(),
        }
    }
}

impl Position {
    pub fn new(fen: &str) -> Position {
        init_from_fen(fen)
    }

    pub fn get_board_i32(&self) -> Vec<i32> {
        self.board.iter().map(|&piece| Piece::to_i32(&piece).unwrap()).collect()
    }

    pub fn get_side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn get_halfmove_clock(&self) -> i32 {
        self.halfmove_clock
    }

    pub fn get_fullmove_count(&self) -> i32 {
        self.fullmove_count
    }

    pub fn get_compute_time_string(&self) -> String {
        format!("{:?}", self.compute_time)
    }

    pub fn is_valid_move(&self, from: Square, to: Square) -> bool {
        let piece = self.board[from];
        if Piece::color_of(piece) != self.side_to_move {
            return false;
        }
        return bitboards::is_bit_set(self.bitboards.valid_moves[from as usize], to);
    }

    fn get_checkers_bb(&self, color: Color) -> Bitboard {
        if color == Color::ColorNb {
            return self.bitboards.checkers[Color::White as usize] | self.bitboards.checkers[Color::Black as usize];
        }
        return self.bitboards.checkers[color as usize];
    }

    fn compute_pawn_moves(&self, sq: Square) -> Bitboard {
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
            if (target_sq != Square::SquareNb)
                && !bitboards::is_bit_set(self.get_checkers_bb(Color::ColorNb), target_sq)
            {
                bitboards::set_bit(&mut valid_moves, target_sq);
            } else {
                break;
            }

            // TODO: set en passant square
        }

        // capture moves
        let target_sq_east = (sq + forward) + Direction::East;
        if (target_sq_east != Square::SquareNb) && bitboards::is_bit_set(self.get_checkers_bb(!color), target_sq_east) {
            bitboards::set_bit(&mut valid_moves, target_sq_east);
        }

        let target_sq_west = (sq + forward) + Direction::West;
        if (target_sq_west != Square::SquareNb) && bitboards::is_bit_set(self.get_checkers_bb(!color), target_sq_west) {
            bitboards::set_bit(&mut valid_moves, target_sq_west);
        }

        // TODO: capture en passant

        return valid_moves;
    }

    fn compute_knight_moves(&self, sq: Square) -> Bitboard {
        KNIGHT_MOVES_DB[sq as usize]
    }

    fn compute_rook_moves(&self, sq: Square) -> Bitboard {
        let rank = Square::rank_of(sq);
        let file = Square::file_of(sq);

        let h_mask = constants::HORIZONTAL_MASK << (rank as u64 * 8);
        let v_mask = constants::VERTICAL_MASK << (file as u64);
        let move_mask = (h_mask | v_mask) & !(1u64 << (sq as u64));

        let blocker_key = self.get_checkers_bb(Color::ColorNb) & move_mask;
        let valid_moves = ROOK_MOVES_DB[sq as usize]
            .get(&blocker_key)
            .unwrap_or(&Bitboard::default())
            .clone();

        return valid_moves;
    }

    fn compute_bishop_moves(&self, sq: Square) -> Bitboard {
        let mut valid_moves = 0;

        let mut target_square_ne = sq + Direction::North + Direction::East;
        while target_square_ne != Square::SquareNb {
            if bitboards::is_bit_set(self.get_checkers_bb(Color::ColorNb), target_square_ne) {
                bitboards::set_bit(&mut valid_moves, target_square_ne);
                break;
            }
            bitboards::set_bit(&mut valid_moves, target_square_ne);
            target_square_ne = target_square_ne + Direction::North + Direction::East;
        }

        let mut target_square_nw = sq + Direction::North + Direction::West;
        while target_square_nw != Square::SquareNb {
            if bitboards::is_bit_set(self.get_checkers_bb(Color::ColorNb), target_square_nw) {
                bitboards::set_bit(&mut valid_moves, target_square_nw);
                break;
            }
            bitboards::set_bit(&mut valid_moves, target_square_nw);
            target_square_nw = target_square_nw + Direction::North + Direction::West;
        }

        let mut target_square_se = sq + Direction::South + Direction::East;
        while target_square_se != Square::SquareNb {
            if bitboards::is_bit_set(self.get_checkers_bb(Color::ColorNb), target_square_se) {
                bitboards::set_bit(&mut valid_moves, target_square_se);
                break;
            }
            bitboards::set_bit(&mut valid_moves, target_square_se);
            target_square_se = target_square_se + Direction::South + Direction::East;
        }

        let mut target_square_sw = sq + Direction::South + Direction::West;
        while target_square_sw != Square::SquareNb {
            if bitboards::is_bit_set(self.get_checkers_bb(Color::ColorNb), target_square_sw) {
                bitboards::set_bit(&mut valid_moves, target_square_sw);
                break;
            }
            bitboards::set_bit(&mut valid_moves, target_square_sw);
            target_square_sw = target_square_sw + Direction::South + Direction::West;
        }

        return valid_moves;
    }

    fn compute_king_moves(&self, sq: Square) -> Bitboard {
        return 0;
    }

    pub fn compute_valid_moves(&mut self, color: Color) {
        let start = Instant::now();

        // TODO: probably need to keep attack bitboards for each color

        for sq in Square::iter() {
            let piece = self.board[sq];
            self.bitboards.valid_moves[sq as usize] = 0; // clear previous moves
            if Piece::color_of(piece) == color {
                match Piece::type_of(piece) {
                    PieceType::Pawn => {
                        self.bitboards.valid_moves[sq as usize] = self.compute_pawn_moves(sq);
                    }
                    PieceType::Knight => {
                        self.bitboards.valid_moves[sq as usize] = self.compute_knight_moves(sq);
                    }
                    PieceType::Rook => {
                        self.bitboards.valid_moves[sq as usize] = self.compute_rook_moves(sq);
                    }
                    PieceType::Bishop => {
                        self.bitboards.valid_moves[sq as usize] = self.compute_bishop_moves(sq);
                    }
                    PieceType::Queen => {
                        self.bitboards.valid_moves[sq as usize] =
                            self.compute_rook_moves(sq) | self.compute_bishop_moves(sq);
                    }
                    PieceType::King => {
                        self.bitboards.valid_moves[sq as usize] = self.compute_king_moves(sq);
                    }
                    _ => {}
                }
            }

            // TODO: check if king is in check

            // can't capture own pieces
            self.bitboards.valid_moves[sq as usize] &= !self.get_checkers_bb(color);
        }

        let duration = start.elapsed();
        println!("Computed moves in {:?}", duration);
        self.compute_time = duration;
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> bool {
        if !self.is_valid_move(from, to) {
            return false;
        }

        let piece = self.board[from];
        self.board[from] = Piece::NoPiece;
        self.board[to] = piece;

        bitboards::clear_bit(&mut self.bitboards.checkers[Piece::color_of(piece) as usize], from);
        bitboards::set_bit(&mut self.bitboards.checkers[Piece::color_of(piece) as usize], to);

        if self.side_to_move == Color::Black {
            self.fullmove_count += 1;
        }

        self.side_to_move = !self.side_to_move;
        // TODO: store move history for undos
        // TODO: count halfmoves
        return true;
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
            c if c.is_digit(10) => {
                let c_digit = c.to_digit(10).expect("Expected digit");
                file = file + (c_digit as u8);
            }
            _ => {
                let color = if c.is_uppercase() { Color::White } else { Color::Black };
                let sq = Square::make_square(file, rank);
                let piece_type = PieceType::make_piece_type(c);
                position.board[sq as usize] = Piece::make_piece(piece_type, color);
                bitboards::set_bit(&mut position.bitboards.checkers[color as usize], sq);
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

    return position;
}
