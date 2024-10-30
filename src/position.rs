use crate::utils::constants;
use crate::utils::types::{
    Bitboard, Color, File, KnightMoveDatabase, Piece, PieceType, Rank, RookMoveDatabase, Square,
};
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Cursor, Read};
use std::time::Instant;

pub struct Position {
    board: [Piece; Square::SquareNb as usize],

    // TODO: maybe move this into a separate struct bitboard.rs
    valid_moves: [Bitboard; Square::SquareNb as usize],
    checkers_bb: [Bitboard; Color::ColorNb as usize],

    side_to_move: Color,
    halfmove_clock: i32,
    fullmove_count: i32,

    knight_moves_db: KnightMoveDatabase,
    rook_moves_db: RookMoveDatabase,
}

impl Position {
    pub fn new(fen: &str) -> Position {
        load_from_fen(fen)
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

    pub fn is_valid_move(&self, from: Square, to: Square) -> bool {
        let piece = self.board[from];
        if Piece::color_of(piece) != self.side_to_move {
            return false;
        }
        is_bit_set(self.valid_moves[from as usize], to)
    }

    fn get_checkers_bb(&self, color: Color) -> Bitboard {
        if color == Color::ColorNb {
            return self.checkers_bb[Color::White as usize] | self.checkers_bb[Color::Black as usize];
        }
        self.checkers_bb[color as usize]
    }

    fn compute_knight_moves(&self, sq: Square) -> Bitboard {
        self.knight_moves_db[sq as usize]
    }

    fn compute_rook_moves(&self, sq: Square) -> Bitboard {
        let mut valid_moves = Bitboard::default();

        let rank = Square::rank_of(sq);
        let file = Square::file_of(sq);

        let h_mask = constants::HORIZONTAL_MASK << (rank as u64 * 8);
        let v_mask = constants::VERTICAL_MASK << (file as u64);
        let move_mask = (h_mask | v_mask) & !(1u64 << (sq as u64));

        let blocker_key = self.get_checkers_bb(Color::ColorNb) & move_mask;
        valid_moves = self.rook_moves_db[sq as usize]
            .get(&blocker_key)
            .unwrap_or(&Bitboard::default())
            .clone();

        valid_moves
    }

    pub fn compute_valid_moves(&mut self, color: Color) {
        let start = Instant::now();

        for sq in Square::iter() {
            let piece = self.board[sq];
            if Piece::color_of(piece) == color {
                match Piece::type_of(piece) {
                    PieceType::Pawn => {}
                    PieceType::Knight => {
                        self.valid_moves[sq as usize] = self.compute_knight_moves(sq);
                    }
                    PieceType::King => {}
                    PieceType::Queen => {}
                    PieceType::Bishop => {}
                    PieceType::Rook => {
                        self.valid_moves[sq as usize] = self.compute_rook_moves(sq);
                    }
                    _ => {}
                }
            }
            self.valid_moves[sq as usize] &= !self.checkers_bb[color as usize];
        }

        let duration = start.elapsed();
        println!("Time elapsed in compute_valid_moves() is: {:?}", duration);
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> bool {
        if !self.is_valid_move(from, to) {
            return false;
        }

        let piece = self.board[from];
        self.board[from] = Piece::NoPiece;
        self.board[to] = piece;

        if self.side_to_move == Color::Black {
            self.fullmove_count += 1;
        }

        self.side_to_move = !self.side_to_move;
        // TODO: store move history for undos
        // TODO: count halfmoves
        true
    }
}

fn load_from_fen(fen: &str) -> Position {
    /*
    More info about fen notation: https://www.chess.com/terms/fen-chess
    */
    let mut fen_parts = fen.split_whitespace();

    let mut file = File::FileA;
    let mut rank = Rank::Rank8;
    let mut board = [Piece::NoPiece; Square::SquareNb as usize];
    let mut checkers_bb = [Bitboard::default(); Color::ColorNb as usize];

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
                board[sq as usize] = Piece::make_piece(piece_type, color);
                set_bit(&mut checkers_bb[color as usize], sq);
                file = file + 1;
            }
        }
    }

    let side_to_move = match fen_parts.next().unwrap_or("w") {
        "w" => Color::White,
        "b" => Color::Black,
        _ => panic!("Invalid side to move"),
    };

    // TODO: castling rights
    let _ = fen_parts.next().unwrap_or("-");

    // TODO: en passant square
    let _ = fen_parts.next().unwrap_or("-");

    let halfmove_clock = fen_parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0);
    let fullmove_count = fen_parts.next().unwrap_or("1").parse::<i32>().unwrap_or(1);

    Position {
        board,
        checkers_bb,
        side_to_move,
        halfmove_clock,
        fullmove_count,
        valid_moves: [Bitboard::default(); Square::SquareNb as usize], // TODO: probably a better way to default this
        knight_moves_db: load_knight_move_db(),
        rook_moves_db: load_rook_move_db(),
    }
}

fn load_knight_move_db() -> KnightMoveDatabase {
    let file = constants::DATA_DIR
        .get_file("knight_moves.bin")
        .expect("Failed to get file");
    let data = file.contents();

    assert_eq!(data.len(), (Square::SquareNb as usize) * 8, "Invalid data length!");

    let mut knight_moves = [Bitboard::default(); Square::SquareNb as usize];
    for (i, bb) in knight_moves.iter_mut().enumerate() {
        let start = i * 8;
        let end = start + 8;
        *bb = u64::from_le_bytes(data[start..end].try_into().unwrap());
    }

    knight_moves
}

pub fn load_rook_move_db() -> RookMoveDatabase {
    let file = constants::DATA_DIR
        .get_file("rook_moves.bin")
        .expect("Failed to get file");
    let mut reader = Cursor::new(file.contents());

    let mut rook_moves: RookMoveDatabase = std::array::from_fn(|_| HashMap::new());
    for sq in Square::iter() {
        let mut moves: HashMap<Bitboard, Bitboard> = HashMap::new();

        let mut num_entries_buf = [0u8; 4];
        reader
            .read_exact(&mut num_entries_buf)
            .expect("Failed to read number of entries");
        let num_entries = u32::from_le_bytes(num_entries_buf);
        for _ in 0..num_entries {
            let mut blockers_buf = [0u8; 8];
            let mut attacks_buf = [0u8; 8];

            reader.read_exact(&mut blockers_buf).expect("Failed to read blockers");
            reader.read_exact(&mut attacks_buf).expect("Failed to read attacks");

            let blockers = u64::from_le_bytes(blockers_buf);
            let attacks = u64::from_le_bytes(attacks_buf);

            moves.insert(blockers, attacks);
        }
        rook_moves[sq as usize] = moves;
    }

    rook_moves
}

// TODO: move these to bitboard.rs
fn set_bit(bitboard: &mut Bitboard, sq: Square) {
    *bitboard |= 1u64 << sq as u64;
}

fn is_bit_set(bitboard: Bitboard, sq: Square) -> bool {
    bitboard & (1u64 << sq as u64) != 0
}
