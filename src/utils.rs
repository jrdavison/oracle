use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::fmt;
use std::ops::{Add, Mul, Not, Sub};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MoveType {
    #[default]
    Invalid,

    Quiet,
    Capture,
    EnPassant,
    TwoSquarePush,
    // Castle,
    Promotion,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, FromPrimitive, PartialEq, ToPrimitive)]
pub enum Color {
    White,
    Black,

    #[default]
    Both = 2,
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Color {
        let inverted = !(self as u8) & 1;
        Color::from_u8(inverted).unwrap_or_default()
    }
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, FromPrimitive, PartialEq)]
pub enum PieceType {
    #[default]
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
        let c_lower = c.to_lowercase().next().unwrap_or('\0');
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

    pub fn make_notation_string(&self) -> &str {
        match self {
            PieceType::Pawn => "",
            PieceType::King => "K",
            PieceType::Queen => "Q",
            PieceType::Bishop => "B",
            PieceType::Knight => "N",
            PieceType::Rook => "R",
            PieceType::Empty => "???",
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, FromPrimitive, PartialEq, ToPrimitive)]
pub enum Piece {
    #[default]
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
        Piece::from_u8((pt as u8) + ((c as u8) << 3)).unwrap_or_default()
    }

    pub fn color_of(piece: Piece) -> Color {
        if piece == Piece::Empty {
            return Color::Both;
        }
        Color::from_u8((piece as u8) >> 3).unwrap_or_default()
    }

    pub fn type_of(piece: Piece) -> PieceType {
        PieceType::from_u8(piece as u8 & 0b111).unwrap_or_default()
    }
}

#[repr(u8)]
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, Default, FromPrimitive, PartialEq, PartialOrd)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,

    #[default]
    Count = 64,
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

        Square::from_i8(new_sq).unwrap_or_default()
    }
}

impl Square {
    pub fn make_square(file: File, rank: Rank) -> Square {
        Square::from_u8((rank as u8) << 3 | (file as u8)).unwrap_or_default()
    }

    pub fn rank_of(sq: Square) -> Rank {
        Rank::from_u8((sq as u8) >> 3).unwrap_or_default()
    }

    pub fn file_of(sq: Square) -> File {
        File::from_u8(sq as u8 & 0b111).unwrap_or_default()
    }

    pub fn iter() -> impl Iterator<Item = Square> {
        (0..(Square::Count as usize)).filter_map(|i| Square::from_u8(i as u8))
    }

    fn is_valid(sq: i8) -> bool {
        Square::A1 as i8 <= sq && sq < Square::Count as i8
    }
}

#[repr(i8)]
#[derive(Clone, Copy, Debug, Default, FromPrimitive, PartialEq)]
pub enum Direction {
    #[default]
    Invalid,

    North = 8,
    East = 1,
    South = -8,
    West = -1,
}

impl Not for Direction {
    type Output = Direction;
    fn not(self) -> Direction {
        let inverted = -(self as i8);
        Direction::from_i8(inverted).unwrap_or_default()
    }
}

impl Direction {
    pub fn forward_direction(c: Color) -> Direction {
        match c {
            Color::White => Direction::North,
            Color::Black => Direction::South,
            _ => Direction::Invalid,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, FromPrimitive, PartialEq)]
pub enum File {
    FileA,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH,

    #[default]
    Count = 8,
}

impl File {
    pub fn from_x(x: f32) -> File {
        let file = (x.floor() as i32) / 80; // TODO: don't hardcode square size
        File::from_i32(file).unwrap_or_default()
    }

    pub fn iter() -> impl Iterator<Item = File> {
        (0..(File::Count as usize)).filter_map(|i| File::from_u8(i as u8))
    }

    pub fn make_notation_string(&self) -> &str {
        match self {
            File::FileA => "a",
            File::FileB => "b",
            File::FileC => "c",
            File::FileD => "d",
            File::FileE => "e",
            File::FileF => "f",
            File::FileG => "g",
            File::FileH => "h",
            _ => "???",
        }
    }
}

impl Add<u8> for File {
    type Output = File;
    fn add(self, rhs: u8) -> File {
        File::from_u8(self as u8 + rhs).unwrap_or_default()
    }
}

impl Sub<u8> for File {
    type Output = File;
    fn sub(self, rhs: u8) -> File {
        File::from_u8((self as u8).wrapping_sub(rhs)).unwrap_or_default()
    }
}

impl Add<i8> for File {
    type Output = File;
    fn add(self, rhs: i8) -> File {
        File::from_i8(self as i8 + rhs).unwrap_or_default()
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, FromPrimitive, PartialEq)]
pub enum Rank {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,

    #[default]
    Count = 8,
}

impl Rank {
    pub fn from_y(y: f32) -> Rank {
        let rank = 7 - (y.floor() as i32 / 80); // TODO: don't hardcode square size
        Rank::from_i32(rank).unwrap_or_default()
    }

    pub fn relative_rank(color: Color, rank: Rank) -> Rank {
        match color {
            Color::White => rank,
            Color::Black => Rank::Rank8 - (rank as u8),
            _ => Rank::Count,
        }
    }

    pub fn iter_reverse() -> impl Iterator<Item = Rank> {
        (0..(Rank::Count as usize)).rev().filter_map(|i| Rank::from_u8(i as u8))
    }

    pub fn make_notation_string(&self) -> &str {
        match self {
            Rank::Rank1 => "1",
            Rank::Rank2 => "2",
            Rank::Rank3 => "3",
            Rank::Rank4 => "4",
            Rank::Rank5 => "5",
            Rank::Rank6 => "6",
            Rank::Rank7 => "7",
            Rank::Rank8 => "8",
            _ => "???",
        }
    }
}

impl Add<u8> for Rank {
    type Output = Rank;
    fn add(self, rhs: u8) -> Rank {
        Rank::from_u8(self as u8 + rhs).unwrap_or_default()
    }
}

impl Sub<u8> for Rank {
    type Output = Rank;
    fn sub(self, rhs: u8) -> Rank {
        Rank::from_u8((self as u8).wrapping_sub(rhs)).unwrap_or_default()
    }
}

impl Add<i8> for Rank {
    type Output = Rank;
    fn add(self, rhs: i8) -> Rank {
        Rank::from_i8(self as i8 + rhs).unwrap_or_default()
    }
}

impl Mul<u8> for Rank {
    type Output = Rank;
    fn mul(self, rhs: u8) -> Rank {
        Rank::from_u8(self as u8 * rhs).unwrap_or_default()
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self as u8 + 1)
    }
}
