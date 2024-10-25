// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod position;
mod utils;

use position::Position;
use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new().unwrap();

    let pos = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    ui.set_state(OracleState {
        board: pos.get_board_i32().into(),
        turn: pos.get_side_to_move(),
        halfmove: pos.get_halfmove_clock(),
        fullmove: pos.get_fullmove_number(),
    });
    ui.run().unwrap();

    Ok(())
}
