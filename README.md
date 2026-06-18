# Tincta V2

[![CI](https://github.com/sindus/tincta-V2/actions/workflows/ci.yml/badge.svg)](https://github.com/sindus/tincta-V2/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast, stable, cross-platform text editor for **macOS** and **Linux**, inspired by [Tincta for macOS](https://github.com/CodingFriends/Tincta). Built in Rust with [iced](https://github.com/iced-rs/iced).

**[Website & Downloads](https://sindus.github.io/tincta-V2)**

---

## Features

- **Syntax highlighting** for 60+ languages (TextMate grammars)
- **File sidebar** — open and switch between multiple files
- **Search & Replace** with regex support
- **Dark / Light theme**
- **Internationalisation** — English and French
- Configurable: font size, tab width, word wrap, auto-indent, bracket/quote completion
- Native binaries — macOS (Apple Silicon) and Linux (x86_64)

---

## Installation

### macOS — Homebrew

```bash
brew tap sindus/tincta
brew install tincta-v2
```

### Ubuntu / Debian — .deb

```bash
wget https://github.com/sindus/tincta-V2/releases/latest/download/tincta_0.5.1-1_amd64.deb
sudo dpkg -i tincta_0.5.1-1_amd64.deb
```

### Build from source

```bash
# Prerequisites (Ubuntu)
sudo apt-get install libgtk-3-dev libxkbcommon-dev

# Build
cargo build --release
./target/release/tincta
```

---

## Development

```bash
# Run
cargo run

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Tests
cargo test
```

---

## License

MIT — see [LICENSE](LICENSE).

Inspired by [Tincta](https://github.com/CodingFriends/Tincta) by CodingFriends.
