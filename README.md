# ScreammOS

A retro-modern operating system with DOS feel, built from scratch in Rust.

## Project Description

ScreammOS is an experiment in building an operating system that combines nostalgic DOS aesthetics with modern functionality. The project focuses on creating a retro-inspired experience with window management, various visual themes (including CRT effects), and basic operating system functions.

## Features

- Text-based VGA buffer with 16-color palette
- DOS-inspired interface design
- Window management with overlapping windows
- Multiple visual themes (DOS classic, Amber terminal, Green CRT, Modern)
- Keyboard input handling with scancode processing
- Command-line interface with useful commands
- Basic memory management with heap allocation
- Interrupt handling for hardware events

## Technical Information

- Developed in Rust without standard library (no_std)
- Runs directly on hardware (bare metal)
- Target architecture: x86_64
- Uses bootloader crate for booting
- VGA text mode for 80x25 text-based interface
- Protected mode operation with paging

## Build Process

### Prerequisites

- Rust and Cargo (latest stable version)
- QEMU for system emulation
- `cargo-bootimage` tool

### Installation

Install the required tools:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install bootimage tool
cargo install bootimage

# Install QEMU (Linux example)
# Ubuntu/Debian: apt install qemu-system-x86
# Arch: pacman -S qemu
# Windows: Download and install from https://www.qemu.org/download/
```

### Building

To build ScreammOS:

```bash
# Clone the repository
git clone https://github.com/yourusername/ScreammOS.git
cd ScreammOS

# Build the bootable image
cargo bootimage
```

### Running

To run ScreammOS in QEMU:

```bash
# Linux/macOS
qemu-system-x86_64 -drive format=raw,file=target/x86_64-screamos/debug/bootimage-screamos.bin

# Windows PowerShell
& "C:\Program Files\QEMU\qemu-system-x86_64w.exe" -drive format=raw,file=target/x86_64-screamos/debug/bootimage-screamos.bin
```

#### Windows Batch File

For easier running on Windows, you can create a `run_qemu.bat` file with the following content:

```batch
@echo off
"C:\Program Files\QEMU\qemu-system-x86_64w.exe" -drive format=raw,file=target/x86_64-screamos/debug/bootimage-screamos.bin
```

Then simply run it by double-clicking or typing `.\run_qemu.bat` in PowerShell.

## Available Commands

ScreammOS comes with several built-in commands:

- `help` - Show available commands
- `clear` - Clear the screen
- `version` - Show ScreammOS version
- `theme` - Change the visual theme (`dos`, `amber`, `green`, `modern`)
- `sysinfo` - Display system information
- `memory` - Show memory usage
- `about` - About ScreammOS
- `reboot` - Restart the system

## Roadmap

- [x] Basic keyboard handling
- [x] Command line interface (shell)
- [x] Memory management
- [ ] Simple filesystem
- [ ] Program loading and execution
- [ ] Multitasking
- [ ] Sound support
- [ ] Network stack
- [ ] More DOS-style applications

## License

MIT License

## Project Structure

The ScreammOS codebase is organized into several modules:

- `src/main.rs` - Entry point and kernel initialization
- `src/vga_buffer/` - Text mode display driver
- `src/ui/` - User interface components and window management
- `src/interrupts.rs` - Interrupt handling (CPU exceptions, hardware interrupts)
- `src/keyboard.rs` - Keyboard input handling and command processing
- `src/memory.rs` - Memory management and heap allocation

## Contributing

Contributions to ScreammOS are welcome! Here are some ways you can help:

1. Implement missing features from the roadmap
2. Report bugs or suggest enhancements
3. Improve documentation
4. Add tests

Before submitting a pull request, please make sure your code follows the project's style and conventions.

---

*ScreammOS - The Retro-modern Experience* 