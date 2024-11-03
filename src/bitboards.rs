use crate::utils::types::{Bitboard, Color, File, Rank, Square};

pub struct Bitboards {
    valid_moves: [Bitboard; Square::Count as usize],
    checkers: [Bitboard; Color::Both as usize],
    // attacks: [Bitboard; Color::ColorNb as usize],
}

impl Default for Bitboards {
    fn default() -> Self {
        Bitboards {
            valid_moves: [0; Square::Count as usize],
            checkers: [0; Color::Both as usize],
            // attacks: [0; Color::ColorNb as usize],
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

    pub fn get_checkers(&self, color: Color) -> Bitboard {
        if color == Color::Both {
            return self.checkers[Color::White as usize] | self.checkers[Color::Black as usize];
        }
        self.checkers[color as usize]
    }

    pub fn set_checkers(&mut self, color: Color, sq: Square) {
        assert!(color != Color::Both, "Invalid color");
        set_bit(&mut self.checkers[color as usize], sq);
    }

    pub fn unset_checkers(&mut self, color: Color, sq: Square) {
        clear_bit(&mut self.checkers[color as usize], sq);
    }

    pub fn is_checkers_set(&self, color: Color, sq: Square) -> bool {
        if color == Color::Both {
            return is_bit_set(self.get_checkers(color), sq);
        }
        is_bit_set(self.checkers[color as usize], sq)
    }
}

#[allow(dead_code)]
pub fn print_bitboard(bitboard: Bitboard) {
    for rank in Rank::iter_reverse() {
        for file in File::iter() {
            let sq = Square::make_square(file, rank);
            if is_bit_set(bitboard, sq) {
                print!("1 ");
            } else {
                print!("0 ");
            }
        }
        println!();
    }
}

pub fn set_bit(bitboard: &mut Bitboard, sq: Square) {
    *bitboard |= 1u64 << sq as u64;
}

pub fn clear_bit(bitboard: &mut Bitboard, sq: Square) {
    *bitboard &= !(1u64 << sq as u64);
}

pub fn is_bit_set(bitboard: Bitboard, sq: Square) -> bool {
    if sq == Square::Count {
        return false;
    }
    bitboard & (1u64 << sq as u64) != 0
}
