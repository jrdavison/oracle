pub mod generate;
pub mod storage;

use crate::bitboards::Bitboard;
use crate::magic_bitboards::generate::{
    generate_bishop_attack_dbs, generate_jumping_attacks_db, generate_rook_attack_db, KING_DIRECTIONS,
    KNIGHT_DIRECTIONS,
};
use crate::magic_bitboards::storage::{save_attack_masks_bin, save_blockers_db};
use crate::utils::Square;
use std::collections::HashMap;
use std::error::Error;

pub type BlockersDatabase = [HashMap<Bitboard, Bitboard>; Square::Count as usize]; // TODO: better name
pub type AttackMaskTable = [Bitboard; Square::Count as usize];

pub fn generate() -> Result<(), Box<dyn Error>> {
    println!("Generating magic bitboards...");

    println!();
    println!("Generating Knight attacks...");
    let knight_attacks = generate_jumping_attacks_db(&KNIGHT_DIRECTIONS);
    save_attack_masks_bin("knight_attacks.bin", &knight_attacks);
    println!("Saved knight attacks.");

    println!();
    println!("Generating King attacks...");
    let king_attacks = generate_jumping_attacks_db(&KING_DIRECTIONS);
    save_attack_masks_bin("king_attacks.bin", &king_attacks);
    println!("Saved king attacks.");

    println!();
    println!("Generating Rook attacks...");
    let rook_attacks = generate_rook_attack_db();
    save_attack_masks_bin("rook_masks.bin", &rook_attacks.masks);
    save_blockers_db("rook_attacks.bin", &rook_attacks.blockers);
    println!("Saved rook attacks.");

    println!();
    println!("Generating Bishop attacks...");
    let bishop_attacks = generate_bishop_attack_dbs();
    save_attack_masks_bin("bishop_masks.bin", &bishop_attacks.masks);
    save_blockers_db("bishop_attacks.bin", &bishop_attacks.blockers);
    println!("Saved bishop attacks.");

    Ok(())
}
