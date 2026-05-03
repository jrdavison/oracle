use oracle::position::{Position, count_valid_moves};

// https://www.chessprogramming.org/Perft
#[test]
pub fn test_example() {
    let mut pos = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert_eq!(count_valid_moves(&mut pos, 1), 20);
    assert_eq!(count_valid_moves(&mut pos, 2), 400);
    assert_eq!(count_valid_moves(&mut pos, 3), 8902);
}

// #[test]
// pub fn test_kiwipete() {
//     let mut pos = Position::new("r3k2r/p1ppqpb1/bn2pnp1/2pP4/1p2P3/2N2N2/PPQ1BPPP/R1B1K2R w KQkq - 0 1");
//     assert_eq!(count_valid_moves(&mut pos, 1), 48);
//     assert_eq!(count_valid_moves(&mut pos, 2), 2039);
//     assert_eq!(count_valid_moves(&mut pos, 3), 97862);
// }

// #[test]
// pub fn test_position_3() {
//     let mut pos = Position::new("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
//     assert_eq!(count_valid_moves(&mut pos, 1), 44);
//     assert_eq!(count_valid_moves(&mut pos, 2), 1486);
//     assert_eq!(count_valid_moves(&mut pos, 3), 62379);
// }
