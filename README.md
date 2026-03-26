# RecWay

[![CI](https://github.com/nabiko02/recway/actions/workflows/ci.yml/badge.svg)](https://github.com/nabiko02/recway/actions/workflows/ci.yml)
[![Release](https://github.com/nabiko02/recway/actions/workflows/release.yml/badge.svg)](https://github.com/nabiko02/recway/actions/workflows/release.yml)
[![AUR version](https://img.shields.io/aur/version/recway)](https://aur.archlinux.org/packages/recway/)

Fork of [ali205412/wf-recorder-gui](https://github.com/ali205412/wf-recorder-gui).

A frontend for [wf-recorder](https://github.com/ammen99/wf-recorder), the Wayland screen recorder. Built with Rust and [iced](https://github.com/iced-rs/iced), featuring a glass-morphism dark theme, responsive layout, and a non-intrusive compact overlay during recording.

## Features

- **Display picker** — select which monitor to record (with position labels on multi-monitor setups)
- **Capture modes** — full screen or interactive region selection via `slurp`
- **Framerate** — 24 / 30 / 60 FPS
- **Audio** — system audio, microphone, both, or none
- **Output formats** — WebM, MP4, MKV
- **Persistent settings** — all options saved automatically
- **Error reporting** — wf-recorder errors surfaced in a floating popup; invalid region selection (overlapping displays) detected before recording starts

## Requirements

Runtime dependencies:

| Tool | Purpose |
|------|---------|
| `wf-recorder` | Core screen recording |
| `wlr-randr` | Display geometry and position detection |
| `slurp` | Interactive region selection (Region mode only) |
| `pactl` | Audio source detection |

### Installing dependencies

**Arch Linux**
```bash
sudo pacman -S wf-recorder wlr-randr slurp libpulse
```

## Installation

### Arch Linux (AUR)

> **Note:** The AUR package is not yet available.

```bash
yay -S recway
# or
paru -S recway
```

### Pre-built binary (all distros)

Download the latest `recway-x.x.x-x86_64-linux.tar.gz` from the [Releases](https://github.com/nabiko02/recway/releases) page, extract and place the binary in your `PATH`:

```bash
tar -xzf recway-*.tar.gz
sudo install -Dm755 recway /usr/local/bin/recway
```

### Build from source

```bash
# Install build dependencies (Arch)
sudo pacman -S rust libxkbcommon wayland

git clone https://github.com/nabiko02/recway.git
cd recway
cargo build --release
sudo install -Dm755 target/release/recway /usr/bin/recway
sudo install -Dm644 recway.desktop /usr/share/applications/recway.desktop
```

## Usage

1. Launch the application
2. Configure your recording:
   - **Capture mode** — Screen (pick a display) or Region (draw with `slurp`)
   - **Framerate** — 24 / 30 / 60 FPS
   - **Audio source** — System, Micro, both, or neither
   - **Output format** — WebM / MP4 / MKV
   - **Save location** — browse or type a path
3. Click **Start Recording** — a 3-second countdown appears, then the compact overlay
4. Click **Stop** in the overlay when finished

## Development

### Project structure

```
src/
├── audio/           # Audio source detection
├── config/          # Persistent JSON config (~/.config/recway/config.json)
├── recorder/        # wf-recorder subprocess, argument building, geometry validation
├── theme.rs         # Glass-morphism design system, responsive scaling
└── main.rs          # App state machine, iced update/view
```

### Commands

```bash
cargo build                          # debug build
cargo build --release                # release build (stripped)
cargo run                            # run
cargo test --verbose                 # tests
cargo fmt --all -- --check           # format check
cargo clippy -- -D warnings          # lint
cargo audit                          # security audit
```

### CI/CD

| Workflow | Trigger | What it does |
|----------|---------|--------------|
| `ci.yml` | push / PR to `main` | build, test, fmt, clippy, audit |
| `release.yml` | push `v*` tag | build binary + Arch `.pkg.tar.zst`, create GitHub release |
| `publish-aur.yml` | push `v*` tag | update AUR package |

## License

MIT — see [LICENSE](LICENSE).

## Acknowledgments

- [wf-recorder](https://github.com/ammen99/wf-recorder) — the underlying recording engine
- [iced](https://github.com/iced-rs/iced) — Rust GUI framework
- [slurp](https://github.com/emersion/slurp) — Wayland region selection
