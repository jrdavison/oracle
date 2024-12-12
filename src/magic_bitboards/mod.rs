mod compute;
mod storage;

use crate::bitboards::Bitboard;
use crate::magic_bitboards::compute::{
    generate_bishop_attack_tables, generate_jumping_attacks_db, generate_rook_attack_tables, BLACK_PAWN_ATTACKS,
    KING_DIRECTIONS, KNIGHT_DIRECTIONS, WHITE_PAWN_ATTACKS,
};
use crate::magic_bitboards::storage::{
    load_attack_masks_bin, load_blockers_lookup_bin, save_attack_masks_bin, save_blockers_table_bin,
};
use crate::utils::{Color, Square};
use once_cell::sync::Lazy;
use ph::fmph;
use std::collections::HashMap;
use std::error::Error;

pub type AttackMaskTable = [Bitboard; Square::Count as usize];
pub type BlockersTable = [HashMap<Bitboard, Bitboard>; Square::Count as usize];
pub type PerfectBlockersTable = [PerfectHashTable; Square::Count as usize];

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

pub struct PerfectHashTable {
    hasher: fmph::Function,
    table: Vec<Bitboard>,
}

impl PerfectHashTable {
    pub fn get(&self, key: &Bitboard) -> Bitboard {
        // if let Some(index) = self.hasher.get(key) {
            let start= std::time::Instant::now();
            let index = self.hasher.get(key).unwrap();
            println!("Time taken: {:?}", start.elapsed());
            let test = self.table[index as usize];
            test
        // } else {
            // Bitboard::default()
        // }
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
    pub rook_blockers_lookup: Lazy<PerfectBlockersTable>,
    pub bishop_blockers_lookup: Lazy<PerfectBlockersTable>,
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

fn generate_perfect_blockers_table(path: &str) -> PerfectBlockersTable {
    let blockers = load_blockers_lookup_bin(path);
    let mut perfect_blockers: PerfectBlockersTable = std::array::from_fn(|_| PerfectHashTable {
        hasher: fmph::Function::from(vec![0]),
        table: Vec::new(),
    });

    for (sq, blockers_map) in blockers.iter().enumerate() {
        let keys = blockers_map.keys().cloned().collect::<Vec<Bitboard>>();
        let hasher = fmph::Function::from(keys);
        let mut table = Vec::new();

        for (blockers, attacks) in blockers_map {
            let hash_index = hasher.get(&blockers).unwrap() as usize;
            if hash_index >= table.len() {
                table.resize_with(hash_index + 1, || 0);
            }
            table[hash_index as usize] = *attacks;
        }

        perfect_blockers[sq] = PerfectHashTable { hasher, table };
    }

    println!();

    // TODO: move this to storage?
    perfect_blockers
}
