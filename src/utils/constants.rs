use include_dir::{include_dir, Dir};

pub static DATA_DIR: Dir = include_dir!("data/");

pub const HORIZONTAL_MASK: u64 = 0x00000000000000FF;
pub const VERTICAL_MASK: u64 = 0x0101010101010101;
