use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DefWindowProcW, DestroyMenu, GetCursorPos,
    PostQuitMessage, TrackPopupMenu, MF_CHECKED, MF_DISABLED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, TPM_NONOTIFY,
    TPM_RETURNCMD, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER,
};
use windows::core::PCWSTR;

use crate::config::{Config, save_config};
use crate::monitors::{get_monitors, toggle_monitors};

pub static mut CONFIG: *mut Config = std::ptr::null_mut();

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_USER => {
            if lparam.0 as u32 == WM_LBUTTONUP {
                // left click, toggle
                unsafe {
                    if !CONFIG.is_null() {
                        toggle_monitors(&mut *CONFIG);
                    }
                }
            } else if lparam.0 as u32 == WM_RBUTTONUP {
                // right click, show menu
                unsafe {
                    if !CONFIG.is_null() {
                        show_menu(hwnd, &mut *CONFIG);
                    }
                }
            }
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
    LRESULT(0)
}

pub fn show_menu(hwnd: HWND, config: &mut Config) {
    unsafe {
        let hmenu = CreatePopupMenu().unwrap();
        let all_monitors = get_monitors();

        // Add header for secondary monitors
        let disabled_text = "Secondary Monitors:";
        let disabled_wide: Vec<u16> = disabled_text.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = AppendMenuW(hmenu, MF_DISABLED | MF_STRING, 0, PCWSTR(disabled_wide.as_ptr()));

        for (i, (monitor, friendly_name)) in all_monitors.iter().enumerate() {
            let checked = config.secondary_monitors.contains(monitor);
            let flags = if checked { MF_CHECKED } else { MF_UNCHECKED } | MF_STRING;

            // Use friendly name if available, otherwise use device name
            let display_name = friendly_name;

            let monitor_wide: Vec<u16> = display_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let _ = AppendMenuW(
                hmenu,
                flags,
                (i + 1) as usize,
                PCWSTR(monitor_wide.as_ptr()),
            );
        }

        // Add separator
        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

        // Add exit option
        let exit_text = "Exit";
        let exit_wide: Vec<u16> = exit_text.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = AppendMenuW(hmenu, MF_STRING, (all_monitors.len() + 1) as usize, PCWSTR(exit_wide.as_ptr()));

        let mut pt: POINT = std::mem::zeroed();
        GetCursorPos(&mut pt).unwrap();
        let cmd = TrackPopupMenu(
            hmenu,
            TPM_RETURNCMD | TPM_NONOTIFY,
            pt.x,
            pt.y,
            Some(0),
            hwnd,
            None,
        );
        if cmd.0 > 0 {
            let index = (cmd.0 - 1) as usize;
            if index < all_monitors.len() {
                if let Some((monitor, _)) = all_monitors.get(index) {
                    if config.secondary_monitors.contains(monitor) {
                        config.secondary_monitors.retain(|m| m != monitor);
                        save_config(config);
                    } else {
                        // Prevent marking all monitors as secondary
                        if config.secondary_monitors.len() + 1 < all_monitors.len() {
                            config.secondary_monitors.push(monitor.clone());
                            save_config(config);
                        }
                    }
                }
            } else if index == all_monitors.len() {
                // Exit
                PostQuitMessage(0);
            }
        }
        let _ = DestroyMenu(hmenu);
    }
}