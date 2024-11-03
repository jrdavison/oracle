use core::panic;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::ops::{Add, Sub};
use std::ops::{Index, IndexMut, Not};

pub type Bitboard = u64;
pub type BlockersMoveDatabase = [HashMap<Bitboard, Bitboard>; Square::Count as usize];
pub type SimpleMoveDatabase = [Bitboard; Square::Count as usize];

#[repr(u8)]
#[derive(PartialEq)]
pub enum MoveType {
    Invalid,
    Quiet,
    Capture,
    EnPassant,
    TwoSquarePush,
    Castle,
    Promotion,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum Color {
    White,
    Black,
    Count = 2,
}

impl Not for Color {
    type Output = Color;
    fn not(self) -> Color {
        let inverted = !(self as u8) & 1;
        Color::from_u8(inverted).expect("Invalid color")
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq)]
pub enum PieceType {
    Empty,
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
            _ => PieceType::Empty,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum Piece {
    Empty,
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

impl Piece {
    pub fn make_piece(pt: PieceType, c: Color) -> Piece {
        Piece::from_u8((pt as u8) + ((c as u8) << 3)).expect("Cannot make piece")
    }

    pub fn color_of(piece: Piece) -> Color {
        Color::from_u8((piece as u8) >> 3).expect("Cannot get color of piece")
    }

    pub fn type_of(piece: Piece) -> PieceType {
        PieceType::from_u8(piece as u8 & 0b111).expect("Cannot get type of piece")
    }
}

#[repr(u8)]
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq, PartialOrd)]
pub enum Square {
    SqA1, SqB1, SqC1, SqD1, SqE1, SqF1, SqG1, SqH1,
    SqA2, SqB2, SqC2, SqD2, SqE2, SqF2, SqG2, SqH2,
    SqA3, SqB3, SqC3, SqD3, SqE3, SqF3, SqG3, SqH3,
    SqA4, SqB4, SqC4, SqD4, SqE4, SqF4, SqG4, SqH4,
    SqA5, SqB5, SqC5, SqD5, SqE5, SqF5, SqG5, SqH5,
    SqA6, SqB6, SqC6, SqD6, SqE6, SqF6, SqG6, SqH6,
    SqA7, SqB7, SqC7, SqD7, SqE7, SqF7, SqG7, SqH7,
    SqA8, SqB8, SqC8, SqD8, SqE8, SqF8, SqG8, SqH8,

    Count = 64,
}

impl Index<Square> for [Piece; Square::Count as usize] {
    type Output = Piece;
    fn index(&self, index: Square) -> &Piece {
        &self[index as usize]
    }
}

impl IndexMut<Square> for [Piece; Square::Count as usize] {
    fn index_mut(&mut self, index: Square) -> &mut Piece {
        &mut self[index as usize]
    }
}

impl Add<Direction> for Square {
    type Output = Square;
    fn add(self, rhs: Direction) -> Square {
        let new_sq = (self as i8).wrapping_add(rhs as i8);

        // prevent wrapping north-south movement
        if !Square::is_valid(new_sq) {
            return Square::Count;
        }

        // prevent wrapping east-west movement
        let file = Square::file_of(self);
        if (file == File::FileA && rhs == Direction::West) || (file == File::FileH && rhs == Direction::East) {
            return Square::Count;
        }

        Square::from_i8(new_sq).expect("Cannot add direction to square")
    }
}

// impl

impl Square {
    pub fn make_square(file: File, rank: Rank) -> Square {
        Square::from_u8((rank as u8) << 3 | (file as u8)).expect("Cannot make square")
    }

    pub fn rank_of(sq: Square) -> Rank {
        Rank::from_u8((sq as u8) >> 3).expect("Cannot get rank of square")
    }

    pub fn file_of(sq: Square) -> File {
        File::from_u8(sq as u8 & 0b111).expect("Cannot get file of square")
    }

    pub fn iter() -> impl Iterator<Item = Square> {
        (0..(Square::Count as usize)).map(|i| Square::from_u8(i as u8).unwrap())
    }

    pub fn is_valid(sq: i8) -> bool {
        Square::SqA1 as i8 <= sq && sq < Square::Count as i8
    }
}

#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum Direction {
    North = 8,
    East = 1,
    South = -8,
    West = -1,
}

impl Not for Direction {
    type Output = Direction;
    fn not(self) -> Direction {
        let inverted = -(self as i8);
        Direction::from_i8(inverted).expect("Cannot invert direction")
    }
}

impl Direction {
    pub fn forward_direction(c: Color) -> Direction {
        match c {
            Color::White => Direction::North,
            Color::Black => Direction::South,
            _ => panic!("Invalid color"),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq)]
pub enum File {
    FileA,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH,
    Count = 8,
}

impl File {
    pub fn from_x(x: f32) -> File {
        let file = (x.floor() as i32) / 80; // TODO: don't hardcode square size
        File::from_i32(file).expect("Cannot get file from x")
    }

    pub fn iter() -> impl Iterator<Item = File> {
        (0..(File::Count as usize)).map(|i| File::from_u8(i as u8).unwrap())
    }
}

impl Add<u8> for File {
    type Output = File;
    fn add(self, rhs: u8) -> File {
        File::from_u8(self as u8 + rhs).unwrap_or(File::Count)
    }
}

impl Sub<u8> for File {
    type Output = File;
    fn sub(self, rhs: u8) -> File {
        File::from_u8(self as u8 - rhs).unwrap_or(File::Count)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq)]
pub enum Rank {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,
    Count = 8,
}

impl Rank {
    pub fn from_y(y: f32) -> Rank {
        let rank = 7 - (y.floor() as i32 / 80); // TODO: don't hardcode square size
        Rank::from_i32(rank).expect("Cannot get rank from y")
    }

    pub fn relative_rank(color: Color, rank: Rank) -> Rank {
        match color {
            Color::White => rank,
            Color::Black => Rank::Rank8 - (rank as u8),
            _ => panic!("Invalid color"),
        }
    }

    pub fn iter() -> impl Iterator<Item = Rank> {
        (0..(Rank::Count as usize)).map(|i| Rank::from_u8(i as u8).unwrap())
    }

    pub fn iter_reverse() -> impl Iterator<Item = Rank> {
        (0..(Rank::Count as usize))
            .rev()
            .map(|i| Rank::from_u8(i as u8).unwrap())
    }
}

impl Add<u8> for Rank {
    type Output = Rank;
    fn add(self, rhs: u8) -> Rank {
        Rank::from_u8(self as u8 + rhs).unwrap_or(Rank::Count)
    }
}

impl Sub<u8> for Rank {
    type Output = Rank;
    fn sub(self, rhs: u8) -> Rank {
        Rank::from_u8(self as u8 - rhs).unwrap_or(Rank::Count)
    }
}
