// use crate::types::{File, Rank, Square};
use crate::storage::save_attack_cache;
use crate::types::{Bitboard, File, Rank, Square};
// use std::collections::HashMap;
use std::error::Error;
// use std::time::Instant;
use crate::bitboards;

// Constants
// const HORIZONTAL_MASK: u64 = 0xFF;
// const VERTICAL_MASK: u64 = 0x0101010101010101;
// const FIRST_RANK_MASK: u64 = 0xFF;
// const LAST_RANK_MASK: u64 = 0xFF00000000000000;
// const FIRST_FILE_MASK: u64 = 0x0101010101010101;
// const LAST_FILE_MASK: u64 = 0x8080808080808080;

const KNIGHT_DIRECTIONS: [(i8, i8); 8] = [(2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2)];
const KING_DIRECTIONS: [(i8, i8); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

// struct MaskBlockerDbs {
//     masks: Vec<u64>,
//     blockers: Vec<HashMap<u64, u64>>,
// }

// fn remove_edge_bits(mut mask: u64, sq: usize) -> u64 {
//     let rank = get_rank(sq);
//     let file = get_file(sq);

//     if rank != 0 {
//         mask &= !FIRST_RANK_MASK;
//     }
//     if rank != 7 {
//         mask &= !LAST_RANK_MASK;
//     }
//     if file != 0 {
//         mask &= !FIRST_FILE_MASK;
//     }
//     if file != 7 {
//         mask &= !LAST_FILE_MASK;
//     }
//     mask
// }

// fn mask_rook_attacks(sq: usize) -> u64 {
//     let h_mask = HORIZONTAL_MASK << (get_rank(sq) * 8);
//     let v_mask = VERTICAL_MASK << get_file(sq);
//     let attack_mask = h_mask | v_mask;

//     remove_edge_bits(attack_mask, sq) & !(1 << sq)
// }

// fn rook_attacks(sq: usize, blockers: u64) -> u64 {
//     let mut attacks = 0;
//     let rank = get_rank(sq) as i8;
//     let file = get_file(sq) as i8;

//     // Directions: E, W, N, S
//     for &(dr, df) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
//         let mut r = rank;
//         let mut f = file;
//         loop {
//             r += dr;
//             f += df;
//             if r < 0 || r >= 8 || f < 0 || f >= 8 {
//                 break;
//             }
//             let sq = get_square(r as u8, f as u8);
//             attacks |= 1 << sq;
//             if blockers & (1 << sq) != 0 {
//                 break;
//             }
//         }
//     }
//     attacks
// }

// fn bishop_attacks(sq: usize, blockers: u64, remove_edges: bool) -> u64 {
//     let mut attack_mask = 0;
//     let rank = get_rank(sq) as i8;
//     let file = get_file(sq) as i8;

//     // Directions: NE, SE, SW, NW
//     let directions: [(i8, i8); 4] = [(1, 1), (-1, 1), (-1, -1), (1, -1)];
//     for &(dr, df) in &directions {
//         let mut r = rank;
//         let mut f = file;
//         loop {
//             r += dr;
//             f += df;
//             if r < 0 || r >= 8 || f < 0 || f >= 8 {
//                 break;
//             }
//             let sq = get_square(r as u8, f as u8);
//             attack_mask |= 1 << sq;
//             if blockers & (1 << sq) != 0 {
//                 break;
//             }
//         }
//     }

//     if remove_edges {
//         remove_edge_bits(attack_mask, sq)
//     } else {
//         attack_mask
//     }
// }

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

fn generate_jumping_attacks_lookup(directions: &[(i8, i8)]) -> [Bitboard; 64] {
    let mut attack_lookup = [Bitboard::default(); Square::Count as usize];
    for sq in Square::iter() {
        attack_lookup[sq as usize] = jumping_attacks(sq, directions);
    }
    attack_lookup
}

// fn generate_relevant_blockers(mask: u64) -> Vec<u64> {
//     let mut relevant_bits = Vec::new();
//     for i in 0..64 {
//         if mask & (1 << i) != 0 {
//             relevant_bits.push(i);
//         }
//     }
//     let num_relevant_bits = relevant_bits.len();
//     let mut blockers_list = Vec::new();
//     for index in 0..(1 << num_relevant_bits) {
//         let mut blockers = 0;
//         for (i, &bit) in relevant_bits.iter().enumerate() {
//             if index & (1 << i) != 0 {
//                 blockers |= 1 << bit;
//             }
//         }
//         blockers_list.push(blockers);
//     }
//     blockers_list
// }

// fn generate_rook_attack_db() -> MaskBlockerDbs {
//     println!("Generating rook move database...");

//     let mut rook_moves: [HashMap<u64, u64>; 64] = [HashMap::new(); 64];
//     let mut h_v_masks = [0u64; 64];

//     for sq in 0..64 {
//         let start_time = Instant::now();

//         let mask = mask_rook_attacks(sq);
//         h_v_masks[sq] = mask;

//         let relevant_blockers = generate_relevant_blockers(mask);
//         for &blockers in &relevant_blockers {
//             let attacks = rook_attacks(sq, blockers);
//             rook_moves[sq].insert(blockers, attacks);
//         }

//         println!(
//             "Computed {} moves for square {}. Done in {:.2} seconds.",
//             rook_moves[sq].len(),
//             sq,
//             start_time.elapsed().as_secs_f64()
//         );
//     }

//     MaskBlockerDbs {
//         masks: h_v_masks,
//         blockers: rook_moves,
//     }
// }

pub fn generate() -> Result<(), Box<dyn Error>> {
    let knight_attacks = generate_jumping_attacks_lookup(&KNIGHT_DIRECTIONS);
    save_attack_cache("knight_attacks.bin", &knight_attacks);
    println!("Saved knight attacks.");
    bitboards::print_bitboard(knight_attacks[12]);

    let king_attacks = generate_jumping_attacks_lookup(&KING_DIRECTIONS);
    save_attack_cache("king_attacks.bin", &king_attacks);
    bitboards::print_bitboard(king_attacks[12]);

    // let rook_attack_dbs = generate_rook_attack_db();

    Ok(())
}
