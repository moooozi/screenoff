# Screen Off

A Windows system tray application that allows you to quickly disable/enable secondary monitors with a single click or keyboard shortcut.

## Features

- **System Tray Icon**: Easily accessible from the Windows system tray
- **Double Left Click**: Toggle secondary monitors on/off
- **Right Click**: Configure which monitors are considered "secondary" and startup behavior
- **Keyboard Shortcut**: Press `Ctrl + Alt + T` to toggle monitors from anywhere

## Installation
Download the latest release from the [Releases](https://github.com/moooozi/screenoff/releases/) page.

## Usage

1. **Configure secondary monitors**:
   - Right-click the tray icon
   - Select which monitors you want to toggle (checked monitors will be disabled/enabled)
   - By default, all non-primary monitors will be toggled

2. **Toggle monitors**:
   - Double left-click the tray icon, OR
   - Press `Ctrl + Alt + T`


## Build
   ```bash
   cargo build --release
   ```


## License
This project is licensed under the GPL-3.0-or-later License.