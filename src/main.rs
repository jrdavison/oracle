// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod position;
mod utils;

use num_traits::FromPrimitive;
use position::Position;
use slint::VecModel;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use utils::types::{File, Rank, Square};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new().unwrap();

    let position = Rc::new(RefCell::new(Position::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    )));

    set_application_state(&ui, &position, -1); // -1 means no piece is being dragged
    setup_callbacks(&ui, &position);

    ui.run().unwrap();

    Ok(())
}

fn set_application_state(ui: &AppWindow, position: &Rc<RefCell<Position>>, dragged_piece_sq: i32) {
    let pos = position.borrow();
    ui.set_board_state(BoardState {
        board: Rc::new(VecModel::from(pos.get_board_i32())).into(),
        turn: pos.get_side_to_move(),
        halfmove_clock: pos.get_halfmove_clock(),
        fullmove_count: pos.get_fullmove_count(),
    });

    ui.set_dragged_piece_sq(dragged_piece_sq);
}

fn setup_callbacks(ui: &AppWindow, position: &Rc<RefCell<Position>>) {
    let ui_weak = ui.as_weak();
    let position_weak = Rc::downgrade(position);

    ui.global::<RustInterface>().on_square_from_xy(|x: f32, y: f32| {
        let file = File::from_x(x);
        let rank = Rank::from_y(y);
        Square::make_square(file, rank) as i32
    });

    ui.global::<RustInterface>().on_move_piece(move |src: i32, dest: i32| {
        let ui = ui_weak.upgrade().unwrap();
        let position = position_weak.upgrade().unwrap();
        let mut position_mut = position.borrow_mut();

        let src_sq = Square::from_u8(src as u8).unwrap();
        let dest_sq = Square::from_u8(dest as u8).unwrap();

        position_mut.move_piece(src_sq, dest_sq);
        drop(position_mut);

        set_application_state(&ui, &position, -1);
    });
}
