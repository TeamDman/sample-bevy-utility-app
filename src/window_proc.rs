use std::process::Command;
use teamy_rust_windows_utils::tray::{WM_TASKBAR_CREATED, WM_USER_TRAY_CALLBACK, delete_tray_icon, re_add_tray_icon};
use tracing::{error, info};
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::AttachConsole;
use windows::Win32::System::Console::FreeConsole;
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
    match message {
        WM_USER_TRAY_CALLBACK => {
            match lparam.0 as u32 {
                WM_LBUTTONUP => {
                    info!("Tray icon left button up - launching Bevy");
                    if let Err(e) = launch_bevy() {
                        error!("Failed to launch Bevy: {}", e);
                    }
                }
                WM_RBUTTONUP => info!("Tray icon right button up"),
                WM_CONTEXTMENU => show_context_menu(hwnd),
                _ => info!("Tray icon unknown event: {}", lparam.0),
            }
            LRESULT(0)
        }
        m if m == *WM_TASKBAR_CREATED => {
            if let Err(e) = re_add_tray_icon() {
                error!("Failed to re-add tray icon: {}", e);
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = wparam.0 as usize;
            match id {
                ID_SHOW_LOGS => {
                    info!("Showing logs");
                    // Attach to parent's console
                    unsafe { let _ = AttachConsole(u32::MAX); }; // ATTACH_PARENT_PROCESS
                }
                ID_HIDE_LOGS => {
                    info!("Hiding logs");
                    unsafe { let _ = FreeConsole(); };
                }
                ID_LAUNCH_BEVY => {
                    info!("Launching Bevy");
                    if let Err(e) = launch_bevy() {
                        error!("Failed to launch Bevy: {}", e);
                    }
                }
                ID_EXIT => {
                    info!("Exiting");
                    unsafe { PostQuitMessage(0) };
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            if let Err(e) = delete_tray_icon(hwnd) {
                error!("Failed to delete tray icon: {}", e);
            }
            unsafe { DestroyWindow(hwnd) }.ok();
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn show_context_menu(hwnd: HWND) {
    unsafe {
        let menu = CreatePopupMenu().unwrap();
        AppendMenuW(menu, MF_STRING, ID_SHOW_LOGS, w!("Show Logs")).unwrap();
        AppendMenuW(menu, MF_STRING, ID_HIDE_LOGS, w!("Hide Logs")).unwrap();
        AppendMenuW(menu, MF_SEPARATOR, 0, None).unwrap();
        AppendMenuW(menu, MF_STRING, ID_LAUNCH_BEVY, w!("Launch Bevy")).unwrap();
        AppendMenuW(menu, MF_SEPARATOR, 0, None).unwrap();
        AppendMenuW(menu, MF_STRING, ID_EXIT, w!("Exit")).unwrap();

        let mut point = POINT::default();
        GetCursorPos(&mut point).unwrap();
        TrackPopupMenu(menu, TPM_RIGHTBUTTON, point.x, point.y, Some(0), hwnd, None).unwrap();
        DestroyMenu(menu).unwrap();
    }
}

fn launch_bevy() -> eyre::Result<()> {
    Command::new(std::env::current_exe()?)
        .arg("--bevy")
        .spawn()?;
    Ok(())
}
