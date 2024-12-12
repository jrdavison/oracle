use crate::moves::info::MoveInfo;
use crate::position::Position;
use crate::utils::Piece;
use crate::utils::{Color, File, Rank, Square};
use itertools::Itertools;
use num_traits::FromPrimitive;
use slint::VecModel;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

slint::include_modules!();

pub fn run_application() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new().unwrap();

    // start: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    let position = Rc::new(RefCell::new(Position::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    )));

    set_application_state(&ui, &position, -1, true); // -1 no piece is being dragged
    init_callbacks(&ui, &position);

    Ok(ui.run()?)
}

fn set_application_state(ui: &AppWindow, position: &Rc<RefCell<Position>>, dragged_piece_sq: i32, compute_moves: bool) {
    let mut pos = position.borrow_mut();

    let side_to_move = pos.side_to_move();
    let last_move = pos.last_move();
    let board_i32 = pos.board.iter().map(|&piece| piece as i32).collect::<Vec<_>>();

    ui.set_board_state(BoardState {
        board: Rc::new(VecModel::from(board_i32)).into(),
        last_move_from: last_move.from as i32,
        last_move_to: last_move.to as i32,
    });
    ui.set_dragged_piece_sq(dragged_piece_sq);

    if compute_moves {
        let move_history = format_move_history(&pos);
        pos.compute_valid_moves(side_to_move);
        ui.set_dashboard_state(DashboardState {
            move_history: Rc::new(VecModel::from(move_history)).into(),
            halfmove_clock: pos.halfmove_clock(),
            en_passant_square: pos.en_passant_square().into(),
            compute_time: pos.compute_time().into(),
        });
    }
}

fn init_callbacks(ui: &AppWindow, position: &Rc<RefCell<Position>>) {
    let ui_weak = ui.as_weak();
    let position_weak = Rc::downgrade(position);

    ui.global::<RustInterface>().on_highlight_valid_move_sq({
        let position_weak = position_weak.clone();
        move |from, to| {
            let position = position_weak.upgrade().unwrap();
            let position = position.borrow();
            position.valid_move(Square::from_u8(from as u8).unwrap(), Square::from_u8(to as u8).unwrap())
        }
    });

    ui.global::<RustInterface>().on_square_from_xy(|x: f32, y: f32| {
        let file = File::from_x(x);
        let rank = Rank::from_y(y);
        Square::make_square(file, rank) as i32
    });

    ui.global::<RustInterface>().on_move_piece({
        let position_weak = position_weak.clone();
        let ui_weak = ui_weak.clone();
        move |src: i32, dest: i32| {
            let ui = ui_weak.upgrade().unwrap();
            let position = position_weak.upgrade().unwrap();
            let mut position_mut = position.borrow_mut();

            let src_sq = Square::from_u8(src as u8).unwrap();
            let dest_sq = Square::from_u8(dest as u8).unwrap();

            let move_info = position_mut.move_piece(src_sq, dest_sq, true);
            drop(position_mut);

            let valid_move = move_info.is_valid();
            if valid_move {
                set_application_state(&ui, &position, -1, valid_move);
            }
        }
    });

    ui.global::<RustInterface>().on_undo_move({
        let position_weak = position_weak.clone();
        let ui_weak = ui_weak.clone();
        move || {
            let position = position_weak.upgrade().unwrap();
            let ui = ui_weak.upgrade().unwrap();
            let mut position_mut = position.borrow_mut();

            let undo_success = position_mut.undo_move();
            drop(position_mut);

            if undo_success {
                set_application_state(&ui, &position, -1, true);
            }
        }
    });

    ui.global::<RustInterface>().on_redo_move({
        let position_weak = position_weak.clone();
        let ui_weak = ui_weak.clone();
        move || {
            let position = position_weak.upgrade().unwrap();
            let ui = ui_weak.upgrade().unwrap();
            let mut position_mut = position.borrow_mut();

            let redo_success = position_mut.redo_move();
            drop(position_mut);

            if redo_success {
                set_application_state(&ui, &position, -1, true);
            }
        }
    });
}

fn format_move_history(pos: &Position) -> Vec<SlintMoveInfo> {
    let history = pos.move_history();
    let mut redo_history = pos.redo_history();
    redo_history.reverse();

    let mut combined_history = Vec::new();
    combined_history.extend_from_slice(&history);
    combined_history.extend_from_slice(&redo_history);

    let mut moves = chunk_move_history(&combined_history);
    if moves.is_empty() {
        moves.push(SlintMoveInfo {
            move_no: 1,
            white: "".into(),
            black: "".into(),
            active_move: 0,
        });
    } else if let Some(active_move) = history.last() {
        let fullmove_idx = match history.len() {
            0 => 0,
            len => {
                if Piece::color_of(active_move.moved_piece) == Color::White {
                    len / 2
                } else {
                    (len / 2) - 1
                }
            }
        };

        if let Some(move_ref) = moves.get_mut(fullmove_idx) {
            let color = Piece::color_of(active_move.moved_piece);
            move_ref.active_move = if color == Color::White { 1 } else { 2 };
        }
    }

    moves
}

fn chunk_move_history(history: &[MoveInfo]) -> Vec<SlintMoveInfo> {
    let mut moves_iter = history.iter();
    let mut slint_move_info: Vec<SlintMoveInfo> = Vec::new();

    // first move is by black, add ... to white
    if let Some(first_move) = moves_iter.next() {
        if Piece::color_of(first_move.moved_piece) == Color::Black {
            slint_move_info.push(SlintMoveInfo {
                move_no: first_move.fullmove_count,
                white: "...".into(),
                black: first_move.notation.clone(),
                active_move: 0,
            });
        } else if let Some(first_response) = moves_iter.next() {
            slint_move_info.push(SlintMoveInfo {
                move_no: first_move.fullmove_count,
                white: first_move.notation.clone(),
                black: first_response.notation.clone(),
                active_move: 0,
            });
        } else {
            slint_move_info.push(SlintMoveInfo {
                move_no: first_move.fullmove_count,
                white: first_move.notation.clone(),
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
                    move_no: chunk_vec[0].fullmove_count,
                    white: chunk_vec[0].notation.clone(),
                    black: chunk_vec[1].notation.clone(),
                    active_move: 0,
                });
            }
            1 => {
                slint_move_info.push(SlintMoveInfo {
                    move_no: chunk_vec[0].fullmove_count,
                    white: chunk_vec[0].notation.clone(),
                    black: "".into(),
                    active_move: 0,
                });
            }
            _ => {}
        }
    }

    slint_move_info
}
