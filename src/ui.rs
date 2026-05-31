use crate::bitboards::Bitboard;
use crate::game::{GameMove, GameState};
use crate::utils::{Color, File, Piece, Rank, Square};
use itertools::Itertools;
use num_traits::FromPrimitive;
use slint::VecModel;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

slint::include_modules!();

pub fn run_application() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    // start: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    let game = Rc::new(RefCell::new(GameState::new(
        "r1bBr3/1p3pk1/p1p1p3/n1P4n/3PP3/P5QP/B4PP1/R4RK1 b - - 0 23",
    )));

    set_application_state(&ui, &game, Square::Count, true);
    init_callbacks(&ui, &game);

    Ok(ui.run()?)
}

fn set_application_state(ui: &AppWindow, game: &Rc<RefCell<GameState>>, dragged_piece: Square, refresh_position: bool) {
    let mut game = game.borrow_mut();

    if refresh_position {
        game.position.compute_legal_moves();
        let move_history = format_move_history(&game);
        ui.set_dashboard_state(DashboardState {
            move_history: Rc::new(VecModel::from(move_history)).into(),
            halfmove_clock: game.position.halfmove_clock(),
            en_passant_square: game.position.en_passant_sq().into(),
            avg_compute_time: game.position.avg_compute_time().into(),
        });
    }

    ui.set_board_state(build_board_state(&game, dragged_piece));
}

fn build_board_state(game: &GameState, dragged_piece: Square) -> BoardState {
    let pos = &game.position;
    let side_to_move = pos.side_to_move();
    let last_move = game.last_move();
    let board_i32 = pos.board.iter().map(|&piece| piece as i32).collect::<Vec<_>>();
    let legal_targets = if dragged_piece == Square::Count {
        vec![false; Square::Count as usize]
    } else {
        bitboard_to_square_flags(pos.legal_destinations_from(dragged_piece))
    };
    let check_sq = if pos.king_in_check(side_to_move) {
        pos.king_squares[side_to_move as usize]
    } else {
        Square::Count
    };
    let dragged_piece_value = if dragged_piece == Square::Count {
        Piece::Empty
    } else {
        pos.board[dragged_piece as usize]
    };

    BoardState {
        board: Rc::new(VecModel::from(board_i32)).into(),
        legal_targets: Rc::new(VecModel::from(legal_targets)).into(),
        last_move_from: last_move.from as i32,
        last_move_to: last_move.to as i32,
        check_sq: check_sq as i32,
        dragged_piece_sq: dragged_piece as i32,
        dragged_piece: dragged_piece_value as i32,
    }
}

fn bitboard_to_square_flags(mask: Bitboard) -> Vec<bool> {
    Square::iter().map(|sq| (mask & (1u64 << (sq as u8))) != 0).collect()
}

fn init_callbacks(ui: &AppWindow, game: &Rc<RefCell<GameState>>) {
    let ui_weak = ui.as_weak();
    let game_weak = Rc::downgrade(game);

    ui.global::<RustInterface>().on_begin_drag({
        let game_weak = game_weak.clone();
        let ui_weak = ui_weak.clone();
        move |src| {
            let ui: AppWindow = ui_weak.upgrade().expect("could not upgrade ui");
            let game = game_weak.upgrade().expect("could not upgrade game");
            let game = game.borrow();
            let src_sq = Square::from_u8(src as u8).unwrap_or_default();
            let dragged_piece = if src_sq == Square::Count || game.position.board[src_sq as usize] == Piece::Empty {
                Square::Count
            } else {
                src_sq
            };
            ui.set_board_state(build_board_state(&game, dragged_piece));
        }
    });

    ui.global::<RustInterface>()
        .on_square_from_xy(|x: f32, y: f32, sq_size: f32| {
            let file = File::from_x(x, sq_size);
            let rank = Rank::from_y(y, sq_size);
            Square::from(file, rank) as i32
        });

    ui.global::<RustInterface>().on_move_piece({
        let game_weak = game_weak.clone();
        let ui_weak = ui_weak.clone();
        move |src: i32, dest: i32| {
            let ui: AppWindow = ui_weak.upgrade().expect("could not upgrade ui");
            let game = game_weak.upgrade().expect("could not upgrade game");
            let mut game_mut = game.borrow_mut();

            let src_sq = Square::from_u8(src as u8).unwrap_or_default();
            let dest_sq = Square::from_u8(dest as u8).unwrap_or_default();

            let move_info = game_mut.play_move(src_sq, dest_sq).unwrap_or_default();
            drop(game_mut);

            set_application_state(&ui, &game, Square::Count, move_info.is_valid());
        }
    });

    ui.global::<RustInterface>().on_undo_move({
        let game_weak = game_weak.clone();
        let ui_weak = ui_weak.clone();
        move || {
            let game = game_weak.upgrade().expect("could not upgrade game");
            let ui = ui_weak.upgrade().expect("could not upgrade ui");
            let mut game_mut = game.borrow_mut();

            let undo_success = game_mut.undo_move();
            drop(game_mut);

            if undo_success {
                set_application_state(&ui, &game, Square::Count, true);
            }
        }
    });

    ui.global::<RustInterface>().on_redo_move({
        let game_weak = game_weak.clone();
        let ui_weak = ui_weak.clone();
        move || {
            let game = game_weak.upgrade().expect("could not upgrade game");
            let ui = ui_weak.upgrade().expect("could not upgrade ui");
            let mut game_mut = game.borrow_mut();

            let redo_success = game_mut.redo_move();
            drop(game_mut);

            if redo_success {
                set_application_state(&ui, &game, Square::Count, true);
            }
        }
    });
}

fn format_move_history(game: &GameState) -> Vec<SlintMoveInfo> {
    let history = game.move_history();
    let mut redo_history = game.redo_history().to_vec();
    redo_history.reverse();

    let mut combined_history = Vec::new();
    combined_history.extend_from_slice(history);
    combined_history.extend_from_slice(&redo_history);

    let mut moves = chunk_move_history(&combined_history);
    if moves.is_empty() {
        let white_str = if game.position.side_to_move() == Color::White {
            ""
        } else {
            "..."
        };
        moves.push(SlintMoveInfo {
            move_no: 1,
            white: white_str.into(),
            black: "".into(),
            active_move: 0,
        });
    } else if let Some(active_move) = history.last() {
        let color = Piece::color_of(active_move.info.moved_piece);
        let fullmove_idx = match history.len() {
            0 => 0,
            len => {
                if color == Color::White {
                    len / 2
                } else {
                    std::cmp::max((len + 1) / 2, 1) - 1
                }
            }
        };

        if let Some(move_ref) = moves.get_mut(fullmove_idx) {
            move_ref.active_move = if color == Color::White { 1 } else { 2 };
        }
    }

    moves
}

fn chunk_move_history(history: &[GameMove]) -> Vec<SlintMoveInfo> {
    let mut moves_iter = history.iter();
    let mut slint_move_info: Vec<SlintMoveInfo> = Vec::new();

    // first move is by black, add ... to white
    if let Some(first_move) = moves_iter.next() {
        if Piece::color_of(first_move.info.moved_piece) == Color::Black {
            slint_move_info.push(SlintMoveInfo {
                move_no: first_move.info.fullmove_count,
                white: "...".into(),
                black: first_move.notation.clone().into(),
                active_move: 0,
            });
        } else if let Some(first_response) = moves_iter.next() {
            slint_move_info.push(SlintMoveInfo {
                move_no: first_move.info.fullmove_count,
                white: first_move.notation.clone().into(),
                black: first_response.notation.clone().into(),
                active_move: 0,
            });
        } else {
            slint_move_info.push(SlintMoveInfo {
                move_no: first_move.info.fullmove_count,
                white: first_move.notation.clone().into(),
                black: "".into(),
                active_move: 0,
            });
        }
    }

    for chunk in &moves_iter.chunks(2) {
        let chunk_vec = chunk.collect::<Vec<_>>();
        match chunk_vec.len() {
            2 => {
                slint_move_info.push(SlintMoveInfo {
                    move_no: chunk_vec[0].info.fullmove_count,
                    white: chunk_vec[0].notation.clone().into(),
                    black: chunk_vec[1].notation.clone().into(),
                    active_move: 0,
                });
            }
            1 => {
                slint_move_info.push(SlintMoveInfo {
                    move_no: chunk_vec[0].info.fullmove_count,
                    white: chunk_vec[0].notation.clone().into(),
                    black: "".into(),
                    active_move: 0,
                });
            }
            _ => {}
        }
    }

    slint_move_info
}
