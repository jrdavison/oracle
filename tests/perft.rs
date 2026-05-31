use oracle::position::{count_valid_moves, Position};
use std::time::Instant;

// https://www.chessprogramming.org/Perft
#[test]
pub fn test_basic_count_up_to_4ply() {
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let start_pos = &mut Position::new(start_fen);

    let start_1ply = Instant::now();
    let valid_1ply = count_valid_moves(start_pos, 1);
    println!("Time to count 1ply moves: {:?}", start_1ply.elapsed());
    assert_eq!(valid_1ply, 20);

    let start_2ply = Instant::now();
    let valid_2ply = count_valid_moves(start_pos, 2);
    println!("Time to count 2ply moves: {:?}", start_2ply.elapsed());
    assert_eq!(valid_2ply, 400);

    let start_3ply = Instant::now();
    let valid_3ply = count_valid_moves(start_pos, 3);
    println!("Time to count 3ply moves: {:?}", start_3ply.elapsed());
    assert_eq!(valid_3ply, 8902);

    let start_4ply = Instant::now();
    let valid_4ply = count_valid_moves(start_pos, 4);
    println!("Time to count 4ply moves: {:?}", start_4ply.elapsed());
    assert_eq!(valid_4ply, 197281);

    let start_5ply = Instant::now();
    let valid_5ply = count_valid_moves(start_pos, 5);
    println!("Time to count 5ply moves: {:?}", start_5ply.elapsed());
    assert_eq!(valid_5ply, 4865609);
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
