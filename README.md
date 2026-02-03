# TomatoCrab

A terminal-based Pomodoro timer written in Rust.

## Features

- Customizable work and break durations
- Session tracking with task descriptions
- Persistent history stored locally
- Statistics with weekly charts
- Tabbed interface (Timer, History, Stats)

## Installation

```bash
cargo install --path .
```

Or run directly:

```bash
cargo run --release
```

## Usage

### Start the timer

```bash
tomatocrab
```

With custom durations (in minutes):

```bash
tomatocrab --duration 30 --short-break 10 --long-break 20
```

### Commands

```bash
tomatocrab list              # Show session history
tomatocrab list --today      # Today's sessions only
tomatocrab stats             # View productivity statistics
```

### Controls

| Key       | Action                |
|-----------|-----------------------|
| Enter     | Start timer / confirm |
| Space     | Pause work session    |
| s         | Skip break            |
| r         | Reset current session |
| Tab       | Switch tabs           |
| Up/Down   | Scroll history        |
| f         | Cycle time filters    |
| q         | Quit                  |

## Configuration

Default settings:
- Work session: 25 minutes
- Short break: 5 minutes
- Long break: 15 minutes (every 4 sessions)

All durations can be overridden via command line flags.

## Data Storage

Sessions are saved to your system's data directory:
- Linux: `~/.local/share/tomatocrab/`
- macOS: `~/Library/Application Support/tomatocrab/`
- Windows: `%APPDATA%\tomatocrab\`

## License

MIT
