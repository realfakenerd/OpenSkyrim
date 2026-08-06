# OpenSkyrim Architecture & Roadmap

An open-source engine reimplementation for **The Elder Scrolls V: Skyrim (Special Edition)** (and extensible for Creation Engine / Gamebryo titles like Morrowind / Oblivion) built in **Rust** using the **Bevy Engine**.

Inspired by project initiatives like OpenMW (OpenMorrowind), OpenSkyrim aims to provide a modern, multithreaded, high-performance runtime for Bethesda engine assets.

---

## 1. Core Architecture Overview

```
                          ┌───────────────────────────┐
                          │   Original Skyrim Assets  │
                          │ (.esm, .esp, .bsa, .nif)  │
                          └─────────────┬─────────────┘
                                        │
                                        ▼ (Offline Transpilation Pipeline)
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Modern Converted Asset Storage                           │
│  ┌──────────────────────────┬──────────────────────┬─────────────────────┐  │
│  │   glTF 2.0 (.glb)        │   KTX2 Textures      │   SQLite 3 Database │  │
│  │   (mesh_tools Exporter)  │   (basis-universal)  │   (R-Tree + rkyv)   │  │
│  └──────────────────────────┴──────────────────────┴─────────────────────┘  │
└────────────────────────────────────────┬────────────────────────────────────┘
                                         │ (Instant Memory Mapped / Multi-threaded Streaming)
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    OpenSkyrim Bevy Engine Runtime (ECS)                     │
│  ┌──────────────────────────┬──────────────────────┬─────────────────────┐  │
│  │   Vercidium Instanced    │   WebGPU / Vulkan    │   Luau (mlua) JIT   │  │
│  │   Indirect Render Pool   │   Bevy 0.19 bsn! UI  │   (Web API + Sandbox)│  │
│  └──────────────────────────┴──────────────────────┴─────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Workspace Crate Architecture

To keep compilation fast, binary sizes small (especially for mobile Android APKs), and code ergonomics clean, OpenSkyrim is organized into a **3-crate Cargo Workspace**:

```
OpenSkyrim/
├── Cargo.toml                  # Workspace Root Manifest
├── crates/
│   ├── openskyrim_launcher/    # UI App (Setup Wizard, Mod Manager, Launcher)
│   ├── openskyrim_converter/   # Converter Pipeline (.nif ➔ .glb, .dds ➔ KTX2, .esm ➔ libSQL, .pex ➔ Luau)
│   └── openskyrim_engine/      # Bevy Game Engine Runtime (Render, Physics, Luau, Audio)
```

| Crate Name | Binary Type | Key Responsibilities |
| :--- | :--- | :--- |
| **`openskyrim_launcher`** | GUI Executable | First-run setup wizard, game path detection, built-in mod manager UI, triggers converter progress bar, and launches the engine. |
| **`openskyrim_converter`** | Library & CLI | Heavy transformation logic (`mesh_tools`, `basis-universal`, `ddsfile`, `nom` parsers). Invoked directly by `openskyrim_launcher`. |
| **`openskyrim_engine`** | Game Executable | Light, hyper-fast game binary (Bevy 0.19+, `wgpu`, `libsql`, `mlua` Luau JIT). Runs the transformed game assets at 60+ FPS. |

---

## 3. Implementation Phases & Milestones

### Phase 1: Pipeline & Converter Foundation
- [ ] Implement `.nif` to `.glb` converter crate (`mesh_tools`).
- [ ] Implement `.dds` to `.ktx2` texture encoder crate (`ddsfile` + `basis-universal`).
- [ ] Implement `.esm` to `SQLite3` database parser with R-Tree spatial indexing (`rusqlite`).
- [ ] Implement `.pex` bytecode decompiler and Papyrus-to-Luau transpiler.

### Phase 2: World & Vercidium Rendering Core
- [ ] Set up Bevy 0.19 ECS scene loading pipeline with `bsn!` macro integration.
- [ ] Implement Vercidium-style GPU instanced indirect rendering pool (`DrawMeshInstancedIndirect`).
- [ ] Multi-threaded spatial cell streaming with zero-copy `rkyv` heightmap rendering.

### Phase 3: Luau Scripting, UI & Modding Layer
- [ ] Bind Luau engine (`mlua`) to Bevy ECS entity component systems.
- [ ] Implement Built-in Launcher Mod Manager with priority-weighted SQLite load order overriding.
- [ ] Transpile Flash `.gfx` UI templates to Bevy `bsn!` native UI nodes.
- [ ] Expose async HTTP networking APIs to Luau scripts (Live Weather sync, online features).

### Phase 4: Cross-Platform & Optimizations
- [ ] Target builds for Windows, Linux, macOS (Apple Silicon), and Android (POCO F5 / ARM64).
- [ ] Integrate VR (OpenXR) and Vulkan/WebGPU hardware ray-tracing pipeline.

---

## 4. Technology Stack & Tools

- **Language:** Rust (2024 Edition)
- **Engine Framework:** [Bevy](https://bevyengine.org/) (v0.19+)
- **Graphics Backend:** `wgpu` (Vulkan / Metal / DirectX 12 / WebGPU)
- **Scripting Engine:** Luau (`mlua` crate with `luau-jit` feature)
- **3D Exporter:** `mesh_tools` (`GltfBuilder`)
- **Texture Encoder:** `basis-universal` + `ddsfile`
- **Database & Cache:** SQLite 3 (`rusqlite` + `rtree`) + `rkyv` zero-copy memory mapping
- **UI Engine:** Bevy 0.19 `bsn!` (Bevy Scene Notation) declarative macro trees
