pub mod generate;
pub mod storage;

use crate::bitboards::Bitboard;
use crate::magic_bitboards::generate::{
    generate_bishop_attack_tables, generate_jumping_attacks_db, generate_rook_attack_tables, KING_DIRECTIONS,
    KNIGHT_DIRECTIONS,
};
use crate::magic_bitboards::storage::{
    load_attack_masks_bin, load_blockers_lookup_bin, save_attack_masks_bin, save_blockers_table_bin,
};
use crate::utils::Square;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::error::Error;

pub type BlockersTable = [HashMap<Bitboard, Bitboard>; Square::Count as usize];
pub type AttackMaskTable = [Bitboard; Square::Count as usize];

pub static KNIGHT_MASKS: Lazy<AttackMaskTable> = Lazy::new(|| load_attack_masks_bin("knight_masks.bin"));
pub static KING_MASKS: Lazy<AttackMaskTable> = Lazy::new(|| load_attack_masks_bin("king_masks.bin"));
pub static ORTHOGONAL_MASKS: Lazy<AttackMaskTable> = Lazy::new(|| load_attack_masks_bin("orthogonal_masks.bin"));
pub static DIAGONAL_MASKS: Lazy<AttackMaskTable> = Lazy::new(|| load_attack_masks_bin("diagonal_masks.bin"));

pub static ROOK_BLOCKERS_LOOKUP: Lazy<BlockersTable> =
    Lazy::new(|| load_blockers_lookup_bin("rook_blockers_table.bin"));
pub static BISHOP_BLOCKERS_LOOKUP: Lazy<BlockersTable> =
    Lazy::new(|| load_blockers_lookup_bin("bishop_blockers_table.bin"));

pub fn load_precomputed_moves() {
    Lazy::force(&KNIGHT_MASKS);
    Lazy::force(&KING_MASKS);
    Lazy::force(&ORTHOGONAL_MASKS);
    Lazy::force(&DIAGONAL_MASKS);

    Lazy::force(&ROOK_BLOCKERS_LOOKUP);
    Lazy::force(&BISHOP_BLOCKERS_LOOKUP);
}

pub fn generate() -> Result<(), Box<dyn Error>> {
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
