use crate::bitboards::{self, Bitboard};
use crate::utils::{File, Rank, Square};
use std::collections::HashMap;
use std::time::Instant;

const HORIZONTAL_MASK: Bitboard = 0xFF;
const VERTICAL_MASK: Bitboard = 0x0101010101010101;

pub const KNIGHT_DIRECTIONS: [(i8, i8); 8] = [(2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2)];
pub const KING_DIRECTIONS: [(i8, i8); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];
pub const WHITE_PAWN_ATTACKS: [(i8, i8); 2] = [(1, 1), (1, -1)];
pub const BLACK_PAWN_ATTACKS: [(i8, i8); 2] = [(-1, 1), (-1, -1)];

pub struct BitboardLookupTables {
    pub masks: [Bitboard; Square::Count as usize],
    pub blockers: [HashMap<Bitboard, Bitboard>; Square::Count as usize], // TODO: i think this can be a type
}

fn remove_edge_bits(mask: &mut Bitboard, sq: Square) {
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

fn rook_attacks(sq: Square, blockers: Bitboard, remove_edges: bool) -> Bitboard {
    let mut attack_mask = 0;
    let file = Square::file_of(sq);
    let rank = Square::rank_of(sq);

    // E
    let mut east_file = file + 1u8;
    while east_file != File::Count {
        let dest_sq = Square::make_square(east_file, rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(&blockers, dest_sq) {
            break;
        }
        east_file = east_file + 1u8;
    }
    // W
    let mut west_file = file - 1u8;
    while west_file != File::Count {
        let dest_sq = Square::make_square(west_file, rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(&blockers, dest_sq) {
            break;
        }
        west_file = west_file - 1;
    }
    // N
    let mut north_rank = rank + 1u8;
    while north_rank != Rank::Count {
        let dest_sq = Square::make_square(file, north_rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(&blockers, dest_sq) {
            break;
        }
        north_rank = north_rank + 1u8;
    }
    // S
    let mut south_rank = rank - 1u8;
    while south_rank != Rank::Count {
        let dest_sq = Square::make_square(file, south_rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(&blockers, dest_sq) {
            break;
        }
        south_rank = south_rank - 1u8;
    }

    if remove_edges {
        remove_edge_bits(&mut attack_mask, sq);
    }

    attack_mask
}

fn bishop_attacks(sq: Square, blockers: Bitboard, remove_edges: bool) -> Bitboard {
    let mut attack_mask = 0;
    let file = Square::file_of(sq);
    let rank = Square::rank_of(sq);

    // NE
    let mut ne_file = file + 1u8;
    let mut ne_rank = rank + 1u8;
    while ne_file != File::Count && ne_rank != Rank::Count {
        let dest_sq = Square::make_square(ne_file, ne_rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(&blockers, dest_sq) {
            break;
        }
        ne_file = ne_file + 1u8;
        ne_rank = ne_rank + 1u8;
    }
    // SE
    let mut se_file = file + 1u8;
    let mut se_rank = rank - 1u8;
    while se_file != File::Count && se_rank != Rank::Count {
        let dest_sq = Square::make_square(se_file, se_rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(&blockers, dest_sq) {
            break;
        }
        se_file = se_file + 1u8;
        se_rank = se_rank - 1u8;
    }
    // SW
    let mut sw_file = file - 1u8;
    let mut sw_rank = rank - 1u8;
    while sw_file != File::Count && sw_rank != Rank::Count {
        let dest_sq = Square::make_square(sw_file, sw_rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(&blockers, dest_sq) {
            break;
        }
        sw_file = sw_file - 1u8;
        sw_rank = sw_rank - 1u8;
    }
    // NW
    let mut nw_file = file - 1u8;
    let mut nw_rank = rank + 1u8;
    while nw_file != File::Count && nw_rank != Rank::Count {
        let dest_sq = Square::make_square(nw_file, nw_rank);
        bitboards::set_bit(&mut attack_mask, dest_sq);
        if bitboards::is_bit_set(&blockers, dest_sq) {
            break;
        }
        nw_file = nw_file - 1u8;
        nw_rank = nw_rank + 1u8;
    }

    if remove_edges {
        remove_edge_bits(&mut attack_mask, sq);
    }

    attack_mask
}

pub fn generate_rook_attack_tables() -> BitboardLookupTables {
    let mut rook_moves: [HashMap<Bitboard, Bitboard>; Square::Count as usize] = std::array::from_fn(|_| HashMap::new());
    let mut orthog_masks = [Bitboard::default(); Square::Count as usize];

    for sq in Square::iter() {
        let start = Instant::now();
        let mask = rook_attacks(sq, 0, true);
        orthog_masks[sq as usize] = mask;
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

    BitboardLookupTables {
        masks: orthog_masks,
        blockers: rook_moves,
    }
}

pub fn generate_bishop_attack_tables() -> BitboardLookupTables {
    let mut bishop_moves: [HashMap<Bitboard, Bitboard>; Square::Count as usize] =
        std::array::from_fn(|_| HashMap::new());
    let mut diagonal_masks = [Bitboard::default(); Square::Count as usize];

    for sq in Square::iter() {
        let start = Instant::now();
        let mask = bishop_attacks(sq, 0, true);
        diagonal_masks[sq as usize] = mask;
        for blockers in generate_relevant_blockers(mask) {
            let attacks = bishop_attacks(sq, blockers, false);
            bishop_moves[sq as usize].insert(blockers, attacks);
        }
        println!(
            "Computed {} moves for square {:?}. Done in {:.2} seconds.",
            bishop_moves[sq as usize].len(),
            sq,
            start.elapsed().as_secs_f64()
        );
    }

    BitboardLookupTables {
        masks: diagonal_masks,
        blockers: bishop_moves,
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

pub fn generate_jumping_attacks_db(directions: &[(i8, i8)]) -> [Bitboard; 64] {
    let mut attack_lookup = [Bitboard::default(); Square::Count as usize];
    for sq in Square::iter() {
        attack_lookup[sq as usize] = jumping_attacks(sq, directions);
    }
    attack_lookup
}
