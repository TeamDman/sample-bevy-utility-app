pub mod logs;
mod window_proc;

use crate::logs::init_console;
use crate::logs::init_tracing;
use clap::Parser;
use eyre::Result;
use teamy_rust_windows_utils::event_loop::run_message_loop;
use teamy_rust_windows_utils::hicon::get_icon_from_current_module;
use teamy_rust_windows_utils::tray::add_tray_icon;
use teamy_rust_windows_utils::window::create_window_for_tray;
use tracing::info;
use window_proc::window_proc;
use windows::core::w;

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
        run_bevy()?;
    } else {
        run_tray()?;
    }

    Ok(())
}

fn run_bevy() -> Result<()> {
    use bevy::prelude::*;

    fn setup(mut commands: Commands) {
        commands.spawn(Camera2d);
        commands.spawn(Text::new("Hello Bevy!"));
    }

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();

    Ok(())
}

fn run_tray() -> Result<()> {
    init_tracing()?;
    info!("Starting tray application");

    init_console()?;

    let window = create_window_for_tray(Some(window_proc))?;

    add_tray_icon(
        window,
        get_icon_from_current_module(w!("aaa_my_icon"))?,
        w!("Sample Bevy Utility App"),
    )?;

    run_message_loop(None)?;

    Ok(())
}
