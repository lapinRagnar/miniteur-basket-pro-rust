# 🏀 Shot Clock Pro - Basketball Countdown Timer

[![Rust](https://img.shields.io/badge/built%20with-Rust-red)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Android%20%7C%20Windows%20%7C%20Linux%20%7C%20macOS-blue)]()
[![License](https://img.shields.io/badge/license-MIT-green)]()

> **A professional shot clock for basketball training and games. Voice countdown + loud siren. Build once, run everywhere.**

![Shot Clock Pro Demo](https://via.placeholder.com/800x400?text=Shot+Clock+Pro+Screenshot)

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🗣️ **Voice Countdown** | Announces each second audibly: "12... 11... 10..." |
| 🚨 **Loud Siren** | Play a powerful siren when time reaches zero |
| 🔄 **Loop Mode** | Automatically restart the countdown after a configurable break |
| ⚙️ **Customizable** | Set starting time, break duration, and loop behavior |
| 📱 **Cross-Platform** | Same code runs on Android phones and desktop computers |
| 💾 **Settings Persistence** | Your preferences are saved between sessions |

## 🎯 Use Cases

- **Basketball Training** - Perfect for shot clock drills (24 seconds, 14 seconds, or custom)
- **Game Simulation** - Create realistic pressure situations
- **Team Practice** - Time your offensive plays
- **Fitness Workouts** - Countdown intervals for exercises
- **Education** - Classroom timing activities

## 📱 Platform Support

| Platform | Status | Distribution |
|----------|--------|--------------|
| 🖥️ Windows | ✅ Full support | Standalone .exe |
| 🐧 Linux | ✅ Full support | Binary |
| 🍎 macOS | ✅ Full support | Binary |
| 📱 Android | ✅ Full support | APK |
| 🌐 Web | ✅ Full support | WASM |

## 🚀 Quick Start

### Download

#### Desktop (Windows / Linux / macOS)
```bash
# Download from releases
https://github.com/yourusername/shot-clock-pro/releases

# Or build from source
git clone https://github.com/yourusername/shot-clock-pro.git
cd shot-clock-pro
cargo build --release

# Download the APK from releases
shot-clock-pro.apk

# Install with adb (or manually on device)
adb install shot-clock-pro.apk


# First Launch
Open the app → You'll see 00:12 on screen

Press "Start" → The countdown begins

Hear the voice → "12... 11... 10..."

At zero → Loud siren triggers

Configure → Click "Settings" to customize

⚙️ Configuration Options
|Setting	   |Description                                  |Default	         |Range   |
|-----------------------------------------------------------------------------------------|
|Start Time	   |Initial countdown seconds	                 |12	             |1-3600  |
|Break Duration|Pause time between loops (seconds)           |5	                 |0 - 300 |
|Loop Mode	   |Auto-restart when enabled	                 |Off	             |On/Off  |
🎮 Usage Guide

## Main Controls

┌─────────────────────────────────────┐
│          00:12                      │ ← Current time display
│                                     │
│  [▶ Start]  [⏸ Pause]  [🔄 Reset]   │ ← Control buttons
│                                     │
│           ⏲ En cours...             │ ← Status indicator
└─────────────────────────────────────┘

## Keyboard Shortcuts (Desktop)
|Action	  |Shortcut|
|Start	  |Space|
|Pause	  |P|
|Reset	  |R|
|Settings |S|

# 🏗️ Technical Architecture

// Pure Rust stack
├── Dioxus        → Cross-platform UI (React-like)
├── Rodio         → High-performance audio playback
├── tts-rs        → Native text-to-speech
├── Tokio         → Async runtime for precise timing
└── Serde         → Persistent settings storage

# 🎵 Audio Features
### Voice Announcements

- Uses system TTS (Google Text-to-Speech on Android, OS voices on desktop)
- Clear pronunciation of numbers 1-999
- No internet connection required

### Siren System
- Priority: Custom .wav file → TTS fallback
- Plays at maximum system volume
- Loops automatically for attention-grabbing effect

# 📂 Project Structure
shot-clock-pro/
├── src/
│   ├── main.rs        # UI & app logic
│   ├── timer.rs       # Countdown engine
│   ├── audio.rs       # Sound management
│   └── settings.rs    # Configuration persistence
├── assets/
│   ├── styles.css     # Visual styling
│   └── sirene.wav     # Custom siren (optional)
├── Cargo.toml         # Dependencies & build config
└── README.md          # You are here

 # 🔧 Building from Source
## Prerequisites

```
# Rust (required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System dependencies
# Ubuntu/Debian
sudo apt install libasound2-dev pkg-config

# Android (optional)
rustup target add aarch64-linux-android
```

### Build Commands
```
# Desktop (Windows/Linux/macOS)
cargo build --release
./target/release/shot-clock-pro

# Android APK
cargo install cargo-apk
cargo apk build --release
cargo apk run

# Web (WASM)
cargo install dioxus-cli
dx serve --platform web

```

# 📊 Performance Metrics

# 🗺️ Roadmap
- Vibration - Haptic feedback on mobile
- Presets - 24s, 14s, 5s quick buttons
- Statistics - Track repetitions and accuracy
- Custom ringtones - Let users pick siren sounds
- Background service - Play when screen is off
- Widget - Android home screen widget
- Multi-language - Announce in different languages

# 📄 License
MIT License - Use it freely for personal or commercial projects.

# 🙏 Acknowledgments
- Built with Dioxus - Rust UI framework
- Audio powered by Rodio
- Voice synthesis via tts-rs