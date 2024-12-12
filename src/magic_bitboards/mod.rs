mod compute;
mod storage;

use crate::bitboards::Bitboard;
use crate::magic_bitboards::compute::{
    generate_bishop_attack_tables, generate_jumping_attacks_db, generate_rook_attack_tables, BLACK_PAWN_ATTACKS,
    KING_DIRECTIONS, KNIGHT_DIRECTIONS, WHITE_PAWN_ATTACKS, custom_hash
};
use crate::magic_bitboards::storage::{
    load_attack_masks_bin, load_blockers_lookup_bin, save_attack_masks_bin, save_blockers_table_bin,
};
use crate::utils::{Color, Square};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::error::Error;

pub type AttackMaskTable = [Bitboard; Square::Count as usize];
pub type BlockersTable = [HashMap<Bitboard, Bitboard>; Square::Count as usize];
pub type MagicBlockersTable = [MagicHashTable; Square::Count as usize]; // TODO: better names

const HASH_MULTIPLIER: u64 = 0x9e3779b97f4a7c15;

pub static LOOKUP_TABLES: LookupTables = LookupTables {
    diagonal_masks: Lazy::new(|| load_attack_masks_bin("diagonal_masks.bin")),
    orthogonal_masks: Lazy::new(|| load_attack_masks_bin("orthogonal_masks.bin")),

    king_masks: Lazy::new(|| load_attack_masks_bin("king_masks.bin")),
    knight_masks: Lazy::new(|| load_attack_masks_bin("knight_masks.bin")),
    pawn_attack_masks: [
        Lazy::new(|| load_attack_masks_bin("white_pawn_attack_masks.bin")),
        Lazy::new(|| load_attack_masks_bin("black_pawn_attack_masks.bin")),
    ],

    rook_blockers_lookup: Lazy::new(|| generate_perfect_blockers_table("rook_blockers_table.bin")),
    bishop_blockers_lookup: Lazy::new(|| generate_perfect_blockers_table("bishop_blockers_table.bin")),
};

#[derive(Clone, Copy, Default)]
pub struct MagicHashTable {
    pub table: Vec<Bitboard>,
    pub shift: usize,
    pub magic: usize,
}

impl MagicHashTable {
    pub fn get(&self, key: Bitboard) -> Bitboard {
        let index = custom_hash(key, self.magic, self.shift);
        self.table[index]
    }
}

pub struct LookupTables {
    // masks
    pub diagonal_masks: Lazy<AttackMaskTable>,
    pub orthogonal_masks: Lazy<AttackMaskTable>,

    pub king_masks: Lazy<AttackMaskTable>,
    pub knight_masks: Lazy<AttackMaskTable>,
    pub pawn_attack_masks: [Lazy<AttackMaskTable>; Color::Both as usize],

    // blocker tables
    pub rook_blockers_lookup: Lazy<MagicBlockersTable>,
    pub bishop_blockers_lookup: Lazy<MagicBlockersTable>,
}

impl LookupTables {
    pub fn load_all(&self) {
        Lazy::force(&self.diagonal_masks);
        Lazy::force(&self.orthogonal_masks);
        Lazy::force(&self.king_masks);
        Lazy::force(&self.knight_masks);
        Lazy::force(&self.pawn_attack_masks[Color::White as usize]);
        Lazy::force(&self.pawn_attack_masks[Color::Black as usize]);
        Lazy::force(&self.rook_blockers_lookup);
        Lazy::force(&self.bishop_blockers_lookup);
    }
}

pub fn precompute() -> Result<(), Box<dyn Error>> {
    println!("Precomputing moves...");

    println!();
    println!("Generating knight masks...");
    let knight_attacks = generate_jumping_attacks_db(&KNIGHT_DIRECTIONS);
    save_attack_masks_bin("knight_masks.bin", &knight_attacks);
    println!("Saved knight masks.");

    println!();
    println!("Generating king masks...");
    let king_attacks = generate_jumping_attacks_db(&KING_DIRECTIONS);
    save_attack_masks_bin("king_masks.bin", &king_attacks);
    println!("Saved king attacks.");

    println!();
    println!("Generating pawn attack masks..");
    let w_pawn_attacks = generate_jumping_attacks_db(&WHITE_PAWN_ATTACKS);
    save_attack_masks_bin("white_pawn_attack_masks.bin", &w_pawn_attacks);
    let b_pawn_attacks = generate_jumping_attacks_db(&BLACK_PAWN_ATTACKS);
    save_attack_masks_bin("black_pawn_attack_masks.bin", &b_pawn_attacks);
    println!("Saved pawn attacks.");

    println!();
    println!("Generating rook tables..");
    let rook_tables = generate_rook_attack_tables();
    save_attack_masks_bin("orthogonal_masks.bin", &rook_tables.masks);
    save_blockers_table_bin("rook_blockers_table.bin", &rook_tables.blockers);
    println!("Saved rook tables.");

    println!();
    println!("Generating bishop tables...");
    let bishop_tables = generate_bishop_attack_tables();
    save_attack_masks_bin("diagonal_masks.bin", &bishop_tables.masks);
    save_blockers_table_bin("bishop_blockers_table.bin", &bishop_tables.blockers);
    println!("Saved bishop tables.");

    Ok(())
}

// TODO: rename this
// fn generate_perfect_blockers_table(path: &str) -> MagicBlockersTable {
//     let blockers = load_blockers_lookup_bin(path);

//     let mut magic_blockers = std::array::from_fn(|_| MagicHashTable {
//         table: Vec::new(),
//         num_keys: 0,
//     });

//     for sq in Square::iter() {
//         let blockers_map = &blockers[sq as usize];
//         let magic_hash = &mut magic_blockers[sq as usize];
//         magic_hash.num_keys = blockers_map.len();
//         let mut collisions = 0;

//         for (blockers, attacks) in blockers_map {
//             let hash_index = magic_hash.custom_hash(*blockers);
//             if hash_index >= magic_hash.table.len() {
//                 magic_hash.table.resize_with(hash_index + 1, || 0);
//             }
//             if magic_hash.table[hash_index as usize] != 0 {
//                 collisions += 1;
//             }
//             magic_hash.table[hash_index as usize] = *attacks;
//         }

//         println!("Collisions for {:?}: {}", sq, collisions);
//     }
//     magic_blockers
// }
