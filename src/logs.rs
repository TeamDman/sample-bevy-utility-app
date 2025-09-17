use std::sync::Mutex;
use teamy_rust_windows_utils::log::LOG_BUFFER;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::SubscriberBuilder;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::util::SubscriberInitExt;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogsContextMenuButton {
    ShowLogs,
    HideLogs,
}

/// This controls the creation behaviour of the tray icon menu items.
/// If detached, it will create a new console when "Show Logs" is clicked.
/// If attached, it will detach the console when "Hide Logs" is clicked.
/// If the process is started from an existing console, it will still show "Show Logs" since that creates a new console window.
pub static LOGS_CONTEXT_MENU_BUTTON: Mutex<LogsContextMenuButton> =
    Mutex::new(LogsContextMenuButton::ShowLogs);

pub fn init_tracing() -> eyre::Result<()> {
    SubscriberBuilder::default()
        .with_env_filter(
            EnvFilter::builder()
                // .with_default_directive(Level::INFO.into())
                .with_default_directive(Level::DEBUG.into())
                .from_env_lossy(),
        )
        .with_writer(std::io::stderr.and(LOG_BUFFER.clone()))
        .finish()
        .init();
    Ok(())
}
