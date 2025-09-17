pub mod logs;
mod window_proc;
pub mod main_bevy;
pub mod main_tray;
pub mod main_bevy_bootstrap;

use clap::Parser;
use eyre::Result;

#[derive(Parser)]
struct Args {
    /// Run in Bevy mode
    #[arg(long)]
    bevy: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    if args.bevy {
        main_bevy::main_bevy()?;
    } else {
        main_tray::main_tray()?;
    }

    Ok(())
}

