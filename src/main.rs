// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bitboards;
mod magic_bitboards;
mod moves;
mod position;
mod utils;

use clap::Parser;
use magic_bitboards::storage;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use position::Position;
use slint::SharedString;
use slint::VecModel;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use utils::{Color, File, Rank, Square};

slint::include_modules!();

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    gen_magics: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    if args.gen_magics {
        magic_bitboards::generate()?;
    } else {
        storage::load_move_dbs(); // force lazy static initialization of move databases

        let ui = AppWindow::new().unwrap();

        let position = Rc::new(RefCell::new(Position::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        )));

        set_application_state(&ui, &position, -1, true); // -1 no piece is being dragged
        setup_callbacks(&ui, &position);

        ui.run()?;
    }
    Ok(())
}

fn set_application_state(ui: &AppWindow, position: &Rc<RefCell<Position>>, dragged_piece_sq: i32, compute_moves: bool) {
    let mut pos = position.borrow_mut();
    let side_to_move = pos.side_to_move();

    ui.set_board_state(BoardState {
        board: Rc::new(VecModel::from(pos.board_i32())).into(),
        turn: Color::to_i32(&side_to_move).unwrap(),
        halfmove_clock: pos.halfmove_clock(),
        fullmove_count: pos.fullmove_count(),
    });
    ui.set_dragged_piece_sq(dragged_piece_sq);

    if compute_moves {
        pos.compute_valid_moves(side_to_move);
        ui.set_compute_time(SharedString::from(pos.compute_time_ms()));
    }
}

fn setup_callbacks(ui: &AppWindow, position: &Rc<RefCell<Position>>) {
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

            let move_info = position_mut.move_piece(src_sq, dest_sq);
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
