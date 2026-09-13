<div align="center">

# tomo

**A TUI-based Pomodoro timer to help you stay focused and productive.**

</div>

<hr>

<div align="center">
● <a href="#features">Features</a> ● <a href="#installation">Installation</a> ● <a href="#usage">Usage</a> ● <a href="#configuration">Configuration</a><br>
● <a href="#keybindings">Keybindings</a> ● <a href="#command-line-options">Command Line Options</a> ● <a href="#credit-where-credit-is-due">Credits</a> ● <a href="#license">License</a>
</div>

<br>

![Demo Animation](docs/demo.webp)

## Features

- **Pomodoro Timer:** Focus sessions with short and long breaks to maximize productivity
- **Customizable Durations:** Configure focus, short break, and long break durations via CLI or config file
- **Session Tracking:** Track completed focus sessions and total sessions
- **Terminal User Interface:** Clean TUI built with [ratatui](https://github.com/ratatui/ratatui) for an intuitive in-terminal experience
- **Hooks Support:** Execute custom commands when sessions start (focus, short break, long break)
- **Session Alarms:** Optional alarm sounds for session transitions
- **Lightweight & Fast:** Written in Rust for optimal performance

## Installation

### Pre-compiled Binary (Recommended)

Download the latest binary for your platform from the [GitHub Releases](https://github.com/FAZuH/tomo/releases).

```sh
# Linux (x64)
curl -LO https://github.com/FAZuH/tomo/releases/latest/download/tomo-linux-x64
chmod +x tomo-linux-x64
sudo mv tomo-linux-x64 /usr/local/bin/tomo

# macOS Intel (x64)
curl -LO https://github.com/FAZuH/tomo/releases/latest/download/tomo-macos-x64
chmod +x tomo-macos-x64
sudo mv tomo-macos-x64 /usr/local/bin/tomo

# macOS Apple Silicon (ARM64)
curl -LO https://github.com/FAZuH/tomo/releases/latest/download/tomo-macos-arm64
chmod +x tomo-macos-arm64
sudo mv tomo-macos-arm64 /usr/local/bin/tomo

# Windows (x64 - PowerShell)
# Download tomo-windows-x64.exe from the releases page
# Add it to your PATH or run directly
```

### Build from Source (Cargo)

#### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Cargo (included with Rust)
- `~/.cargo/bin` be in PATH

#### Steps

```sh
cargo install --git https://github.com/FAZuH/tomo
```

## Usage

Simply run `tomo` to start the Pomodoro timer with default settings:

```sh
tomo
```

The timer will start immediately in focus mode (25 minutes by default).

### Command Line Options

Customize timer durations via command line arguments:

```sh
# Custom focus and break durations
tomo --focus 50m --short-break 10m --long-break 20m

# Custom long break interval (after how many focus sessions)
tomo --long-interval 4

# Shorthand options
tomo -f 45m -s 5m -l 15m -L 3
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--focus` | `-f` | `25m` | Focus session duration |
| `--short-break` | `-s` | `5m` | Short break duration |
| `--long-break` | `-l` | `15m` | Long break duration |
| `--long-interval` | `-L` | `3` | Number of focus sessions before a long break |

**Time format:** Use `h` for hours, `m` for minutes, `s` for seconds (e.g., `25m`, `1.5h`, `90s`).

## Configuration

`tomo` stores its configuration in a YAML file. The config file is automatically created on first run.

### Config Location

| Platform | Path |
|----------|------|
| Linux | `~/.config/tomo/config.yaml` |
| macOS | `~/Library/Application Support/tomo/config.yaml` |
| Windows | `%APPDATA%\tomo\config.yaml` |

### Config Options

```yaml
pomodoro:
  timer:
    focus: 1500                 # 25 minutes in seconds
    short: 300                  # 5 minutes in seconds
    long: 900                   # 15 minutes in seconds
    long_interval: 3            # Long break after 3 focus sessions
    auto_start_on_launch: false # Start automatically on launch
    auto_focus: false           # Auto-start focus sessions
    auto_short: false           # Auto-start short breaks
    auto_long: false            # Auto-start long breaks
  hook:
    focus: ""                   # Command to run on focus start
    short: ""                   # Command to run on short break start
    long: ""                    # Command to run on long break start
  alarm:
    focus:
      path: null                # Path to focus start sound
      volume: 100               # Volume percentage (0-100)
    short:
      path: null                # Path to short break start sound
      volume: 100               # Volume percentage (0-100)
    long:
      path: null                # Path to long break start sound
      volume: 100               # Volume percentage (0-100)
```

## Keybindings

### Timer View

| Key | Action |
|-----|--------|
| `Space` | Toggle pause/resume |
| `↑` / `↓` / `k` / `j` | Add / subtract 1 minute |
| `←` / `→` / `h` / `l` | Add / subtract 30 seconds |
| `Enter` | Skip to next session |
| `Backspace` | Reset current session |
| `m` | Stop alarm sound |
| `s` | Open settings |
| `?` | Toggle keybinds help |
| `q` | Quit |

### Settings View

| Key | Action |
|-----|--------|
| `↑` / `↓` / `k` / `j` | Move selection up / down |
| `Tab` / `Shift+Tab` | Next / previous section |
| `1` / `2` / `3` | Jump to section |
| `Enter` / `Space` | Edit selected setting / toggle |
| `s` | Save settings |
| `c` / `y` | Copy selected value |
| `v` / `p` | Paste copied value |
| `Esc` | Back to timer |
| `?` | Toggle keybinds help |
| `m` | Stop alarm sound |
| `q` | Quit |

## Credit where credit is due

This project is inspired by:

- **[Pomofocus](https://pomofocus.io/app)** - A customizable pomodoro timer web app that works on desktop & mobile browser
- **[pomo](https://github.com/Bahaaio/pomo)** - A simple Pomodoro CLI tool

## License

`tomo` is distributed under the terms of the [MIT](https://spdx.org/licenses/MIT.html) license.
