use std::collections::HashMap;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    ChangeDisplaySettingsExW, EnumDisplayDevicesW, EnumDisplaySettingsW, CDS_TYPE, DEVMODEW,
    DISPLAY_DEVICEW, DISPLAY_DEVICE_PRIMARY_DEVICE, DISP_CHANGE_SUCCESSFUL, ENUM_CURRENT_SETTINGS,
};
use wmi::{COMLibrary, WMIConnection};

use crate::config::{save_config, Config};

pub fn get_monitor_friendly_names() -> HashMap<String, String> {
    let mut names = HashMap::new();
    if let Ok(com_lib) = COMLibrary::new() {
        if let Ok(wmi_con) = WMIConnection::with_namespace_path("root\\wmi", com_lib) {
            #[derive(serde::Deserialize, Debug)]
            #[allow(non_snake_case)]
            struct WmiMonitorID {
                InstanceName: String,
                UserFriendlyName: Vec<u8>,
            }
            if let Ok(results) = wmi_con.query::<WmiMonitorID>() {
                for monitor in results {
                    if let Some(model_start) = monitor.InstanceName.find('\\') {
                        if let Some(model_end) = monitor.InstanceName[model_start + 1..].find('\\')
                        {
                            let model =
                                &monitor.InstanceName[model_start + 1..model_start + 1 + model_end];
                            let friendly = String::from_utf8_lossy(&monitor.UserFriendlyName)
                                .trim_matches('\0')
                                .to_string();
                            if !friendly.is_empty() {
                                names.insert(model.to_string(), friendly);
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("WMI connection failed");
        }
    } else {
        eprintln!("COM lib failed");
    }
    names
}

pub fn get_monitors() -> Vec<(String, String)> {
    let friendly_names = get_monitor_friendly_names();
    let mut devices = Vec::new();
    let mut dd: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
    dd.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
    let mut i = 0;
    while unsafe { EnumDisplayDevicesW(PCWSTR::null(), i, &mut dd, 0) }.as_bool() {
        let name = &dd.DeviceName;
        let name_len = name.iter().position(|&c| c == 0).unwrap_or(32);
        let name_str = String::from_utf16_lossy(&name[..name_len]);
        if !name_str.is_empty() {
            // Check if active by trying to get settings
            let mut devmode = DEVMODEW::default();
            devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
            let device_name_wide: Vec<u16> =
                name_str.encode_utf16().chain(std::iter::once(0)).collect();
            if unsafe {
                EnumDisplaySettingsW(
                    PCWSTR(device_name_wide.as_ptr()),
                    ENUM_CURRENT_SETTINGS,
                    &mut devmode,
                )
            }
            .as_bool()
            {
                // Get the monitor's friendly name
                let display_num = if let Some(num_str) = name_str.strip_prefix("\\\\.\\DISPLAY") {
                    num_str.parse::<u32>().unwrap_or(i as u32 + 1)
                } else {
                    i as u32 + 1
                };
                let mut monitor_dd: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
                monitor_dd.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
                let friendly_name_str = if unsafe {
                    EnumDisplayDevicesW(PCWSTR(device_name_wide.as_ptr()), 0, &mut monitor_dd, 0)
                }
                .as_bool()
                {
                    let device_id = &monitor_dd.DeviceID;
                    let device_id_len = device_id.iter().position(|&c| c == 0).unwrap_or(128);
                    let device_id_str = String::from_utf16_lossy(&device_id[..device_id_len]);
                    // Extract model from DeviceID: MONITOR\MODEL\...
                    let model = if let Some(start) = device_id_str.find('\\') {
                        if let Some(end) = device_id_str[start + 1..].find('\\') {
                            device_id_str[start + 1..start + 1 + end].to_string()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    if let Some(friendly) = friendly_names.get(&model) {
                        friendly.clone()
                    } else {
                        let device_string = &monitor_dd.DeviceString;
                        let ds_len = device_string.iter().position(|&c| c == 0).unwrap_or(32);
                        let ds = String::from_utf16_lossy(&device_string[..ds_len]);
                        if !ds.is_empty() && ds != "Generic PnP Monitor" {
                            ds
                        } else {
                            format!("Display {}", display_num)
                        }
                    }
                } else {
                    format!("Display {}", display_num)
                };
                devices.push((name_str, friendly_name_str));
            }
        }
        i += 1;
    }
    devices
}

pub fn get_primary_monitor() -> Option<String> {
    let mut dd: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
    dd.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
    if unsafe { EnumDisplayDevicesW(PCWSTR::null(), 0, &mut dd, 0) }.as_bool() {
        if (dd.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE).0 != 0 {
            let name = &dd.DeviceName;
            let len = name.iter().position(|&c| c == 0).unwrap_or(32);
            let name_str = String::from_utf16_lossy(&name[..len]);
            return Some(name_str);
        }
    }
    None
}

pub fn disable_monitor(
    device_name: &str,
    saved_modes: &mut HashMap<String, (u32, u32, i32, i32)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_name_wide: Vec<u16> = device_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut devmode: DEVMODEW = unsafe { std::mem::zeroed() };
    devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
    if unsafe {
        EnumDisplaySettingsW(
            PCWSTR(device_name_wide.as_ptr()),
            ENUM_CURRENT_SETTINGS,
            &mut devmode,
        )
    }
    .as_bool()
    {
        saved_modes.insert(
            device_name.to_string(),
            (
                devmode.dmPelsWidth,
                devmode.dmPelsHeight,
                unsafe { devmode.Anonymous1.Anonymous2.dmPosition.x },
                unsafe { devmode.Anonymous1.Anonymous2.dmPosition.y },
            ),
        );
        eprintln!(
            "Disabling {} (was {}x{} at ({}, {}))",
            device_name,
            devmode.dmPelsWidth,
            devmode.dmPelsHeight,
            unsafe { devmode.Anonymous1.Anonymous2.dmPosition.x },
            unsafe { devmode.Anonymous1.Anonymous2.dmPosition.y }
        );
        // Set resolution to 0x0 to disable
        devmode.dmPelsWidth = 0;
        devmode.dmPelsHeight = 0;
        let result = unsafe {
            ChangeDisplaySettingsExW(
                PCWSTR(device_name_wide.as_ptr()),
                Some(&devmode),
                None,
                CDS_TYPE(0),
                None,
            )
        };
        if result != DISP_CHANGE_SUCCESSFUL {
            return Err(format!("Failed to disable monitor: {:?}", result).into());
        }
    } else {
        return Err("Failed to get current settings".into());
    }
    Ok(())
}

pub fn enable_all_monitors() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Re-enabling all monitors");
    // Reset all displays to their registry settings
    let result = unsafe { ChangeDisplaySettingsExW(PCWSTR::null(), None, None, CDS_TYPE(0), None) };
    if result != DISP_CHANGE_SUCCESSFUL {
        return Err(format!("Failed to enable monitors: {:?}", result).into());
    }
    Ok(())
}

pub fn toggle_monitors(config: &mut Config) {
    eprintln!(
        "Left click registered: Trying to {} {:?}",
        if config.saved_modes.is_empty() {
            "disable"
        } else {
            "enable"
        },
        config.secondary_monitors
    );
    if config.saved_modes.is_empty() {
        // disable
        for monitor in &config.secondary_monitors {
            if let Err(e) = disable_monitor(monitor, &mut config.saved_modes) {
                eprintln!("Error disabling {}: {}", monitor, e);
            }
        }
    } else {
        // enable - just reset all displays at once
        if let Err(e) = enable_all_monitors() {
            eprintln!("Error enabling monitors: {}", e);
        }
        config.saved_modes.clear();
    }
    save_config(config);
}

pub fn update_secondary_monitors(config: &mut Config) {
    let all_monitors = get_monitors();
    let primary = get_primary_monitor();
    config.secondary_monitors = all_monitors
        .into_iter()
        .filter(|(m, _)| Some(m) != primary.as_ref())
        .map(|(m, _)| m)
        .collect();
    save_config(config);
}
