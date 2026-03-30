# emu-launcher

A terminal-based emulator launcher with a cozy TUI. Pick your emulator from a menu, launch it, and return to the menu when you're done.

## Features

- Keyboard-driven menu (arrow keys or `j`/`k`)
- Add new emulators directly from the launcher
- Config persists automatically as TOML
- Launches emulators in the background and waits for them to close

## Installation

Requires [Rust](https://rustup.rs/).

```
git clone https://github.com/jm12460/emu-launcher
cd emu-launcher
cargo build --release
```

The binary will be at `target/release/emu-launcher`.

## Usage

```
./emu-launcher
```

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Launch selected emulator |
| `q` / `Esc` | Quit |

Select **+ add emulator** at the bottom of the list to add a new entry.

## Configuration

Config is stored at `~/.config/emu-launcher/config.toml`. A default config is created on first run with entries for RetroArch, PCSX2, mGBA, Cemu, and Dolphin — edit the paths to match your system.

```toml
[[emulators]]
name = "RetroArch"
path = "/usr/bin/retroarch"
args = []

[[emulators]]
name = "PCSX2"
path = "/usr/bin/pcsx2"
args = []
```

You can also pass launch arguments via the `args` array.

## Dependencies

- [crossterm](https://github.com/crossterm-rs/crossterm) — terminal UI
- [serde](https://serde.rs/) + [toml](https://crates.io/crates/toml) — config serialization
- [dirs](https://crates.io/crates/dirs) — platform config directory
