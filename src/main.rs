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

    let pos = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    ui.set_board_state(BoardState {
        board: Rc::new(VecModel::from(pos.get_board_i32())).into(),
        turn: pos.get_side_to_move(),
        halfmove: pos.get_halfmove_clock(),
        fullmove: pos.get_fullmove_number(),
    });
    ui.set_dragged_piece(DraggedPiece {
        piece: -1, // default to -1 to indicate no piece is being dragged
        x: -1.0,
        y: -1.0,
    });

    let ui_weak = ui.as_weak();
    ui.global::<Logic>().on_drag_piece(move |x, y| {
        // println!("x: {},  y: {}", x, y
        if let Some(ui) = ui_weak.upgrade() {
            let file = File::from_x(x);
            let rank = Rank::from_y(y);
            let sq = Square::make_square(file, rank);
            // println!("sq: {:?}", sq);
            ui.set_dragged_piece(DraggedPiece { piece: sq as i32, x, y });
        }
    });

    ui.run().unwrap();

    Ok(())
}
