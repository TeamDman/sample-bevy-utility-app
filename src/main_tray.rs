use crate::logs::init_tracing;
use crate::window_proc::window_proc;
use eyre::Result;
use teamy_rust_windows_utils::console::hide_default_console_or_attach_ctrl_handler;
use teamy_rust_windows_utils::event_loop::run_message_loop;
use teamy_rust_windows_utils::hicon::get_icon_from_current_module;
use teamy_rust_windows_utils::tray::add_tray_icon;
use teamy_rust_windows_utils::window::create_window_for_tray;
use tracing::info;
use windows::core::w;

pub fn main_tray() -> Result<()> {
    init_tracing()?;
    info!("Starting tray application");

    hide_default_console_or_attach_ctrl_handler()?;

    let window = create_window_for_tray(Some(window_proc))?;

    add_tray_icon(
        window,
        get_icon_from_current_module(w!("aaa_my_icon"))?,
        w!("Sample Bevy Utility App"),
    )?;

    run_message_loop(None)?;

    Ok(())
}
