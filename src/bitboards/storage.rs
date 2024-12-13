use super::magics::{AttackMaskTable, MagicBlockersTable, MagicHashTable};
use super::Bitboard;
use crate::utils::Square;
use include_dir::{include_dir, Dir};
use std::fs::File;
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::Path;

const SAVE_PATH: &str = "./data/";
static DATA_DIR: Dir = include_dir!("data/");

pub fn load_magic_hash_table_bin(path: &str) -> MagicBlockersTable {
    let file = DATA_DIR.get_file(path).expect("Failed to get file");
    let mut reader = Cursor::new(file.contents());

    let mut magic_tables: MagicBlockersTable = std::array::from_fn(|_| MagicHashTable::default());

    for sq in Square::iter() {
        let mut vec_len_buf = [0u8; 4];
        reader.read_exact(&mut vec_len_buf).unwrap();
        let vec_len = u32::from_le_bytes(vec_len_buf) as usize;

        let mut table = Vec::with_capacity(vec_len);
        for _ in 0..vec_len {
            let mut bitboard_buf = [0u8; 8];
            reader.read_exact(&mut bitboard_buf).unwrap();
            let bitboard = u64::from_le_bytes(bitboard_buf);
            table.push(bitboard);
        }

        let mut shift_buf = [0u8; 8];
        reader.read_exact(&mut shift_buf).unwrap();
        let shift = usize::from_le_bytes(shift_buf);

        let mut magic_buf = [0u8; 8];
        reader.read_exact(&mut magic_buf).unwrap();
        let magic = usize::from_le_bytes(magic_buf);

        magic_tables[sq as usize] = MagicHashTable { table, shift, magic };
    }

    magic_tables
}

pub fn save_magic_hash_table_bin(filename: &str, tables: &MagicBlockersTable) {
    let full_path = Path::new(SAVE_PATH).join(filename);
    let file = File::create(full_path).expect("Failed to create magic hash table file");
    let mut writer = BufWriter::new(file);

    for table in tables.iter() {
        let vec_len = table.table.len() as u32;
        writer.write_all(&vec_len.to_le_bytes()).unwrap();

        for &bitboard in &table.table {
            writer.write_all(&bitboard.to_le_bytes()).unwrap();
        }
        writer.write_all(&table.shift.to_le_bytes()).unwrap();
        writer.write_all(&table.magic.to_le_bytes()).unwrap();
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
