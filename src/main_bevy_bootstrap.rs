use std::process::Command;
use std::process::Stdio;
use teamy_rust_windows_utils::job::SpawnJobExt;
use teamy_rust_windows_utils::log::IoHook;

pub fn launch_bevy() -> eyre::Result<()> {
    let mut child = Command::new(std::env::current_exe()?)
        .arg("--bevy")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn_job()?;

    child.hook_stdio_logs()?;

    Ok(())
}
