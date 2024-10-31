// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bitboards;
mod position;
mod utils;

use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use position::Position;
use slint::SharedString;
use slint::VecModel;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use utils::types::{Color, File, Rank, Square};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new().unwrap();

    let position = Rc::new(RefCell::new(Position::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    )));

    set_application_state(&ui, &position, -1, true); // -1 no piece is being dragged
    setup_callbacks(&ui, &position);

    ui.run().unwrap();

    return Ok(());
}

fn set_application_state(ui: &AppWindow, position: &Rc<RefCell<Position>>, dragged_piece_sq: i32, compute_moves: bool) {
    let mut pos = position.borrow_mut();
    let side_to_move = pos.get_side_to_move();

    ui.set_board_state(BoardState {
        board: Rc::new(VecModel::from(pos.get_board_i32())).into(),
        turn: Color::to_i32(&side_to_move).unwrap(),
        halfmove_clock: pos.get_halfmove_clock(),
        fullmove_count: pos.get_fullmove_count(),
    });
    ui.set_dragged_piece_sq(dragged_piece_sq);

    if compute_moves {
        pos.compute_valid_moves(side_to_move);
        ui.set_compute_time(SharedString::from(pos.get_compute_time_string()));
    }
}

fn setup_callbacks(ui: &AppWindow, position: &Rc<RefCell<Position>>) {
    let ui_weak = ui.as_weak();
    let position_weak = Rc::downgrade(&position);

    ui.global::<RustInterface>().on_highlight_valid_move_sq({
        let position_weak = position_weak.clone();
        move |from, to| {
            let position = position_weak.upgrade().unwrap();
            let position = position.borrow();
            position.is_valid_move(Square::from_u8(from as u8).unwrap(), Square::from_u8(to as u8).unwrap())
        }
    });

    ui.global::<RustInterface>().on_square_from_xy(|x: f32, y: f32| {
        let file = File::from_x(x);
        let rank = Rank::from_y(y);
        Square::make_square(file, rank) as i32
    });

    ui.global::<RustInterface>().on_move_piece({
        let position_weak = position_weak.clone();
        move |src: i32, dest: i32| {
            let ui = ui_weak.upgrade().unwrap();
            let position = position_weak.upgrade().unwrap();
            let mut position_mut = position.borrow_mut();

            let src_sq = Square::from_u8(src as u8).unwrap();
            let dest_sq = Square::from_u8(dest as u8).unwrap();

            let valid_move = position_mut.move_piece(src_sq, dest_sq);
            drop(position_mut);

            set_application_state(&ui, &position, -1, valid_move);
        }
    });
}
