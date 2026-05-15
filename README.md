<div align="center">

<img src="https://raw.githubusercontent.com/TS-DEV-DEBUG-V2/rustyboy/refs/heads/main/assets/rustyboy.png" width="200">

# RustyBoy

**A Game Boy DMG Emulator written in PURE Rust**

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/TS-DEV-DEBUG-V2/rustyboy)
[![Emscripten](https://img.shields.io/badge/Emscripten-F5D442?style=for-the-badge&logo=webassembly&logoColor=black)](https://emscripten.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue?style=for-the-badge)](https://github.com/TS-DEV-DEBUG-V2/rustyboy/blob/main/LICENSE)

---

![GitHub repo size](https://img.shields.io/github/repo-size/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue)
![GitHub code size](https://img.shields.io/github/languages/code-size/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue)
![GitHub stars](https://img.shields.io/github/stars/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=yellow)
![GitHub forks](https://img.shields.io/github/forks/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=green)
![GitHub watchers](https://img.shields.io/github/watchers/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=orange)
![GitHub issues](https://img.shields.io/github/issues/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=red)
![GitHub closed issues](https://img.shields.io/github/issues-closed/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=brightgreen)
![GitHub pull requests](https://img.shields.io/github/issues-pr/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue)
![GitHub closed PRs](https://img.shields.io/github/issues-pr-closed/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=brightgreen)
![GitHub last commit](https://img.shields.io/github/last-commit/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=purple)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue)
![GitHub contributors](https://img.shields.io/github/contributors/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=green)
![GitHub top language](https://img.shields.io/github/languages/top/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=orange)
![GitHub language count](https://img.shields.io/github/languages/count/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue)
![GitHub release](https://img.shields.io/github/v/release/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue&include_prereleases)
![GitHub tag](https://img.shields.io/github/v/tag/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=teal)
![GitHub created at](https://img.shields.io/github/created-at/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue)
![GitHub discussions](https://img.shields.io/github/discussions/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=purple)
![GitHub license](https://img.shields.io/github/license/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/TS-DEV-DEBUG-V2/rustyboy/rust.yml?style=flat-square&label=CI)
![GitHub deployments](https://img.shields.io/github/deployments/TS-DEV-DEBUG-V2/rustyboy/production?style=flat-square&color=brightgreen)
![GitHub manifest version](https://img.shields.io/github/manifest-json/v/TS-DEV-DEBUG-V2/rustyboy?style=flat-square&color=blue)
![Rust Version](https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust)
![WebAssembly](https://img.shields.io/badge/wasm-supported-624DE5?style=flat-square&logo=webassembly)
![Platform Windows](https://img.shields.io/badge/platform-windows-0078D6?style=flat-square&logo=windows)
![Platform Web](https://img.shields.io/badge/platform-web_(emscripten)-F5D442?style=flat-square&logo=googlechrome)
![Architecture x86_64](https://img.shields.io/badge/arch-x86__64-informational?style=flat-square)
![Architecture WASM32](https://img.shields.io/badge/arch-wasm32-informational?style=flat-square)
![No Unsafe](https://img.shields.io/badge/unsafe-forbidden-success?style=flat-square)
![Zero Dependencies](https://img.shields.io/badge/deps-minimal-brightgreen?style=flat-square)
![Game Boy DMG](https://img.shields.io/badge/system-Game_Boy_DMG-8bac0f?style=flat-square)
![CPU](https://img.shields.io/badge/cpu-Sharp_LR35902-lightgrey?style=flat-square)
![PPU](https://img.shields.io/badge/ppu-emulated-blue?style=flat-square)
![APU](https://img.shields.io/badge/apu-emulated-blue?style=flat-square)
![MBC1](https://img.shields.io/badge/mapper-MBC1-9cf?style=flat-square)
![MBC2](https://img.shields.io/badge/mapper-MBC2-9cf?style=flat-square)
![MBC3](https://img.shields.io/badge/mapper-MBC3-9cf?style=flat-square)
![MBC5](https://img.shields.io/badge/mapper-MBC5-9cf?style=flat-square)
![ROM Only](https://img.shields.io/badge/mapper-ROM_Only-9cf?style=flat-square)
![Resolution](https://img.shields.io/badge/resolution-160x144-informational?style=flat-square)
![Sprite Limit](https://img.shields.io/badge/sprites-10_per_line-informational?style=flat-square)
![Timer](https://img.shields.io/badge/timer-emulated-blue?style=flat-square)
![Interrupts](https://img.shields.io/badge/interrupts-emulated-blue?style=flat-square)
![Joypad](https://img.shields.io/badge/joypad-emulated-blue?style=flat-square)
![Serial](https://img.shields.io/badge/serial-stubbed-lightgrey?style=flat-square)
![Save States](https://img.shields.io/badge/save_states-supported-brightgreen?style=flat-square)
![Battery Saves](https://img.shields.io/badge/battery_saves-supported-brightgreen?style=flat-square)
![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square)
![Made with Love](https://img.shields.io/badge/made_with-❤️-red?style=flat-square)

</div>

---

## Features

- **Accurate DMG Emulation** — Sharp LR35902 CPU with full instruction set and cycle-accurate timing
- **Complete PPU** — Background, window, and sprite rendering at native 160×144 resolution
- **Audio Processing Unit** — All four sound channels emulated (pulse, wave, noise)
- **Memory Bank Controllers** — Support for ROM Only, MBC1, MBC2, MBC3, and MBC5 cartridge types
- **Battery Saves** — Persistent save RAM for supported cartridges
- **Timer & Interrupts** - Accurate timer divider and interrupt handling
- **Windows Native Build** - Standalone desktop executable via Cargo
- **Web Build via Emscripten** - Play directly in the browser through WebAssembly

---

## Building

### Windows (Native)

```bash
cargo build --release
```

### Web (Emscripten)

```bash
cargo build --release --target wasm32-unknown-emscripten
```

---

## Controls

| Game Boy | Keyboard |
|:--------:|:--------:|
| D-Pad | Arrow Keys |
| A | Z |
| B | X |
| Start | Enter |
| Select | Backspace |

---

## Development

Contributions are welcome — feel free to open issues or submit pull requests.

![Alt](https://repobeats.axiom.co/api/embed/3e621f3cf8b5c7a9a867575a933155a56335d6b4.svg "Repobeats analytics image")

---

## License

This project is licensed under the [Apache License 2.0](https://github.com/TS-DEV-DEBUG-V2/rustyboy/blob/main/LICENSE).
