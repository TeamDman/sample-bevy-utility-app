use crate::logs::LOG_BUFFER;
use crate::logs::LOG_DISPLAY_STATE;
use crate::logs::LogDisplayState;
use eyre::Context;
use teamy_rust_windows_utils::console::attach_ctrl_c_handler;
use std::process::Command;
use teamy_rust_windows_utils::console::console_attach;
use teamy_rust_windows_utils::console::console_create;
use teamy_rust_windows_utils::console::console_detach;
use teamy_rust_windows_utils::tray::WM_TASKBAR_CREATED;
use teamy_rust_windows_utils::tray::WM_USER_TRAY_CALLBACK;
use teamy_rust_windows_utils::tray::delete_tray_icon;
use teamy_rust_windows_utils::tray::re_add_tray_icon;
use tracing::debug;
use tracing::error;
use tracing::info;
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::ATTACH_PARENT_PROCESS;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;

const ID_SHOW_LOGS: usize = 1;
const ID_HIDE_LOGS: usize = 2;
const ID_LAUNCH_BEVY: usize = 3;
const ID_EXIT: usize = 4;

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match window_proc_inner(hwnd, message, wparam, lparam) {
        Ok(result) => result,
        Err(e) => {
            error!("Error in window_proc: {}", e);
            LRESULT(0)
        }
    }
}

fn window_proc_inner(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> eyre::Result<LRESULT> {
    match message {
        WM_USER_TRAY_CALLBACK => {
            match lparam.0 as u32 {
                WM_LBUTTONUP => {
                    info!("Tray icon left button up - launching Bevy");
                    if let Err(e) = launch_bevy() {
                        error!("Failed to launch Bevy: {}", e);
                    }
                }
                WM_RBUTTONUP => show_context_menu(hwnd)?,
                WM_CONTEXTMENU => show_context_menu(hwnd)?,
                _ => info!("Tray icon unknown event: {}", lparam.0),
            }
            Ok(LRESULT(0))
        }
        m if m == *WM_TASKBAR_CREATED => {
            re_add_tray_icon()?;
            Ok(LRESULT(0))
        }
        WM_COMMAND => {
            let id = wparam.0 as usize;
            match id {
                ID_SHOW_LOGS => {
                    info!("Showing logs");

                    debug!("Detaching any existing console");
                    _ = console_detach();

                    debug!("Creating new console");
                    console_create()?;

                    debug!("Attaching ctrl+c handler to new console");
                    attach_ctrl_c_handler()?;

                    debug!("Updating log status");
                    *(LOG_DISPLAY_STATE.lock().unwrap()) = LogDisplayState::AttachedToConsole;

                    debug!("Replaying log buffer to new console");
                    LOG_BUFFER.replay(&mut std::io::stderr())?;

                    info!("You may use the 'Hide Logs' tray action to hide the console again.");
                    Ok(LRESULT(0))
                }
                ID_HIDE_LOGS => {
                    info!("Hiding logs");

                    debug!("Detaching console");
                    _ = console_detach();

                    debug!("Attaching to parent console if present");
                    if console_attach(ATTACH_PARENT_PROCESS).is_ok() {
                        debug!("Attaching ctrl+c handler to new console");
                        attach_ctrl_c_handler()?;
                    }

                    debug!("Updating log status");
                    *(LOG_DISPLAY_STATE.lock().unwrap()) = LogDisplayState::DetachedFromConsole;

                    Ok(LRESULT(0))
                }
                ID_LAUNCH_BEVY => {
                    info!("Launching Bevy");
                    launch_bevy()?;
                    Ok(LRESULT(0))
                }
                ID_EXIT => {
                    info!("Exiting");
                    // Synchronously trigger our cleanup behaviour
                    Ok(unsafe { SendMessageW(hwnd, WM_CLOSE, None, None) })
                }
                _ => Ok(LRESULT(0)),
            }
        }
        WM_CLOSE => {
            _ = delete_tray_icon(hwnd);
            unsafe { DestroyWindow(hwnd) }.ok();
            Ok(LRESULT(0))
        }
        WM_DESTROY => {
            _ = delete_tray_icon(hwnd);
            unsafe { PostQuitMessage(0) };
            Ok(LRESULT(0))
        }
        _ => Ok(unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }),
    }
}

fn show_context_menu(hwnd: HWND) -> eyre::Result<()> {
    unsafe {
        let menu = CreatePopupMenu()?;
        match *LOG_DISPLAY_STATE.lock().unwrap() {
            LogDisplayState::AttachedToConsole => {
                AppendMenuW(menu, MF_STRING, ID_HIDE_LOGS, w!("Hide Logs"))?;
            }
            LogDisplayState::DetachedFromConsole => {
                AppendMenuW(menu, MF_STRING, ID_SHOW_LOGS, w!("Show Logs"))?;
            }
        }
        AppendMenuW(menu, MF_SEPARATOR, 0, None)?;
        AppendMenuW(menu, MF_STRING, ID_LAUNCH_BEVY, w!("Launch Bevy"))?;
        AppendMenuW(menu, MF_SEPARATOR, 0, None)?;
        AppendMenuW(menu, MF_STRING, ID_EXIT, w!("Exit"))?;

        let mut point = POINT::default();
        GetCursorPos(&mut point)?;
        SetForegroundWindow(hwnd).ok()?;
        TrackPopupMenu(menu, TPM_RIGHTBUTTON, point.x, point.y, Some(0), hwnd, None).ok()?;
        DestroyMenu(menu)?;
    }
    Ok(())
}

fn launch_bevy() -> eyre::Result<()> {
    Command::new(std::env::current_exe()?)
        .arg("--bevy")
        .spawn()
        .wrap_err("Bevy process did not run happily")?;
    Ok(())
}
