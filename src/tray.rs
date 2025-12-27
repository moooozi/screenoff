use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, LineTo, MoveToEx, SelectObject,
    SetBkColor, SetTextColor, DT_LEFT, DT_SINGLELINE, DT_VCENTER, HGDIOBJ, PS_SOLID,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};
use windows::Win32::UI::Controls::{DRAWITEMSTRUCT, MEASUREITEMSTRUCT, ODS_SELECTED, ODT_MENU};
use windows::Win32::UI::Shell::{Shell_NotifyIconW, NIF_ICON, NIM_MODIFY, NOTIFYICONDATAW};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DefWindowProcW, DestroyMenu, GetCursorPos, LoadImageW,
    PostMessageW, PostQuitMessage, SetForegroundWindow, TrackPopupMenu, HICON, IMAGE_FLAGS,
    IMAGE_ICON, MF_OWNERDRAW, TPM_NONOTIFY, TPM_RETURNCMD, WM_DESTROY, WM_HOTKEY, WM_LBUTTONUP,
    WM_NULL, WM_RBUTTONUP, WM_USER,
};

use crate::config::{save_config, Config};
use crate::monitors::{get_monitors, toggle_monitors};

pub static mut CONFIG: *mut Config = std::ptr::null_mut();

pub static mut TRAY_HWND: HWND = HWND(std::ptr::null_mut());

pub static mut HINSTANCE: windows::Win32::Foundation::HINSTANCE =
    windows::Win32::Foundation::HINSTANCE(std::ptr::null_mut());

pub const IDI_SCREEN_ON: u16 = 101;
pub const IDI_SCREEN_OFF: u16 = 102;

pub fn load_icon_from_resource(id: u16) -> HICON {
    unsafe {
        HICON(
            LoadImageW(
                Some(HINSTANCE),
                PCWSTR((id as usize) as *mut u16),
                IMAGE_ICON,
                0,
                0,
                IMAGE_FLAGS(0),
            )
            .unwrap()
            .0,
        )
    }
}

pub fn update_tray_icon(icon_id: u16) {
    let hicon = load_icon_from_resource(icon_id);
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = unsafe { TRAY_HWND };
    nid.uID = 1;
    nid.uFlags = NIF_ICON;
    nid.hIcon = hicon;
    unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid).unwrap() };
}

const WM_MEASUREITEM: u32 = 0x002C;
const WM_DRAWITEM: u32 = 0x002B;

fn is_startup_enabled() -> bool {
    unsafe {
        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let value_name: Vec<u16> = "ScreenOff"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_READ,
            &mut hkey,
        )
        .is_ok()
        {
            let mut data: [u16; 512] = [0; 512];
            let mut data_size = (data.len() * 2) as u32;
            let result = RegQueryValueExW(
                hkey,
                PCWSTR(value_name.as_ptr()),
                None,
                None,
                Some(data.as_mut_ptr() as *mut u8),
                Some(&mut data_size),
            );
            let _ = RegCloseKey(hkey);
            result.is_ok()
        } else {
            false
        }
    }
}

fn toggle_startup() {
    if is_startup_enabled() {
        // Remove from startup
        unsafe {
            let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let value_name: Vec<u16> = "ScreenOff"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut hkey = HKEY::default();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                Some(0),
                KEY_WRITE,
                &mut hkey,
            )
            .is_ok()
            {
                let _ = RegDeleteValueW(hkey, PCWSTR(value_name.as_ptr()));
                let _ = RegCloseKey(hkey);
            }
        }
    } else {
        // Add to startup
        unsafe {
            let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let value_name: Vec<u16> = "ScreenOff"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let exe_path = std::env::current_exe().unwrap();
            let exe_path_str = format!("\"{}\"", exe_path.display());
            let exe_path_wide: Vec<u16> = exe_path_str
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut hkey = HKEY::default();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                Some(0),
                KEY_WRITE,
                &mut hkey,
            )
            .is_ok()
            {
                let _ = RegSetValueExW(
                    hkey,
                    PCWSTR(value_name.as_ptr()),
                    Some(0),
                    REG_SZ,
                    Some(&std::slice::from_raw_parts(
                        exe_path_wide.as_ptr() as *const u8,
                        exe_path_wide.len() * 2,
                    )),
                );
                let _ = RegCloseKey(hkey);
            }
        }
    }
}

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
        WM_HOTKEY => {
            // Global hotkey pressed, toggle monitors
            unsafe {
                if !CONFIG.is_null() {
                    toggle_monitors(&mut *CONFIG);
                }
            }
        }
        WM_MEASUREITEM => {
            let measure_item = unsafe { &mut *(lparam.0 as *mut MEASUREITEMSTRUCT) };
            if measure_item.CtlType == ODT_MENU {
                measure_item.itemWidth = 250;
                measure_item.itemHeight = 30;
                return LRESULT(1);
            }
        }
        WM_DRAWITEM => {
            let draw_item = unsafe { &mut *(lparam.0 as *mut DRAWITEMSTRUCT) };
            if draw_item.CtlType == ODT_MENU {
                let item_id = draw_item.itemID;
                let all_monitors = get_monitors();
                let screens_off = unsafe { !(*CONFIG).saved_modes.is_empty() };

                let (text, checked, disabled, is_separator) = if screens_off {
                    // Screen off mode menu
                    if item_id == 1000 {
                        ("Turn back on", false, false, false)
                    } else if item_id == 1001 {
                        ("", false, false, true)
                    } else if item_id == 1002 {
                        ("Start on Sign in", is_startup_enabled(), false, false)
                    } else if item_id == 1003 {
                        ("", false, false, true)
                    } else if item_id == 1004 {
                        ("Exit", false, false, false)
                    } else {
                        ("", false, false, false)
                    }
                } else {
                    // Screen on mode menu
                    if item_id == 0 {
                        ("Select Monitors to turn off:", false, true, false)
                    } else if item_id <= all_monitors.len() as u32 {
                        let index = (item_id - 1) as usize;
                        let (monitor, friendly_name) = &all_monitors[index];
                        let checked = unsafe { (*CONFIG).secondary_monitors.contains(monitor) };
                        (friendly_name.as_str(), checked, false, false)
                    } else if item_id == all_monitors.len() as u32 + 1 {
                        ("", false, false, true)
                    } else if item_id == all_monitors.len() as u32 + 2 {
                        ("Turn off selected screens", false, false, false)
                    } else if item_id == all_monitors.len() as u32 + 3 {
                        ("", false, false, true)
                    } else if item_id == all_monitors.len() as u32 + 4 {
                        ("Start on Sign in", is_startup_enabled(), false, false)
                    } else if item_id == all_monitors.len() as u32 + 5 {
                        ("Exit", false, false, false)
                    } else {
                        ("", false, false, false)
                    }
                };
                if is_separator {
                    // draw separator
                    let rect = &draw_item.rcItem;
                    let hdc = draw_item.hDC;
                    let bg_color = COLORREF(0x00222222);
                    let brush = unsafe { CreateSolidBrush(bg_color) };
                    unsafe { FillRect(hdc, rect, brush) };
                    let _ = unsafe { DeleteObject(HGDIOBJ(brush.0)) };
                    let pen = unsafe { CreatePen(PS_SOLID, 1, COLORREF(0x00444444)) };
                    let old_pen = unsafe { SelectObject(hdc, HGDIOBJ(pen.0)) };
                    let _ = unsafe {
                        MoveToEx(
                            hdc,
                            rect.left,
                            rect.top + (rect.bottom - rect.top) / 2,
                            None,
                        )
                    };
                    let _ =
                        unsafe { LineTo(hdc, rect.right, rect.top + (rect.bottom - rect.top) / 2) };
                    let _ = unsafe { SelectObject(hdc, old_pen) };
                    let _ = unsafe { DeleteObject(HGDIOBJ(pen.0)) };
                    return LRESULT(1);
                }
                let rect = &draw_item.rcItem;
                let hdc = draw_item.hDC;
                let bg_color = if (draw_item.itemState.0 & ODS_SELECTED.0) != 0 {
                    COLORREF(0x00333333)
                } else {
                    COLORREF(0x00222222)
                };
                let text_color = if disabled {
                    COLORREF(0x00888888)
                } else {
                    COLORREF(0x00FFFFFF)
                };
                let brush = unsafe { CreateSolidBrush(bg_color) };
                unsafe { FillRect(hdc, rect, brush) };
                let _ = unsafe { DeleteObject(HGDIOBJ(brush.0)) };
                unsafe { SetBkColor(hdc, bg_color) };
                unsafe { SetTextColor(hdc, text_color) };
                let mut display_text = text.to_string();
                if checked {
                    display_text = format!("✓ {}", display_text);
                }
                let mut text_wide: Vec<u16> = display_text
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                let draw_rect = RECT {
                    left: rect.left + if checked { 8 } else { 24 },
                    top: rect.top + 4,
                    right: rect.right - 8,
                    bottom: rect.bottom - 4,
                };
                let mut rect_copy = draw_rect;
                unsafe {
                    DrawTextW(
                        hdc,
                        &mut text_wide,
                        &mut rect_copy as *mut _,
                        DT_LEFT | DT_SINGLELINE | DT_VCENTER,
                    )
                };
                return LRESULT(1);
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
        let screens_off = !config.saved_modes.is_empty();

        if screens_off {
            // Screen off mode: Show "Turn back on" and "Exit" only
            // "Turn back on"
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, 1000 as usize, PCWSTR::null());

            // Separator
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, 1001 as usize, PCWSTR::null());

            // "Start on Sign in"
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, 1002 as usize, PCWSTR::null());

            // Separator
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, 1003 as usize, PCWSTR::null());

            // Exit
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, 1004 as usize, PCWSTR::null());
        } else {
            // Screen on mode: Show full menu
            // Header
            let id = 0u32;
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, id as usize, PCWSTR::null());

            // Monitors
            for (i, _) in all_monitors.iter().enumerate() {
                let id = (i + 1) as u32;
                let _ = AppendMenuW(hmenu, MF_OWNERDRAW, id as usize, PCWSTR::null());
            }

            // Separator
            let id = (all_monitors.len() + 1) as u32;
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, id as usize, PCWSTR::null());

            // "Turn off selected screens"
            let id = (all_monitors.len() + 2) as u32;
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, id as usize, PCWSTR::null());

            // Separator
            let id = (all_monitors.len() + 3) as u32;
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, id as usize, PCWSTR::null());

            // "Start on Sign in"
            let id = (all_monitors.len() + 4) as u32;
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, id as usize, PCWSTR::null());

            // Exit
            let id = (all_monitors.len() + 5) as u32;
            let _ = AppendMenuW(hmenu, MF_OWNERDRAW, id as usize, PCWSTR::null());
        }

        let mut pt: POINT = std::mem::zeroed();
        GetCursorPos(&mut pt).unwrap();

        // Required for tray menus: set foreground window so menu dismisses when clicking outside
        let _ = SetForegroundWindow(hwnd);

        loop {
            let cmd = TrackPopupMenu(
                hmenu,
                TPM_RETURNCMD | TPM_NONOTIFY,
                pt.x,
                pt.y,
                Some(0),
                hwnd,
                None,
            );

            // Required for tray menus: post a message to ensure menu closes properly
            let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));

            if cmd.0 > 0 {
                if screens_off {
                    // Screen off mode menu
                    if cmd.0 == 1000 {
                        // "Turn back on"
                        toggle_monitors(config);
                        break;
                    } else if cmd.0 == 1002 {
                        // "Start on Sign in"
                        toggle_startup();
                        // Continue loop to re-show menu
                    } else if cmd.0 == 1004 {
                        // Exit
                        PostQuitMessage(0);
                        break;
                    } else {
                        break;
                    }
                } else {
                    // Screen on mode menu
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
                        // Continue the loop to re-show the menu
                    } else if index == all_monitors.len() + 1 {
                        // "Turn off selected screens"
                        toggle_monitors(config);
                        break;
                    } else if index == all_monitors.len() + 3 {
                        // "Start on Sign in"
                        toggle_startup();
                        // Continue loop to re-show menu
                    } else if index == all_monitors.len() + 4 {
                        // Exit
                        PostQuitMessage(0);
                        break;
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        let _ = DestroyMenu(hmenu);
    }
}
