# Kodama - JUCE 8 WebView VST Demo Project

## Overview

| Property | Value |
|----------|-------|
| **Project Name** | Kodama |
| **Type** | Delay Effect VST Plugin |
| **Frontend** | Vite + Vue 3 + TypeScript |
| **Backend** | JUCE 8 (C++20) |
| **Target Platform** | macOS (AU, VST3, Standalone) |
| **Reference** | https://github.com/yuichkun/my-first-juce-with-webview |

---

## Naming Conventions

### Plugin Identifiers

| Context | Value |
|---------|-------|
| **Product Name** | Kodama |
| **Company Name** | KodamaAudio |
| **Manufacturer Code** | Koda (4 chars) |
| **Plugin Code** | Kdma (4 chars) |
| **Bundle ID** | com.kodama-audio.kodama |

### Code Namespaces & Classes

| Context | Name |
|---------|------|
| **C++ Namespace** | `kodama` |
| **Processor Class** | `KodamaProcessor` |
| **Editor Class** | `KodamaEditor` |
| **CMake Target** | `Kodama` |

### File/Directory Names

| Context | Name |
|---------|------|
| **Root Directory** | `kodama/` |
| **Plugin Source Dir** | `plugin/` |
| **Frontend Dir** | `ui/` |
| **Include Dir** | `plugin/include/Kodama/` |

---

## Parameters (3 total)

| ID | Name | Range | Default | Unit | Description |
|----|------|-------|---------|------|-------------|
| `delayTime` | Delay Time | 0 - 2000 | 300 | ms | Delay time in milliseconds |
| `feedback` | Feedback | 0 - 100 | 30 | % | Amount of signal fed back |
| `mix` | Dry/Wet | 0 - 100 | 50 | % | Dry/wet mix ratio |

---

## Project Structure

```
kodama/
├── CMakeLists.txt                      # Root CMake
├── .gitmodules                         # JUCE submodule
├── .gitignore
├── README.md
│
├── libs/
│   └── juce/                           # JUCE 8 (git submodule)
│
├── plugin/
│   ├── CMakeLists.txt                  # Plugin CMake config
│   ├── include/
│   │   └── Kodama/
│   │       ├── PluginProcessor.h       # KodamaProcessor
│   │       └── PluginEditor.h          # KodamaEditor
│   └── src/
│       ├── PluginProcessor.cpp         # Audio processing + delay
│       └── PluginEditor.cpp            # WebView integration
│
└── ui/                                 # Vite + Vue 3 + TypeScript
    ├── package.json
    ├── pnpm-lock.yaml
    ├── vite.config.ts
    ├── tsconfig.json
    ├── tsconfig.node.json
    ├── index.html                      # Entry HTML
    │
    ├── public/
    │   └── js/
    │       └── juce/                   # JUCE JS SDK (copied by CMake)
    │
    └── src/
        ├── main.ts                     # Vue app entry
        ├── App.vue                     # Root component
        ├── style.css                   # Global styles
        │
        ├── components/
        │   ├── KnobControl.vue         # Rotary knob component
        │   └── PluginHeader.vue        # Plugin title/branding
        │
        ├── composables/
        │   └── useJuceParameter.ts     # JUCE parameter binding hook
        │
        └── types/
            └── juce.d.ts               # JUCE SDK TypeScript declarations
```

---

## Implementation Tasks

### Phase 1: Project Scaffolding

- [ ] **1.1** Create root directory structure
- [ ] **1.2** Initialize git repository
- [ ] **1.3** Add JUCE 8 as git submodule (`libs/juce`)
- [ ] **1.4** Create root `CMakeLists.txt`
- [ ] **1.5** Create `.gitignore` (CMake build, node_modules, etc.)

### Phase 2: C++ Plugin Core

- [ ] **2.1** Create `plugin/CMakeLists.txt` with:
  - `juce_add_plugin()` configuration
  - `JUCE_WEB_BROWSER=1` definition
  - JUCE JS SDK copy command
  - BinaryData for production UI
- [ ] **2.2** Create `PluginProcessor.h` / `.cpp`:
  - `AudioProcessorValueTreeState` with 3 parameters
  - Simple delay line implementation (circular buffer)
  - State save/load
- [ ] **2.3** Create `PluginEditor.h` / `.cpp`:
  - `WebBrowserComponent` with options
  - Resource provider (dev: file system, prod: BinaryData)
  - `WebSliderRelay` for each parameter
  - `WebSliderParameterAttachment` bindings

### Phase 3: Frontend (Vite + Vue 3 + TypeScript)

- [ ] **3.1** Initialize Vite project with Vue + TypeScript
- [ ] **3.2** Install dependencies:
  - `vue`, `vite`, `typescript`
  - `@vitejs/plugin-vue`
  - `vite-plugin-singlefile`
- [ ] **3.3** Configure `vite.config.ts`:
  - Single-file build for production
  - Dev server proxy if needed
- [ ] **3.4** Create TypeScript types for JUCE SDK (`juce.d.ts`)
- [ ] **3.5** Create `useJuceParameter.ts` composable:
  - Connect to `window.__JUCE__.getSliderState()`
  - Reactive value binding
- [ ] **3.6** Create `KnobControl.vue`:
  - SVG-based rotary knob
  - Mouse drag interaction
  - Value display
- [ ] **3.7** Create `App.vue`:
  - 3 knobs for delay parameters
  - Plugin branding header
  - Dark theme styling

### Phase 4: Build Integration

- [ ] **4.1** Add npm scripts:
  - `dev`: Vite dev server
  - `build`: Production build (singlefile)
- [ ] **4.2** Update CMake to:
  - Run `pnpm build` before plugin build
  - Include built `index.html` in BinaryData
- [ ] **4.3** Configure resource provider:
  - Dev mode: Load from file system (hot reload)
  - Prod mode: Load from BinaryData

### Phase 5: Testing & Polish

- [ ] **5.1** Test in DAW (standalone first, then AU/VST3)
- [ ] **5.2** Verify parameter automation works
- [ ] **5.3** Test UI hot reload in dev mode
- [ ] **5.4** Create README with build instructions

---

## Technical Details

### CMake Configuration (plugin/CMakeLists.txt)

```cmake
juce_add_plugin(Kodama
    COMPANY_NAME "KodamaAudio"
    PLUGIN_MANUFACTURER_CODE Koda
    PLUGIN_CODE Kdma
    FORMATS AU VST3 Standalone
    PRODUCT_NAME "Kodama"
)

target_compile_definitions(Kodama
    PUBLIC
        JUCE_WEB_BROWSER=1
        JUCE_USE_CURL=0
        JUCE_VST3_CAN_REPLACE_VST2=0
)
```

### WebBrowserComponent Setup Pattern

```cpp
// In KodamaEditor constructor
webView = std::make_unique<juce::WebBrowserComponent>(
    juce::WebBrowserComponent::Options{}
        .withNativeIntegrationEnabled()
        .withResourceProvider([this](const auto& url) { 
            return getResource(url); 
        })
        .withKeepPageLoadedWhenBrowserIsHidden()
        .withOptionsFrom(*delayTimeRelay)
        .withOptionsFrom(*feedbackRelay)
        .withOptionsFrom(*mixRelay)
);
```

### Vue Parameter Binding Pattern

```typescript
// useJuceParameter.ts
export function useJuceParameter(name: string) {
  const value = ref(0)
  const properties = ref({ start: 0, end: 1, skew: 1 })
  
  onMounted(() => {
    const slider = window.__JUCE__.getSliderState(name)
    value.value = slider.getNormalisedValue()
    properties.value = slider.properties
    
    slider.valueChangedCallback = (newValue: number) => {
      value.value = newValue
    }
  })
  
  const setValue = (newValue: number) => {
    window.__JUCE__.getSliderState(name).setNormalisedValue(newValue)
  }
  
  return { value, properties, setValue }
}
```

---

## Build Commands

```bash
# Development (frontend hot reload)
cd ui && pnpm dev

# Production build
cd ui && pnpm build

# CMake configure
cmake -B build -DCMAKE_BUILD_TYPE=Release

# Build plugin
cmake --build build --config Release
```

---

## Success Criteria

1. Plugin loads in standalone mode
2. 3 parameters visible and controllable via WebView UI
3. Delay effect audible and parameters affect sound
4. Hot reload works in development mode
5. Production build bundles UI into single binary

---

## References

- [Reference Repo](https://github.com/yuichkun/my-first-juce-with-webview)
- [JUCE WebBrowserComponent Docs](https://docs.juce.com/master/classWebBrowserComponent.html)
- [JUCE WebViewPluginDemo](https://github.com/juce-framework/JUCE/blob/master/examples/Plugins/WebViewPluginDemo.h)
- [vite-plugin-singlefile](https://github.com/richardtallent/vite-plugin-singlefile)
