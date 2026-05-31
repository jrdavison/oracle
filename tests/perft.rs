use oracle::moves::count_legal_moves;
use oracle::position::Position;
use std::time::Instant;

// https://www.chessprogramming.org/Perft
#[test]
pub fn test_basic_count_up_to_4ply() {
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let start_pos = &mut Position::new(start_fen);

    let start_1ply = Instant::now();
    let legal_1ply = count_legal_moves(start_pos, 1);
    println!("Time to count 1ply moves: {:?}", start_1ply.elapsed());
    assert_eq!(legal_1ply, 20);

    let start_2ply = Instant::now();
    let legal_2ply = count_legal_moves(start_pos, 2);
    println!("Time to count 2ply moves: {:?}", start_2ply.elapsed());
    assert_eq!(legal_2ply, 400);

    let start_3ply = Instant::now();
    let legal_3ply = count_legal_moves(start_pos, 3);
    println!("Time to count 3ply moves: {:?}", start_3ply.elapsed());
    assert_eq!(legal_3ply, 8902);

    let start_4ply = Instant::now();
    let legal_4ply = count_legal_moves(start_pos, 4);
    println!("Time to count 4ply moves: {:?}", start_4ply.elapsed());
    assert_eq!(legal_4ply, 197281);

    let start_5ply = Instant::now();
    let legal_5ply = count_legal_moves(start_pos, 5);
    println!("Time to count 5ply moves: {:?}", start_5ply.elapsed());
    assert_eq!(legal_5ply, 4865609);

    let start_6ply = Instant::now();
    let legal_6ply = count_legal_moves(start_pos, 6);
    println!("Time to count 6ply moves: {:?}", start_6ply.elapsed());
    assert_eq!(legal_6ply, 119060324);

    // let start_7ply = Instant::now();
    // let legal_7ply = count_legal_moves(start_pos, 7);
    // println!("Time to count 7ply moves: {:?}", start_7ply.elapsed());
    // assert_eq!(legal_7ply, 3195901860);

    // let start_8ply = Instant::now();
    // let legal_8ply = count_legal_moves(start_pos, 8);
    // println!("Time to count 8ply moves: {:?}", start_8ply.elapsed());
    // assert_eq!(legal_8ply, 84998978956);

    // let start_9ply = Instant::now();
    // let legal_9ply = count_legal_moves(start_pos, 9);
    // println!("Time to count 9ply moves: {:?}", start_9ply.elapsed());
    // assert_eq!(legal_9ply, 2439530234167);

    // let start_10ply = Instant::now();
    // let legal_10ply = count_legal_moves(start_pos, 10);
    // println!("Time to count 10ply moves: {:?}", start_10ply.elapsed());
    // assert_eq!(legal_10ply, 69352859712417);
}

// #[test]
// pub fn test_kiwipete() {
//     let mut pos = Position::new("r3k2r/p1ppqpb1/bn2pnp1/2pP4/1p2P3/2N2N2/PPQ1BPPP/R1B1K2R w KQkq - 0 1");
//     assert_eq!(count_legal_moves(&mut pos, 1), 48);
//     assert_eq!(count_legal_moves(&mut pos, 2), 2039);
//     assert_eq!(count_legal_moves(&mut pos, 3), 97862);
// }

// #[test]
// pub fn test_position_3() {
//     let mut pos = Position::new("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
//     assert_eq!(count_legal_moves(&mut pos, 1), 44);
//     assert_eq!(count_legal_moves(&mut pos, 2), 1486);
//     assert_eq!(count_legal_moves(&mut pos, 3), 62379);
// }
