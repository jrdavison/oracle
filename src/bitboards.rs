/*
https://tearth.dev/bitboard-viewer/
*/

use crate::utils::{Color, File, Rank, Square};

pub type Bitboard = u64;

pub struct Bitboards {
    valid_moves: [Bitboard; Square::Count as usize],
    checkers: [Bitboard; Color::Both as usize],
    attacks: [Bitboard; Color::Both as usize],
}

impl Default for Bitboards {
    fn default() -> Self {
        Bitboards {
            valid_moves: [0; Square::Count as usize],
            checkers: [0; Color::Both as usize],
            attacks: [0; Color::Both as usize],
        }
    }
}

impl Bitboards {
    pub fn set_valid_moves(&mut self, sq: Square, bb: Bitboard) {
        self.valid_moves[sq as usize] = bb;
    }

    pub fn get_valid_moves(&self, sq: Square) -> Bitboard {
        self.valid_moves[sq as usize]
    }

    pub fn is_valid_move(&self, from: Square, to: Square) -> bool {
        is_bit_set(self.valid_moves[from as usize], to)
    }

    pub fn get_checkers(&self, color: Color) -> Bitboard {
        if color == Color::Both {
            return self.checkers[Color::White as usize] | self.checkers[Color::Black as usize];
        }
        self.checkers[color as usize]
    }

    pub fn set_checkers(&mut self, color: Color, sq: Square) {
        if color != Color::Both {
            self.checkers[color as usize] = set_bit(self.checkers[color as usize], sq);
        }
    }

    pub fn unset_checkers(&mut self, color: Color, sq: Square) {
        if color != Color::Both {
            self.checkers[color as usize] = clear_bit(self.checkers[color as usize], sq);
        }
    }

    pub fn is_checkers_sq_set(&self, color: Color, sq: Square) -> bool {
        if color == Color::Both {
            return is_bit_set(self.get_checkers(color), sq);
        }
        is_bit_set(self.checkers[color as usize], sq)
    }

    pub fn set_attacks(&mut self, color: Color, attacks: Bitboard) {
        if color != Color::Both {
            self.attacks[color as usize] = attacks;
        }
    }
}

#[allow(dead_code)]
pub fn print_bitboard(bb: Bitboard) {
    for rank in Rank::iter_reverse() {
        print!("{}: ", rank);
        for file in File::iter() {
            let sq = Square::make_square(file, rank);
            if is_bit_set(bb, sq) {
                print!("1 ");
            } else {
                print!("0 ");
            }
        }
        println!();
    }
    println!("   A B C D E F G H");
}

pub fn set_bit(bb: Bitboard, sq: Square) -> Bitboard {
    if sq != Square::Count {
        bb | (1u64 << sq as u64)
    } else {
        bb
    }
}

pub fn clear_bit(bb: Bitboard, sq: Square) -> Bitboard {
    if sq != Square::Count {
        bb & !(1u64 << sq as u64)
    } else {
        bb
    }
}

pub fn is_bit_set(bb: Bitboard, sq: Square) -> bool {
    if sq != Square::Count {
        bb & (1u64 << sq as u64) != 0
    } else {
        false
    }
}
