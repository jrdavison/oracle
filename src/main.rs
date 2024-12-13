// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bitboards;
mod moves;
mod position;
mod ui;
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
        bitboards::magics::compute()?;
    } else {
        bitboards::magics::LookupTables::load_all();
        ui::run_application()?;
    }
    Ok(())
}
