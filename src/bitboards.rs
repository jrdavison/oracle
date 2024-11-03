use crate::utils::constants;
use crate::utils::types::{Bitboard, BlockersMoveDatabase, File, Rank, SimpleMoveDatabase, Square};
use std::collections::HashMap;
use std::io::{Cursor, Read};

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

pub fn load_simple_move_db(path: &str) -> SimpleMoveDatabase {
    let file = constants::DATA_DIR.get_file(path).expect("Failed to get file");
    let data = file.contents();

    assert_eq!(data.len(), (Square::Count as usize) * 8, "Invalid data length!");

    let mut knight_moves = [Bitboard::default(); Square::Count as usize];
    for (i, bb) in knight_moves.iter_mut().enumerate() {
        let start = i * 8;
        let end = start + 8;
        *bb = u64::from_le_bytes(data[start..end].try_into().unwrap());
    }

    knight_moves
}

pub fn load_blockers_move_db(path: &str) -> BlockersMoveDatabase {
    let file = constants::DATA_DIR.get_file(path).expect("Failed to get file");
    let mut reader = Cursor::new(file.contents());

    let mut move_database: BlockersMoveDatabase = std::array::from_fn(|_| HashMap::new());
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
        move_database[sq as usize] = moves;
    }

    move_database
}
