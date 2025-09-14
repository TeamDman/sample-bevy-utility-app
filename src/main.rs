use clap::Parser;
use eyre::Result;

mod window_proc;

use teamy_rust_windows_utils::hicon::get_icon_from_current_module;
use window_proc::window_proc;

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
    use teamy_rust_windows_utils::console::{is_inheriting_console, detach_console};
    use teamy_rust_windows_utils::event_loop::run_message_loop;
    use teamy_rust_windows_utils::tray::add_tray_icon;
    use teamy_rust_windows_utils::window::create_window_for_tray;
    use tracing::{info, level_filters::LevelFilter};
    use tracing_subscriber::{EnvFilter, fmt::SubscriberBuilder, util::SubscriberInitExt};
    use windows::core::w;

    // Setup tracing
    SubscriberBuilder::default()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::builder().parse_lossy(LevelFilter::INFO.to_string())
        }))
        .finish()
        .init();

    info!("Starting tray application");

    // Handle console
    if !is_inheriting_console() {
        detach_console();
    }

    // Create window
    let window = create_window_for_tray(Some(window_proc))?;

    // Add tray icon
    let icon = get_icon_from_current_module(w!("aaa_my_icon"))?;
    let tooltip = w!("Sample Bevy Utility App");
    add_tray_icon(window, icon, tooltip)?;

    // Run message loop
    run_message_loop()?;

    Ok(())
}
