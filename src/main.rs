// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod position;
mod utils;

use position::Position;
use slint::VecModel;
use std::error::Error;
use std::rc::Rc;
use utils::types::{File, Rank, Square};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new().unwrap();

    // utility functions
    ui.global::<RustInterface>().on_square_from_xy(|x, y| {
        let file = File::from_x(x);
        let rank = Rank::from_y(y);
        Square::make_square(file, rank) as i32
    });

    let pos = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    ui.set_board_state(BoardState {
        board: Rc::new(VecModel::from(pos.get_board_i32())).into(),
        turn: pos.get_side_to_move(),
        halfmove: pos.get_halfmove_clock(),
        fullmove: pos.get_fullmove_number(),
    });
    ui.set_dragged_piece_sq(-1);

    ui.run().unwrap();

    Ok(())
}
