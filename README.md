# brs

[日本語](./README.ja.md)

A BMS (Be-Music Source) rhythm game player implemented in Rust.

## Features

- **LR2/beatoraja Compatible Timing**: Switchable between LR2 and beatoraja judgment systems
- **Multiple Gauge Systems**: ASSIST EASY, EASY, NORMAL, HARD, EX-HARD, HAZARD
- **Gauge Auto Shift (GAS)**: Automatically tracks all gauge types and achieves the best possible clear
- **Long Note Support**: Full support for LN, CN, and HCN modes
- **BGA Video Playback**: Supports MPG, AVI, MP4, WebM, and more via ffmpeg
- **IIDX Controller Support**: Native support for IIDX arcade-style controllers with analog axis input
- **Visual Options**: SUDDEN+, HIDDEN+, LIFT, and floating HI-SPEED
- **Lane Options**: MIRROR, RANDOM, R-RANDOM
- **FAST/SLOW Display**: Timing feedback with millisecond precision

## Installation

### Download Pre-built Binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/dqn/brs/releases).

### ffmpeg Requirement

brs requires ffmpeg for video BGA playback. Install it before running:

**macOS:**
```bash
brew install ffmpeg
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev
```

**Windows:**

Download pre-built binaries from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) and add to your PATH.

## Building from Source

### Prerequisites

- Rust 1.85 or later
- ffmpeg development libraries (see above)

### Build

```bash
cargo build --release
```

The binary will be located at `target/release/brs`.

## Usage

Run brs with a BMS folder path:

```bash
./brs /path/to/bms/folder
```

### Controls

Default keyboard layout for 7-key mode:

| Key | Lane |
|-----|------|
| Left Shift | Scratch |
| Z | 1 |
| S | 2 |
| X | 3 |
| D | 4 |
| C | 5 |
| F | 6 |
| V | 7 |

### Settings

Access the settings screen from the song select menu to configure:

- Key bindings
- HI-SPEED and Green Number
- SUDDEN+/HIDDEN+/LIFT
- Gauge type
- Judgment system (beatoraja/LR2)
- FAST/SLOW display options

## Supported Formats

### Charts
- .bms, .bme, .bml

### Audio
- WAV, OGG, MP3, FLAC

### Video
- MPG, MPEG, AVI, WMV, MP4, WebM, M4V (requires ffmpeg)

## Feature Comparison

| Feature | LR2 | beatoraja | brs |
|---------|-----|-----------|-----|
| Platform | Windows | Cross-platform | Cross-platform |
| Judgment System | LR2 | beatoraja | Both (switchable) |
| Gauge Auto Shift | - | Yes | Yes |
| EX-HARD Gauge | - | Yes | Yes |
| Video BGA | Partial | Yes | Yes |
| IIDX Controller | Yes | Yes | Yes |

See [docs/brs-comparison.md](./docs/brs-comparison.md) for detailed comparison.

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

### Third-Party Licenses

This project uses the following fonts:
- **Noto Sans JP** - Licensed under the SIL Open Font License 1.1. See [assets/fonts/OFL.txt](./assets/fonts/OFL.txt).
