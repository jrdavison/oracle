use crate::bitboards::Bitboard;
use crate::magic_bitboards::{AttackMaskTable, BlockersTable};
use crate::utils::Square;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::{Cursor, Read};
use std::path::Path;

const SAVE_PATH: &str = "./data/";
static DATA_DIR: Dir = include_dir!("data/");

pub fn load_blockers_lookup_bin(path: &str) -> BlockersTable {
    let file = DATA_DIR.get_file(path).expect("Failed to get file");
    let mut reader = Cursor::new(file.contents());

    let mut move_database: BlockersTable = std::array::from_fn(|_| HashMap::new());
    for sq in Square::iter() {
        let mut moves: HashMap<Bitboard, Bitboard> = HashMap::new();

        let mut num_entries_buf = [0u8; 4];
        reader.read_exact(&mut num_entries_buf).unwrap();
        let num_entries = u32::from_le_bytes(num_entries_buf);
        for _ in 0..num_entries {
            let mut blockers_buf = [0u8; 8];
            let mut attacks_buf = [0u8; 8];

            reader.read_exact(&mut blockers_buf).unwrap();
            reader.read_exact(&mut attacks_buf).unwrap();

            let blockers = u64::from_le_bytes(blockers_buf);
            let attacks = u64::from_le_bytes(attacks_buf);
            moves.insert(blockers, attacks);
        }
        move_database[sq as usize] = moves;
    }

    move_database
}

pub fn save_blockers_table_bin(filename: &str, blockers_table: &BlockersTable) {
    let full_path = Path::new(SAVE_PATH).join(filename);
    let mut file = File::create(full_path).expect("Failed to create blockers_db file");

    for square_data in blockers_table.iter() {
        let num_entries = square_data.len() as u32;
        file.write_all(&num_entries.to_le_bytes()).unwrap();

        for (&blockers, &attacks) in square_data {
            file.write_all(&blockers.to_le_bytes()).unwrap();
            file.write_all(&attacks.to_le_bytes()).unwrap();
        }
    }
}

pub fn save_attack_masks_bin(filename: &str, attack_db: &AttackMaskTable) {
    let path = Path::new(SAVE_PATH).join(filename);
    let mut file = File::create(path).expect("Failed to create attack_db file");

    for &value in attack_db {
        file.write_all(&value.to_le_bytes()).unwrap();
    }
}

pub fn load_attack_masks_bin(path: &str) -> AttackMaskTable {
    let file = DATA_DIR.get_file(path).expect("Failed to get file");
    let data = file.contents();

    assert_eq!(data.len(), (Square::Count as usize) * 8, "Invalid data length!");

    let mut attack_moves = [Bitboard::default(); Square::Count as usize];
    for (i, bb) in attack_moves.iter_mut().enumerate() {
        let start = i * 8;
        let end = start + 8;
        *bb = u64::from_le_bytes(data[start..end].try_into().unwrap());
    }

    attack_moves
}
