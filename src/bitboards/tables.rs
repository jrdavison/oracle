use super::magics::MagicHashTable;
use super::storage;
use super::{is_bit_set, set_bit, Bitboard};
use crate::utils::{Color, File, Rank, Square};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

pub type AttackMaskTable = [Bitboard; Square::Count as usize];
pub type BlockersTable = [MagicHashTable; Square::Count as usize];

const HORIZONTAL_MASK: Bitboard = 0xFF;
const VERTICAL_MASK: Bitboard = 0x0101010101010101;
const KNIGHT_DIRECTIONS: [(i8, i8); 8] = [(2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2)];
const KING_DIRECTIONS: [(i8, i8); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];
const WHITE_PAWN_ATTACKS: [(i8, i8); 2] = [(1, 1), (1, -1)];
const BLACK_PAWN_ATTACKS: [(i8, i8); 2] = [(-1, 1), (-1, -1)];

pub struct LookupTables {
    // masks
    diagonal_masks: Lazy<AttackMaskTable>,
    orthogonal_masks: Lazy<AttackMaskTable>,

    king_masks: Lazy<AttackMaskTable>,
    knight_masks: Lazy<AttackMaskTable>,
    pawn_attack_masks: [Lazy<AttackMaskTable>; Color::Both as usize],

    // blocker tables
    bishop_blockers_lookup: Lazy<BlockersTable>,
    rook_blockers_lookup: Lazy<BlockersTable>,
}

impl LookupTables {
    pub const fn init() -> LookupTables {
        LookupTables {
            diagonal_masks: Lazy::new(|| storage::load_attack_masks_bin("diagonal_masks.bin")),
            orthogonal_masks: Lazy::new(|| storage::load_attack_masks_bin("orthogonal_masks.bin")),

            king_masks: Lazy::new(|| storage::load_attack_masks_bin("king_masks.bin")),
            knight_masks: Lazy::new(|| storage::load_attack_masks_bin("knight_masks.bin")),
            pawn_attack_masks: [
                Lazy::new(|| storage::load_attack_masks_bin("white_pawn_attack_masks.bin")),
                Lazy::new(|| storage::load_attack_masks_bin("black_pawn_attack_masks.bin")),
            ],

            rook_blockers_lookup: Lazy::new(|| storage::load_magic_hash_table_bin("rook_blockers_table.bin")),
            bishop_blockers_lookup: Lazy::new(|| storage::load_magic_hash_table_bin("bishop_blockers_table.bin")),
        }
    }

    pub fn get_diagonal_mask(&self, sq: Square) -> Bitboard {
        self.diagonal_masks[sq as usize]
    }

    pub fn get_orthogonal_mask(&self, sq: Square) -> Bitboard {
        self.orthogonal_masks[sq as usize]
    }

    pub fn get_king_mask(&self, sq: Square) -> Bitboard {
        self.king_masks[sq as usize]
    }

    pub fn get_knight_mask(&self, sq: Square) -> Bitboard {
        self.knight_masks[sq as usize]
    }

    pub fn get_pawn_attack_mask(&self, color: Color, sq: Square) -> Bitboard {
        if color == Color::Both {
            return 0;
        }
        self.pawn_attack_masks[color as usize][sq as usize]
    }

    pub fn get_bishop_mask(&self, sq: Square, blocker_key: Bitboard) -> Bitboard {
        self.bishop_blockers_lookup[sq as usize].get(blocker_key)
    }

    pub fn get_rook_mask(&self, sq: Square, blocker_key: Bitboard) -> Bitboard {
        self.rook_blockers_lookup[sq as usize].get(blocker_key)
    }
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

fn relevant_blockers(mask: Bitboard) -> impl Iterator<Item = Bitboard> {
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
        attack_mask = set_bit(attack_mask, dest_sq);
        if is_bit_set(blockers, dest_sq) {
            break;
        }
        east_file = east_file + 1u8;
    }
    // W
    let mut west_file = file - 1u8;
    while west_file != File::Count {
        let dest_sq = Square::make_square(west_file, rank);
        attack_mask = set_bit(attack_mask, dest_sq);
        if is_bit_set(blockers, dest_sq) {
            break;
        }
        west_file = west_file - 1;
    }
    // N
    let mut north_rank = rank + 1u8;
    while north_rank != Rank::Count {
        let dest_sq = Square::make_square(file, north_rank);
        attack_mask = set_bit(attack_mask, dest_sq);
        if is_bit_set(blockers, dest_sq) {
            break;
        }
        north_rank = north_rank + 1u8;
    }
    // S
    let mut south_rank = rank - 1u8;
    while south_rank != Rank::Count {
        let dest_sq = Square::make_square(file, south_rank);
        attack_mask = set_bit(attack_mask, dest_sq);
        if is_bit_set(blockers, dest_sq) {
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
        attack_mask = set_bit(attack_mask, dest_sq);
        if is_bit_set(blockers, dest_sq) {
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
        attack_mask = set_bit(attack_mask, dest_sq);
        if is_bit_set(blockers, dest_sq) {
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
        attack_mask = set_bit(attack_mask, dest_sq);
        if is_bit_set(blockers, dest_sq) {
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
        attack_mask = set_bit(attack_mask, dest_sq);
        if is_bit_set(blockers, dest_sq) {
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

fn jumping_attacks(sq: Square, directions: &[(i8, i8)]) -> Bitboard {
    let mut attacks = 0;
    let file = Square::file_of(sq);
    let rank = Square::rank_of(sq);

    for &(dr, df) in directions {
        let dest_rank = rank + dr;
        let dest_file = file + df;
        if dest_file != File::Count && dest_rank != Rank::Count {
            let dest_sq = Square::make_square(dest_file, dest_rank);
            attacks = set_bit(attacks, dest_sq);
        }
    }
    attacks
}

fn generate_sliding_attack_tables<F>(compute_attacks: F) -> (AttackMaskTable, BlockersTable)
where
    F: Fn(Square, Bitboard, bool) -> Bitboard,
{
    let mut attack_masks = [Bitboard::default(); Square::Count as usize];
    let mut magics = std::array::from_fn(|_| MagicHashTable::default());

    for sq in Square::iter() {
        let start = Instant::now();
        let mask = compute_attacks(sq, 0, true);
        attack_masks[sq as usize] = mask;

        let mut moves = HashMap::new();
        for blockers in relevant_blockers(mask) {
            let attacks = compute_attacks(sq, blockers, false);
            moves.insert(blockers, attacks);
        }
        println!(
            "Precomputed {} moves for square {:?} in {:?}",
            moves.len(),
            sq,
            start.elapsed()
        );
        magics[sq as usize] = MagicHashTable::compute_magic_number(moves);
    }

    (attack_masks, magics)
}

fn generate_jumping_attack_tables(directions: &[(i8, i8)]) -> AttackMaskTable {
    let mut attack_lookup = [Bitboard::default(); Square::Count as usize];
    for sq in Square::iter() {
        attack_lookup[sq as usize] = jumping_attacks(sq, directions);
    }
    attack_lookup
}

pub fn compute() -> Result<(), Box<dyn Error>> {
    println!("Precomputing moves...\n\n");

    println!("Generating knight masks...");
    let knight_attacks = generate_jumping_attack_tables(&KNIGHT_DIRECTIONS);
    storage::save_attack_masks_bin("knight_masks.bin", &knight_attacks);
    println!("Saved knight masks\n\n");

    println!("Generating king masks...");
    let king_attacks = generate_jumping_attack_tables(&KING_DIRECTIONS);
    storage::save_attack_masks_bin("king_masks.bin", &king_attacks);
    println!("Saved king attacks\n\n");

    println!("Generating pawn attack masks...");
    let w_pawn_attacks = generate_jumping_attack_tables(&WHITE_PAWN_ATTACKS);
    storage::save_attack_masks_bin("white_pawn_attack_masks.bin", &w_pawn_attacks);
    let b_pawn_attacks = generate_jumping_attack_tables(&BLACK_PAWN_ATTACKS);
    storage::save_attack_masks_bin("black_pawn_attack_masks.bin", &b_pawn_attacks);
    println!("Saved pawn attacks\n\n");

    println!("Generating rook tables...");
    let rook_tables = generate_sliding_attack_tables(rook_attacks);
    storage::save_attack_masks_bin("orthogonal_masks.bin", &rook_tables.0);
    storage::save_magic_hash_table_bin("rook_blockers_table.bin", &rook_tables.1);
    println!("Saved rook tables\n\n");

    println!("Generating bishop tables...");
    let bishop_tables = generate_sliding_attack_tables(bishop_attacks);
    storage::save_attack_masks_bin("diagonal_masks.bin", &bishop_tables.0);
    storage::save_magic_hash_table_bin("bishop_blockers_table.bin", &bishop_tables.1);
    println!("Saved bishop tables");

    Ok(())
}

pub fn force_load(table: &LookupTables) {
    Lazy::force(&table.diagonal_masks);
    Lazy::force(&table.orthogonal_masks);
    Lazy::force(&table.king_masks);
    Lazy::force(&table.knight_masks);
    Lazy::force(&table.pawn_attack_masks[Color::White as usize]);
    Lazy::force(&table.pawn_attack_masks[Color::Black as usize]);
    Lazy::force(&table.rook_blockers_lookup);
    Lazy::force(&table.bishop_blockers_lookup);
}
