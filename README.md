# Kodama

A digital delay VST plugin with a modern web-based UI built using JUCE 8 WebView and Vue 3.

## Features

- Simple digital delay effect with 3 parameters:
  - **Delay Time**: 0-2000ms
  - **Feedback**: 0-100%
  - **Mix**: 0-100% (Dry/Wet)
- Modern UI built with Vue 3 + TypeScript + Tailwind CSS
- Real-time parameter synchronization via JUCE WebSliderRelay

## Requirements

- macOS 10.15+
- CMake 3.22+
- Xcode Command Line Tools
- Node.js 18+

## Project Structure

```
kodama/
├── libs/juce/          # JUCE 8 (git submodule)
├── plugin/             # C++ plugin source
│   ├── include/Kodama/
│   │   ├── PluginProcessor.h
│   │   └── PluginEditor.h
│   └── src/
│       ├── PluginProcessor.cpp
│       └── PluginEditor.cpp
└── ui/                 # Vue 3 frontend
    ├── src/
    │   ├── components/
    │   ├── composables/
    │   └── types/
    └── dist/           # Built UI (generated)
```

## Build Instructions

### 1. Clone with submodules

```bash
git clone --recursive https://github.com/your-username/kodama.git
cd kodama
```

Or if already cloned:

```bash
git submodule update --init --recursive
```

### 2. Build the UI

```bash
cd ui
npm install
npm run build
cd ..
```

### 3. Build the plugin

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release
```

### 4. Find the built plugins

After building, plugins will be located at:
- **AU**: `build/plugin/Kodama_artefacts/Release/AU/Kodama.component`
- **VST3**: `build/plugin/Kodama_artefacts/Release/VST3/Kodama.vst3`
- **Standalone**: `build/plugin/Kodama_artefacts/Release/Standalone/Kodama.app`

## Development

### Frontend hot reload

For UI development with hot reload:

```bash
cd ui
npm run dev
```

The plugin will load UI from the `ui/dist/` directory. Rebuild UI when making changes.

### Debug build

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build --config Debug
```

## Tech Stack

- **Audio**: JUCE 8
- **UI Framework**: Vue 3 + TypeScript
- **Styling**: Tailwind CSS
- **Build**: Vite + vite-plugin-singlefile
- **C++ <-> JS**: JUCE WebBrowserComponent + WebSliderRelay

## License

MIT
