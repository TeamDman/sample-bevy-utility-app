use eyre::Context;
use std::process::Command;
use teamy_rust_windows_utils::console::console_create;
use teamy_rust_windows_utils::console::console_detach;
use teamy_rust_windows_utils::tray::WM_TASKBAR_CREATED;
use teamy_rust_windows_utils::tray::WM_USER_TRAY_CALLBACK;
use teamy_rust_windows_utils::tray::delete_tray_icon;
use teamy_rust_windows_utils::tray::re_add_tray_icon;
use tracing::error;
use tracing::info;
use windows::Win32::Foundation::*;
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
                    _ = console_detach();
                    console_create()?;
                    info!("You may use the 'Hide Logs' tray action to hide the console again.");
                }
                ID_HIDE_LOGS => {
                    info!("Hiding logs");
                    _ = console_detach()?;
                }
                ID_LAUNCH_BEVY => {
                    info!("Launching Bevy");
                    launch_bevy()?;
                }
                ID_EXIT => {
                    info!("Exiting");
                    unsafe { PostQuitMessage(0) };
                }
                _ => {}
            }
            Ok(LRESULT(0))
        }
        WM_CLOSE => {
            delete_tray_icon(hwnd)?;
            unsafe { DestroyWindow(hwnd) }.ok();
            Ok(LRESULT(0))
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            Ok(LRESULT(0))
        }
        _ => Ok(unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }),
    }
}

fn show_context_menu(hwnd: HWND) -> eyre::Result<()> {
    unsafe {
        let menu = CreatePopupMenu()?;
        AppendMenuW(menu, MF_STRING, ID_SHOW_LOGS, w!("Show Logs"))?;
        AppendMenuW(menu, MF_STRING, ID_HIDE_LOGS, w!("Hide Logs"))?;
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
