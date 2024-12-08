// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bitboards;
mod magic_bitboards;
mod moves;
mod position;
mod slint_interface;
mod utils;

use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    gen_magics: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    if args.gen_magics {
        magic_bitboards::precompute()?;
    } else {
        magic_bitboards::load_precomputed_moves();
        slint_interface::run_application()?;
    }
    Ok(())
}
