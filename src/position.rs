use crate::utils;
use std::fmt;

pub struct Position {
    board: [utils::Piece; utils::SQUARE_NB],
    side_to_move: utils::Color,
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
    pub fn new(fen: &str) -> Position {
        load_from_fen(fen)
    }

    pub fn get_board(&self) -> [utils::Piece; utils::SQUARE_NB] {
        self.board
    }
}

fn load_from_fen(fen: &str) -> Position {
    println!("Loading from FEN: {}", fen);

    let mut fen_parts = fen.split_whitespace();

    let mut file: utils::File = utils::File::FileA;
    let mut rank: utils::Rank = utils::Rank::Rank8;
    let mut board = [utils::Piece::NoPiece; utils::SQUARE_NB];

    let pieces = fen_parts.next().unwrap_or("");
    for c in pieces.chars() {
        match c {
            '/' => {
                rank = rank - 1;
                file = utils::File::FileA;
            }
            c if c.is_digit(10) => {
                let c_digit = c.to_digit(10).expect("Expected digit");
                file = file + (c_digit as u8);
            }
            _ => {
                let color = if c.is_uppercase() {
                    utils::Color::White
                } else {
                    utils::Color::Black
                };
                let sq = utils::Square::make_square(file, rank);
                let piece_type = from_char(c);
                board[sq as usize] = utils::Piece::make_piece(piece_type, color);
                file = file + 1;
            }
        }
    }

    Position {
        board,
        side_to_move: utils::Color::White,
    }
}

fn from_char(c: char) -> utils::PieceType {
    let c_lower = c.to_lowercase().next().unwrap_or(' ');
    match c_lower {
        'k' => utils::PieceType::King,
        'q' => utils::PieceType::Queen,
        'b' => utils::PieceType::Bishop,
        'n' => utils::PieceType::Knight,
        'r' => utils::PieceType::Rook,
        'p' => utils::PieceType::Pawn,
        _ => utils::PieceType::NoPiece,
    }
}
