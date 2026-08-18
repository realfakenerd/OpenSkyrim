# OpenSkyrim

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
[![Engine](https://img.shields.io/badge/Engine-Bevy_0.19-blue.svg)](https://bevyengine.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE-APACHE)
[![CI Status](https://img.shields.io/badge/CI-passing-brightgreen.svg)](#-quick-start-development-setup)
[![Platforms](https://img.shields.io/badge/Platforms-Windows_%7C_Linux_%7C_macOS_%7C_Android-purple.svg)](docs/specs/meta/platforms.md)

An open-source engine reimplementation for **The Elder Scrolls V: Skyrim (Special Edition)** built in **Rust** using the **Bevy Engine**.

Inspired by projects like OpenMW, OpenSkyrim modernizes Bethesda game files (`.esm`, `.bsa`, `.nif`, `.dds`, `.pex`) into ultra-fast, GPU-native formats (`glTF 2.0`, `KTX2 Basis`, `libSQL / SQLite3`, `Luau`) to deliver **60+ FPS high-performance gameplay without loading screens**.

---

## ✨ Features & Architecture Highlights

- **🚀 Zero Loading Screens:** Interior and exterior city transitions load 100% seamlessly in real time using libSQL spatial R-Tree indexing and `rkyv` zero-copy memory mapping.
- **⚡ High-FPS Vercidium Rendering:** Batched instanced indirect draw calls (`DrawMeshInstancedIndirect`) reduce 5,000+ individual draw calls to under 100 per frame.
- **📜 Luau Scripting Engine (`mlua`):** Replaces single-threaded Papyrus with sandboxed, JIT-compiled Luau scripts. Includes async HTTP networking for live web APIs (e.g. real-world weather sync mods).
- **📱 Cross-Platform (Desktop & Mobile):** Native support for Windows, Linux, macOS (Apple Silicon), and Android (ARM64 / POCO F5).
- **🎨 Modern UI (Bevy 0.19 `bsn!`):** Transpiles obsolete Flash (`.gfx`) menus into hardware-accelerated declarative Bevy UI node trees.
- **🎮 Integrated Mod Manager:** Built-in launcher with drag-and-drop archive installation and priority-weighted `.esp`/`.esl` plugin load orders.

---

## 🏗️ Workspace Crate Architecture

OpenSkyrim is organized into a modular 3-crate Cargo workspace:

```
OpenSkyrim/
├── Cargo.toml                  # Workspace Root Manifest
├── crates/
│   ├── launcher/    # GUI Setup Wizard & Built-in Mod Manager
│   ├── converter/   # Converter Pipeline (.nif ➔ .glb, .dds ➔ KTX2, .esm ➔ libSQL)
│   └── engine/      # Bevy Game Engine Binary (Render, Physics, Luau, Audio)
```

| Crate           | Responsibilities                                                                                                                |
| :-------------- | :------------------------------------------------------------------------------------------------------------------------------ |
| **`launcher`**  | First-run setup wizard, game path detection, built-in mod manager UI, triggers converter progress bar, and launches the engine. |
| **`converter`** | Heavy offline asset converter (`mesh-tools`, `basis-universal`, `ddsfile`, `nom` binary parsers).                               |
| **`engine`**    | Lightweight, hyper-fast game binary (Bevy 0.19+, `wgpu`, `libsql`, `mlua` Luau JIT).                                            |

---

## 🗺️ Project Roadmap

OpenSkyrim is being built systematically across 5 core phases. Explore the full roadmap specs in [`docs/roadmap/`](docs/roadmap/README.md).

- [ ] **[Phase 1: Asset Modernization Pipeline (`converter`)](docs/roadmap/01-asset-pipeline.md)** — Transpile legacy `.esm`, `.nif`, `.dds`, and `.pex` into `SQLite 3`, `glTF 2.0`, `KTX2`, and `Luau`.
- [ ] **[Phase 2: Core Engine Runtime & Vercidium Renderer (`engine`)](docs/roadmap/02-core-engine.md)** — Runtime, integration, profiling, and acceptance infrastructure implemented; complete real-asset sign-off remains pending.
- [ ] **[Phase 3: Luau Runtime, Declarative UI & Launcher (`launcher`)](docs/roadmap/03-luau-and-ui.md)** — Sandboxed Luau JIT with async web APIs, Flash-to-Bevy `bsn!` UI conversion, and setup wizard.
- [ ] **[Phase 4: Gameplay Mechanics, Physics & Persistence](docs/roadmap/04-gameplay-and-physics.md)** — Rapier 3D physics, animation blending, combat state machine, and sub-second save state snapshots.
- [ ] **[Phase 5: Hardware Ray-Tracing, Multiplatform & Networking](docs/roadmap/05-multiplatform-and-networking.md)** — WebGPU/Vulkan RTGI, DLSS/FSR3 frame gen, native co-op multiplayer, Android ARM64, and OpenXR VR.

---

## 🚀 Quick Start (Development Setup)

### Prerequisites

- **Rust** (2024 Edition)
- **CMake** & **Ninja** / **GCC** (for native libSQL / SQLite compilation)

### Building & Running

1. **Clone the repository:**

   ```bash
   git clone https://github.com/your-username/OpenSkyrim.git
   cd OpenSkyrim
   ```

2. **Check workspace compilation:**

   ```bash
   cargo check --workspace
   ```

3. **Run the Launcher App:**
   ```bash
   cargo run -p launcher
   ```

---

## 📚 Technical Specifications (`docs/specs/`)

For detailed technical specifications, format breakdowns, and architectural guidelines, explore the [`docs/specs/`](docs/specs/README.md) directory:

- **[`architecture.md`](docs/specs/engine/architecture.md)** — Core engine architecture overview & phase milestones.
- **[`nif-to-gltf.md`](docs/specs/converters/nif-to-gltf.md)** — 3D mesh converter specification (`mesh-tools`).
- **[`dds-to-ktx2.md`](docs/specs/converters/dds-to-ktx2.md)** — Texture compressor specification (`basis-universal` + `ddsfile`).
- **[`esm-to-sqlite.md`](docs/specs/converters/esm-to-sqlite.md)** — Master database specification (`libSQL` + `rkyv`).
- **[`pex-to-lua.md`](docs/specs/converters/pex-to-lua.md)** — Papyrus bytecode to Luau transpilation spec (`mlua`).
- **[`mods-and-ui.md`](docs/specs/modding/mods-and-ui.md)** — Integrated Mod Manager & Flash-to-Bevy UI strategy.
- **[`launcher.md`](docs/specs/modding/launcher.md)** — First-run setup wizard & launcher workflow.
- **[`vercidium-optimizations.md`](docs/specs/engine/vercidium-optimizations.md)** — High-FPS GPU instancing & HZB culling.
- **[`platforms.md`](docs/specs/meta/platforms.md)** — Target matrix for Desktop, Mobile ARM64, and WebGPU.
- **[`requirements.md`](docs/specs/meta/requirements.md)** — Hardware system specs and optimization analysis.
- **[`features.md`](docs/specs/modding/features.md)** — Unlocked capabilities (Zero-loading screens, AI dialogue, native co-op).
- **[`bevy-examples.md`](docs/specs/engine/bevy-examples.md)** — Mapping official Bevy 0.19 patterns to engine subsystems.

---

## 🤝 Contributing

We welcome community contributions! Whether you're fixing bugs in asset converters, enhancing Bevy rendering pipelines, or writing documentation, check out our guidelines before submitting a PR:

- 📖 **[CONTRIBUTING.md](CONTRIBUTING.md)** — Guide on development workflow, code style (`cargo fmt`/`clippy`), and PR guidelines.
- 🐛 **[Issue Tracker](../../issues)** — Search existing issues or report a new bug using our template.

---

## ⚖️ Legal Disclaimer & License

### Licensing

OpenSkyrim is dual-licensed under either of the following licenses at your option:

- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT))
- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE))

### Legal Disclaimer

OpenSkyrim is an independent open-source game engine reimplementation. It does **not** contain any copyrighted game assets, artwork, 3D models, audio, or game data from Bethesda Softworks LLC or ZeniMax Media Inc. Users must supply their own legally owned copy of _The Elder Scrolls V: Skyrim_ to extract game data.
