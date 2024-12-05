#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

mod bitboards;
mod magic_bitboards;
mod moves;
mod position;
mod utils;
mod slint_interface;

use clap::Parser;
use std::error::Error;

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
        magic_bitboards::precompute()?;
    } else {
        magic_bitboards::load_precomputed_moves();
        slint_interface::run_application()?;
    }
    Ok(())
}
