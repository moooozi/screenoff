# ScreenOff - AI Instructions

Multi-file Windows tray app for toggling secondary monitors using Win32 API directly. Code is organized into modules:
- [src/main.rs](../src/main.rs): Main entry point and window setup
- [src/config.rs](../src/config.rs): Configuration loading/saving
- [src/monitors.rs](../src/monitors.rs): Monitor enumeration, WMI queries for EDID names, and display control
- [src/tray.rs](../src/tray.rs): System tray icon and menu handling

## Development Workflow

**Rules:**
1. Always Check for errors using your API before running code (using `get_errors`).
2. Use `cargo run` during development (skip `cargo build`)

**Debugging Monitor Logic:**
- Monitor enumeration prints on startup between `=== Detected Monitors ===` markers
- Toggle actions print "Disabling/Re-enabling" messages
- Check `screenoff_config.json` in working directory for state

**Hotkey Feature:** Commented-out `RegisterHotKey` calls in code - planned but not implemented
