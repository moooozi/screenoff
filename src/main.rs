#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod monitors;
mod tray;

use windows::core::PCWSTR;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DispatchMessageW, GetMessageW, RegisterClassW, ShowWindow, TranslateMessage,
    CW_USEDEFAULT, MSG, SW_HIDE, WINDOW_EX_STYLE, WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set DPI awareness to prevent blurry text
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let mut config = config::load_config();

    // Only update secondary monitors if starting fresh (no saved disabled state)
    // If monitors were disabled when app closed, keep that state
    if config.saved_modes.is_empty() {
        monitors::update_secondary_monitors(&mut config);
    }

    // Print monitor information
    let all_monitors = monitors::get_monitors();
    eprintln!("=== Detected Monitors ===");
    for (monitor, friendly_name) in &all_monitors {
        eprintln!("{} -> {}", monitor, friendly_name);
    }
    eprintln!("=========================");

    let icon_id = if config.saved_modes.is_empty() {
        tray::IDI_SCREEN_ON
    } else {
        tray::IDI_SCREEN_OFF
    };

    let config_box = Box::new(config);
    unsafe { tray::CONFIG = Box::into_raw(config_box) };

    let hinstance = unsafe { GetModuleHandleW(PCWSTR::null()) }.unwrap();
    let hinstance = HINSTANCE(hinstance.0);
    unsafe { tray::HINSTANCE = hinstance };
    let class_name_wide: Vec<u16> = "ScreenOffTrayClass"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let class = WNDCLASSW {
        lpfnWndProc: Some(tray::window_proc),
        hInstance: hinstance,
        lpszClassName: PCWSTR(class_name_wide.as_ptr()),
        ..Default::default()
    };
    unsafe { RegisterClassW(&class) };

    let window_name_wide: Vec<u16> = "ScreenOff"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name_wide.as_ptr()),
            PCWSTR(window_name_wide.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(hinstance),
            None,
        )
    }
    .unwrap();

    unsafe { tray::TRAY_HWND = hwnd };

    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };

    // Register hotkey Ctrl+Alt+S
    // unsafe { RegisterHotKey(hwnd, 1, HOT_KEY_MODIFIERS::MOD_CONTROL | HOT_KEY_MODIFIERS::MOD_ALT, 'S' as u32) };

    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_USER;
    nid.hIcon = tray::load_icon_from_resource(icon_id);
    let tip = "Screen Off";
    let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
    nid.szTip[..tip_wide.len()].copy_from_slice(&tip_wide);

    unsafe { Shell_NotifyIconW(NIM_ADD, &nid).unwrap() };

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        let _ = unsafe { TranslateMessage(&msg) };
        unsafe {
            DispatchMessageW(&msg);
        }
    }

    let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) };
    // unsafe { UnregisterHotKey(hwnd, 1) };
    unsafe {
        let _ = Box::from_raw(tray::CONFIG);
    };

    Ok(())
}
