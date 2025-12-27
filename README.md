# Screen Off

A Windows system tray application that allows you to quickly disable/enable secondary monitors with a single click or keyboard shortcut.

## Features

- **System Tray Icon**: Minimalist tray application with no visible window
- **Left Click**: Toggle secondary monitors on/off
- **Right Click**: Configure which monitors are considered "secondary"
- **Keyboard Shortcut**: Press `Ctrl + Alt + T` to toggle monitors from anywhere
- **Persistent Configuration**: Your monitor settings are saved in `screenoff_config.json`

## Usage

1. **Build the application**:
   ```bash
   cargo build --release
   ```

2. **Run the application**:
   ```bash
   target\release\screenoff.exe
   ```
   Or simply double-click the executable.

3. **Configure secondary monitors**:
   - Right-click the tray icon
   - Select which monitors you want to toggle (checked monitors will be disabled/enabled)
   - If no monitors are configured, all non-primary monitors will be toggled

4. **Toggle monitors**:
   - Left-click the tray icon, OR
   - Press `Ctrl + Alt + T`

## How It Works

The application uses Windows Display Settings API to:
- Enumerate all connected displays
- Identify primary and secondary monitors
- Dynamically disable/enable monitors by modifying display settings

When you disable a monitor, Windows will temporarily turn it off. The monitor can be re-enabled with another click/shortcut press.

## Configuration

Configuration is stored in `screenoff_config.json` in the same directory as the executable. The file contains:
```json
{
  "secondary_monitors": [
    "\\\\.\\DISPLAY2",
    "\\\\.\\DISPLAY3"
  ]
}
```

## Building

Requirements:
- Rust toolchain (2021 edition or later)
- Windows OS
- Dependencies:
  - `windows` crate (v0.62.2)
  - `serde` and `serde_json` for configuration

## Notes

- The application runs as a Windows subsystem application (no console window)
- The executable is completely standalone once built
- Monitor states persist until changed or system restart