use crate::bitboards::{self, Bitboard};
use crate::magic_bitboards::storage::save_blockers_db;
use crate::storage::save_attack_db;
use crate::utils::{File, Rank, Square};
use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

const HORIZONTAL_MASK: Bitboard = 0xFF;
const VERTICAL_MASK: Bitboard = 0x0101010101010101;

const KNIGHT_DIRECTIONS: [(i8, i8); 8] = [(2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2)];
const KING_DIRECTIONS: [(i8, i8); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

struct MaskBlockerDbs {
    masks: [Bitboard; Square::Count as usize],
    blockers: [HashMap<Bitboard, Bitboard>; Square::Count as usize],
}

pub fn remove_edge_bits(mask: &mut Bitboard, sq: Square) {
    let file = Square::file_of(sq);
    let rank = Square::rank_of(sq);

    if rank != Rank::Rank1 {
        *mask &= !HORIZONTAL_MASK;
    }
    if rank != Rank::Rank8 {
        *mask &= !(HORIZONTAL_MASK << 56);
    }
    if file != File::FileA {
        *mask &= !VERTICAL_MASK;
    }
    if file != File::FileH {
        *mask &= !(VERTICAL_MASK << 7);
    }
}

fn generate_relevant_blockers(mask: Bitboard) -> impl Iterator<Item = Bitboard> {
    let relevant_bits: Vec<usize> = (0..64).filter(|&i| mask & (1 << i) != 0).collect();
    let num_relevant_bits = relevant_bits.len();

    (0..(1 << num_relevant_bits)).map(move |index| {
        let mut blockers = 0u64;
        for (i, &bit) in relevant_bits.iter().enumerate() {
            if index & (1 << i) != 0 {
                blockers |= 1 << bit;
            }
        }
        blockers
    })
}

fn mask_rook_attacks(sq: Square) -> Bitboard {
    let v_mask = VERTICAL_MASK << Square::file_of(sq) as u8;
    let h_mask = HORIZONTAL_MASK << (Square::rank_of(sq) * 8) as u8;
    let mut attack_mask = h_mask | v_mask;

    remove_edge_bits(&mut attack_mask, sq);
    bitboards::clear_bit(&mut attack_mask, sq);
    attack_mask
}

fn rook_attacks(sq: Square, blockers: Bitboard, remove_edges: bool) -> Bitboard {
    let mut attack_mask = 0;
    let file = Square::file_of(sq);
    let rank = Square::rank_of(sq);

    // E
    let mut east_file = file + 1u8;
    while east_file != File::Count {
        let dest_sq = Square::make_square(east_file, rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(blockers, dest_sq) {
            break;
        }
        east_file = east_file + 1u8;
    }
    // W
    let mut west_file = file - 1u8;
    while west_file != File::Count {
        let dest_sq = Square::make_square(west_file, rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(blockers, dest_sq) {
            break;
        }
        west_file = west_file - 1;
    }
    // N
    let mut north_rank = rank + 1u8;
    while north_rank != Rank::Count {
        let dest_sq = Square::make_square(file, north_rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(blockers, dest_sq) {
            break;
        }
        north_rank = north_rank + 1u8;
    }
    // S
    let mut south_rank = rank - 1u8;
    while south_rank != Rank::Count {
        let dest_sq = Square::make_square(file, south_rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(blockers, dest_sq) {
            break;
        }
        south_rank = south_rank - 1u8;
    }

    if remove_edges {
        remove_edge_bits(&mut attack_mask, sq);
    }

    attack_mask
}

fn generate_rook_attack_db() -> MaskBlockerDbs {
    println!("Generating rook move database...");

    let mut rook_moves: [HashMap<Bitboard, Bitboard>; Square::Count as usize] = std::array::from_fn(|_| HashMap::new());
    let mut h_v_masks = [Bitboard::default(); Square::Count as usize];

    for sq in Square::iter() {
        let start = Instant::now();
        let mask = mask_rook_attacks(sq);
        h_v_masks[sq as usize] = mask;
        for blockers in generate_relevant_blockers(mask) {
            let attacks = rook_attacks(sq, blockers, false);
            rook_moves[sq as usize].insert(blockers, attacks);
        }
        println!(
            "Computed {} moves for square {:?}. Done in {:.2} seconds.",
            rook_moves[sq as usize].len(),
            sq,
            start.elapsed().as_secs_f64()
        );
    }

    MaskBlockerDbs {
        masks: h_v_masks,
        blockers: rook_moves,
    }
}

fn jumping_attacks(sq: Square, directions: &[(i8, i8)]) -> Bitboard {
    let mut attacks = 0;
    let file = Square::file_of(sq);
    let rank = Square::rank_of(sq);

    for &(dr, df) in directions {
        let dest_rank = rank + dr;
        let dest_file = file + df;
        if dest_file != File::Count && dest_rank != Rank::Count {
            let dest_sq = Square::make_square(dest_file, dest_rank);
            bitboards::set_bit(&mut attacks, dest_sq);
        }
    }
    attacks
}

fn generate_jumping_attacks_db(directions: &[(i8, i8)]) -> [Bitboard; 64] {
    let mut attack_lookup = [Bitboard::default(); Square::Count as usize];
    for sq in Square::iter() {
        attack_lookup[sq as usize] = jumping_attacks(sq, directions);
    }
    attack_lookup
}

pub fn generate() -> Result<(), Box<dyn Error>> {
    println!("Generating magic bitboards...");

    println!("Generating Knight attacks...");
    let knight_attacks = generate_jumping_attacks_db(&KNIGHT_DIRECTIONS);
    save_attack_db("knight_attacks.bin", &knight_attacks);
    println!("Saved knight attacks.");

    println!("Generating King attacks...");
    let king_attacks = generate_jumping_attacks_db(&KING_DIRECTIONS);
    save_attack_db("king_attacks.bin", &king_attacks);
    println!("Saved king attacks.");

    println!("Generating Rook attacks...");
    let rook_attacks = generate_rook_attack_db();
    save_attack_db("rook_masks.bin", &rook_attacks.masks);
    save_blockers_db("rook_attacks.bin", &rook_attacks.blockers);
    println!("Saved rook attacks.");

    Ok(())
}
