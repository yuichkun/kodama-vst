# Kodama

A digital delay plugin with shared Rust DSP core and dual runtime support (VST3/AU via JUCE, Web via WASM).

## Features

- Simple digital delay effect with 3 parameters:
  - **Delay Time**: 0-2000ms
  - **Feedback**: 0-100%
  - **Mix**: 0-100% (Dry/Wet)
- Modern UI built with Vue 3 + TypeScript + Tailwind CSS
- Real-time parameter synchronization via JUCE WebSliderRelay

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Shared UI (Vue 3)                      │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┴─────────────────────┐
        │                                           │
        ▼                                           ▼
┌───────────────────┐                   ┌───────────────────┐
│   Native Runtime  │                   │    Web Runtime    │
│  (JUCE + C++ FFI) │                   │  (WASM + Worklet) │
└───────────────────┘                   └───────────────────┘
        │                                           │
        └─────────────────────┬─────────────────────┘
                              │
                              ▼
                   ┌───────────────────┐
                   │   Rust DSP Core   │
                   │   (kodama-dsp)    │
                   └───────────────────┘
```

- **dsp/**: Rust DSP core (delay algorithm)
  - Native: `staticlib` → C FFI → JUCE plugin
  - Web: `cdylib` → WASM → AudioWorklet
- **plugin/**: JUCE C++ wrapper (VST3/AU)
- **ui/**: Vue 3 frontend (shared between both runtimes)

## Prerequisites

| Platform | Requirements |
|----------|-------------|
| All | Node.js 18+, Rust (`rustup`), CMake 3.22+ |
| macOS | macOS 10.15+, Xcode Command Line Tools |
| Linux | build-essential, libasound2-dev, libwebkit2gtk-4.1-dev |
| Windows | MSVC (Visual Studio 2022+) |

```bash
# Install Rust WASM target
rustup target add wasm32-unknown-unknown
```

## Quick Start

### Clone

```bash
git clone --recursive https://github.com/your-username/kodama.git
cd kodama
```

### Web Development

```bash
npm run setup:web   # Install deps + build WASM
npm run dev:web     # Start dev server (http://localhost:5173)
```

### JUCE Development

```bash
npm run setup:juce  # Install deps + build Debug plugin
npm run dev:juce    # Start Vite dev server with HMR
# Then open the plugin in DAW or standalone
```

## npm Scripts

| Script | Description |
|--------|-------------|
| `setup:web` | Install UI deps, build DSP to WASM |
| `setup:juce` | Install UI deps, build Debug plugin |
| `dev:web` | Start web dev server with HMR |
| `dev:juce` | Start Vite dev server with HMR (Debug plugin connects to localhost:5173) |
| `build:dsp` | Build Rust DSP to WASM |
| `release:web` | Production build for web deployment |
| `release:vst` | Production build for VST3/AU (Release) |

## Project Structure

```
kodama/
├── dsp/                # Rust DSP core
│   ├── src/
│   │   ├── lib.rs      # Library entry
│   │   ├── delay.rs    # Delay algorithm
│   │   ├── ffi.rs      # C FFI bindings
│   │   └── wasm.rs     # WASM bindings
│   └── Cargo.toml
├── libs/juce/          # JUCE 8 (git submodule)
├── plugin/             # JUCE C++ wrapper
│   ├── include/Kodama/
│   └── src/
└── ui/                 # Vue 3 frontend
    ├── src/
    │   ├── components/
    │   ├── composables/
    │   └── types/
    └── public/
        ├── wasm/       # Built WASM (generated)
        └── worklet/    # AudioWorklet processor
```

## Build Artifacts

| Target | Location |
|--------|----------|
| VST3 | `build/plugin/Kodama_artefacts/Release/VST3/Kodama.vst3` |
| AU | `build/plugin/Kodama_artefacts/Release/AU/Kodama.component` |
| Standalone | `build/plugin/Kodama_artefacts/Release/Standalone/Kodama.app` |
| Web | `ui/dist-web/` |

## Tech Stack

- **DSP**: Rust
- **Audio Plugin**: JUCE 8 (C++)
- **UI**: Vue 3 + TypeScript + Tailwind CSS
- **Build**: Vite, CMake, Cargo
- **Web Audio**: WASM + AudioWorklet

## License

MIT
