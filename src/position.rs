use num_traits::ToPrimitive;

use crate::utils::{constants, types};

pub struct Position {
    board: [types::Piece; constants::SQUARE_NB],
    side_to_move: types::Color,
    halfmove_clock: i32,
    fullmove_number: i32,
}

impl Position {
    pub fn new(fen: &str) -> Self {
        load_from_fen(fen)
    }

    pub fn get_board_i32(&self) -> Vec<i32> {
        self.board
            .iter()
            .map(|&piece| types::Piece::to_i32(&piece).unwrap())
            .collect()
    }

    pub fn get_side_to_move(&self) -> i32 {
        types::Color::to_i32(&self.side_to_move).unwrap()
    }

    pub fn get_halfmove_clock(&self) -> i32 {
        self.halfmove_clock
    }

    pub fn get_fullmove_number(&self) -> i32 {
        self.fullmove_number
    }

    pub fn move_piece(&mut self, src: types::Square, dest: types::Square) {
        println!("Move from {:?} to {:?}", src, dest);

        let src_piece = self.board[src];
        self.board[src] = types::Piece::NoPiece;
        self.board[dest] = src_piece;

        // TODO: store move history for undos
    }
}

fn load_from_fen(fen: &str) -> Position {
    // https://www.chess.com/terms/fen-chess

    println!("Loading from FEN: {}", fen);

    let mut fen_parts = fen.split_whitespace();

    let mut file = types::File::FileA;
    let mut rank = types::Rank::Rank8;
    let mut board = [types::Piece::NoPiece; constants::SQUARE_NB];

    let pieces = fen_parts.next().unwrap_or("");
    for c in pieces.chars() {
        match c {
            '/' => {
                rank = rank - 1;
                file = types::File::FileA;
            }
            c if c.is_digit(10) => {
                let c_digit = c.to_digit(10).expect("Expected digit");
                file = file + (c_digit as u8);
            }
            _ => {
                let color = if c.is_uppercase() {
                    types::Color::White
                } else {
                    types::Color::Black
                };
                let sq = types::Square::make_square(file, rank);
                let piece_type = types::PieceType::make_piece_type(c);
                board[sq as usize] = types::Piece::make_piece(piece_type, color);
                file = file + 1;
            }
        }
    }

    let side_to_move = match fen_parts.next().unwrap_or("w") {
        "w" => types::Color::White,
        "b" => types::Color::Black,
        _ => panic!("Invalid side to move"),
    };

    // TODO: castling rights
    let _ = fen_parts.next().unwrap_or("-");

    // TODO: en passant square
    let _ = fen_parts.next().unwrap_or("-");

    let halfmove_clock = fen_parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0);
    let fullmove_number = fen_parts.next().unwrap_or("1").parse::<i32>().unwrap_or(1);

    Position {
        board,
        side_to_move,
        halfmove_clock,
        fullmove_number,
    }
}
