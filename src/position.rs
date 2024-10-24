use crate::utils::{constants, types};
use std::fmt;

pub struct Position {
    board: [types::Piece; constants::SQUARE_NB],
    side_to_move: types::Color,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            board: [types::Piece::default(); constants::SQUARE_NB],
            side_to_move: types::Color::default(),
        }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Position {{\n  board: {{\n")?;

        for (index, value) in self.board.iter().enumerate() {
            write!(f, "    [{}]: {:?},\n", index, value)?;
        }

        write!(f, "  }},\n  side_to_move: {:?}\n}}", self.side_to_move)
    }
}

impl Position {
    pub fn new(fen: &str) -> Self {
        load_from_fen(fen)
    }

    pub fn get_board(&self) -> [types::Piece; constants::SQUARE_NB] {
        self.board
    }
}

fn load_from_fen(fen: &str) -> Position {
    println!("Loading from FEN: {}", fen);

    let mut fen_parts = fen.split_whitespace();

    let mut file: types::File = types::File::FileA;
    let mut rank: types::Rank = types::Rank::Rank8;
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

    Position {
        board,
        side_to_move: types::Color::White,
    }
}
