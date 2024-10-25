use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::ops::{Add, Sub};

use crate::utils::constants::SQUARE_NB;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Color {
    White,
    Black,
    ColorNb = 2,
}

impl From<Color> for i32 {
    fn from(color: Color) -> Self {
        color as i32
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum PieceType {
    NoPiece,
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

impl PieceType {
    pub fn make_piece_type(c: char) -> PieceType {
        let c_lower = c.to_lowercase().next().unwrap_or(' ');
        match c_lower {
            'k' => PieceType::King,
            'q' => PieceType::Queen,
            'b' => PieceType::Bishop,
            'n' => PieceType::Knight,
            'r' => PieceType::Rook,
            'p' => PieceType::Pawn,
            _ => PieceType::NoPiece,
        }
    }
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum Piece {
    NoPiece,
    WKing = PieceType::King as u8,
    WQueen,
    WBishop,
    WKnight,
    WRook,
    WPawn,
    BKing = PieceType::King as u8 + 8,
    BQueen,
    BBishop,
    BKnight,
    BRook,
    BPawn,
}

impl Default for Piece {
    fn default() -> Self {
        Piece::NoPiece
    }
}

impl From<Piece> for i32 {
    fn from(piece: Piece) -> Self {
        piece as i32
    }
}

impl Piece {
    pub fn make_piece(pt: PieceType, c: Color) -> Piece {
        Piece::from_u8((pt as u8) + ((c as u8) << 3)).unwrap_or(Piece::NoPiece)
    }
}

#[repr(u8)]
#[rustfmt::skip]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum Square {
    SqA1, SqB1, SqC1, SqD1, SqE1, SqF1, SqG1, SqH1,
    SqA2, SqB2, SqC2, SqD2, SqE2, SqF2, SqG2, SqH2,
    SqA3, SqB3, SqC3, SqD3, SqE3, SqF3, SqG3, SqH3,
    SqA4, SqB4, SqC4, SqD4, SqE4, SqF4, SqG4, SqH4,
    SqA5, SqB5, SqC5, SqD5, SqE5, SqF5, SqG5, SqH5,
    SqA6, SqB6, SqC6, SqD6, SqE6, SqF6, SqG6, SqH6,
    SqA7, SqB7, SqC7, SqD7, SqE7, SqF7, SqG7, SqH7,
    SqA8, SqB8, SqC8, SqD8, SqE8, SqF8, SqG8, SqH8,

    SquareNb = SQUARE_NB as u8,
}

impl Square {
    pub fn make_square(file: File, rank: Rank) -> Square {
        Square::from_u8((rank as u8) << 3 | (file as u8)).unwrap_or(Square::SquareNb)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum File {
    FileA,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH,
    FileNb = 8,
}

impl Add<u8> for File {
    type Output = File;
    fn add(self, rhs: u8) -> File {
        File::from_u8(self as u8 + rhs).unwrap_or(File::FileNb)
    }
}

impl Sub<u8> for File {
    type Output = File;
    fn sub(self, rhs: u8) -> File {
        File::from_u8(self as u8 - rhs).unwrap_or(File::FileNb)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum Rank {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,
    RankNb = 8,
}

impl Add<u8> for Rank {
    type Output = Rank;
    fn add(self, rhs: u8) -> Rank {
        Rank::from_u8(self as u8 + rhs).unwrap_or(Rank::RankNb)
    }
}

impl Sub<u8> for Rank {
    type Output = Rank;
    fn sub(self, rhs: u8) -> Rank {
        Rank::from_u8(self as u8 - rhs).unwrap_or(Rank::RankNb)
    }
}
