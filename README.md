# Breaker Pro

A lightweight, native health timer application for Linux built with **Rust** and **GTK4**. Breaker Pro helps you manage your sitting and standing habits by reminding you to take breaks and switch positions, keeping your posture healthy during long working hours.

## Key Features

- **Extremely Lightweight:** Compiled to native machine code. Uses shared GTK4 system libraries, consuming only ~45MB RAM while running.
- **Smart Timer:** Automatically tracks your sitting, standing, and transition times.
- **Full Screen Overlay:** A non-intrusive yet effective screen overlay blocks your screen during break transitions with customizable opacity and strict mode.
- **Emergency Exit:** An emergency exit button (Hold 5 seconds) allows you to dismiss the overlay in urgent situations.
- **Auto-Start:** Optionally launch automatically when you log into your Linux desktop environment.
- **System Tray Integration:** Runs quietly in the background and can be summoned anytime from the taskbar tray.

## Requirements

To run Breaker Pro natively on your Linux system, ensure you have the following installed:
- `libgtk-4-1`
- `libc6`
- `libglib2.0-0`

## Build from Source

You will need the Rust toolchain and GTK4 development headers to build the application from source.

1. **Install Dependencies (Ubuntu/Debian):**
   ```bash
   sudo apt install build-essential rustc cargo libgtk-4-dev
   ```

2. **Clone and Build:**
   ```bash
   git clone https://github.com/harveyzoka/Breaker-Pro-Linux.git
   cd Breaker-Pro-Linux
   cargo build --release
   ```

3. **Run:**
   ```bash
   ./target/release/breaker-pro-rust
   ```

## Installation

You can package this application as a `.deb` file or build an `.AppImage` for distribution across various Linux environments.

## License

MIT License. Feel free to fork and modify!
