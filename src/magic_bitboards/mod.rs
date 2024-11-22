use crate::bitboards::Bitboard;
use crate::utils::Square;
use std::collections::HashMap;

pub mod generate;
pub mod storage;

pub type BlockersDatabase = [HashMap<Bitboard, Bitboard>; Square::Count as usize];
pub type AttackDatabase = [Bitboard; Square::Count as usize];
