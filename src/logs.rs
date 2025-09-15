use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use teamy_rust_windows_utils::console::attach_ctrl_c_handler;
use teamy_rust_windows_utils::console::console_detach;
use teamy_rust_windows_utils::console::is_inheriting_console;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::SubscriberBuilder;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Logs are stored in a buffer to be displayed in the console when the user clicks show logs
#[derive(Debug, Clone, Default)]
pub struct BufferSink {
    buffer: Arc<Mutex<Vec<u8>>>,
}
impl BufferSink {
    pub fn replay(&self, writer: &mut impl Write) -> eyre::Result<()> {
        let buffer = self.lock().unwrap();
        writeln!(writer, "=== Previous Logs ===")?;
        writer
            .write_all(&buffer)
            .map_err(|e| eyre::eyre!("Failed to write log buffer to writer: {}", e))?;
        writeln!(writer, "=== End of Previous Logs ===")?;
        Ok(())
    }
}
impl Deref for BufferSink {
    type Target = Arc<Mutex<Vec<u8>>>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl DerefMut for BufferSink {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}
impl Write for BufferSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<'a> MakeWriter<'a> for BufferSink {
    type Writer = BufferSink;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Captures logs to be replayed later when the user requests to see them.
pub static LOG_BUFFER: LazyLock<BufferSink> = LazyLock::new(|| BufferSink::default());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogDisplayState {
    AttachedToConsole,
    DetachedFromConsole,
}

/// This controls the creation behaviour of the tray icon menu items.
/// If detached, it will create a new console when "Show Logs" is clicked.
/// If attached, it will detach the console when "Hide Logs" is clicked.
/// If the process is started from an existing console, it will still show "Show Logs" since that creates a new console window.
pub static LOG_DISPLAY_STATE: Mutex<LogDisplayState> =
    Mutex::new(LogDisplayState::DetachedFromConsole);

/// If the process is not inheriting a console, hide the default console.
pub fn init_console() -> eyre::Result<()> {
    if !is_inheriting_console() {
        _ = console_detach();
    };

    attach_ctrl_c_handler()?;
    Ok(())
}

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
