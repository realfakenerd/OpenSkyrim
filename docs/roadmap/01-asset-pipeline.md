# Phase 1: Asset Modernization Pipeline (`converter`)

> **Goal:** Ingest legacy Bethesda Skyrim SE assets (`.bsa`, `.esm`, `.nif`, `.dds`, `.pex`) and compile them offline into modern, GPU-native standard formats (`SQLite 3`, `glTF 2.0`, `KTX2`, `Luau`).

---

## 📋 Overview & Conversion Matrix

| Legacy Format                | Target Modern Format               | Purpose                                                     | Conversion Library / Crate    |
| :--------------------------- | :--------------------------------- | :---------------------------------------------------------- | :---------------------------- |
| **`.bsa` / `.ba2`**          | Extracted VFS Hierarchy            | Unpacked file system access                                 | `flate2`, `lz4_flex`          |
| **`.esm` / `.esp` / `.esl`** | **`SQLite 3` (`skyrim_world.db`)** | Conflict-resolved record database & R-Tree spatial indexing | `rusqlite`, `libsql`          |
| **`.nif`**                   | **`glTF 2.0` (`.glb`)**            | Geometry, PBR materials, skeletal skinning & joints         | `mesh_tools` / `gltf` crate   |
| **`.dds` (BC1-BC7)**         | **`KTX2` Basis Universal**         | Supercompressed GPU texture streaming                       | `ddsfile`, `basis-universal`  |
| **`.pex` (Papyrus)**         | **`Luau` Scripts**                 | Sandboxed, high-performance scripting                       | Papyrus AST Decompiler ➔ Luau |

---

## 🎯 Phase 1 Modules & Specifications

### 1.1 BSA / BA2 Archive Extractor

- Extracted virtual filesystem (`vfs`) using async decompression threads.
- Preserves directory layout (`meshes/`, `textures/`, `scripts/`, `sound/`).

### 1.2 Relational World & Spatial Database Transpiler (`ESM_TO_SQLITE`)

- Parse legacy binary records (`CELL`, `REFR`, `LAND`, `NPC_`, `STAT`, `WRLD`).
- Merge plugin overrides in priority order defined by `plugins.txt` to eliminate runtime conflict resolution overhead.
- **Hybrid Spatial Indexing Architecture:**
  - **Exterior R-Tree (`exterior_spatial`):** Indexes exterior world references (`REFR`) with normalized coordinates relative to cell centers (values constrained between $-2048.0$ and $+2048.0$) to guarantee single-precision `float32` accuracy at large worldspace extremes.
  - **Interior Direct Lookup (`cell_id` Hash Index):** Indexes interior references by `cell_id` FormID for instant $O(1)$ object loading upon entering interior doors, avoiding R-Tree spatial collisions.
- **Zero-Copy Cache (`cell_cache.rkyv`):** Memory-mapped (`mmap`) binary heightmaps and vertex normals for instant RAM reads.

### 1.3 NIF Geometry & Material Exporter (`NIF_TO_GLTF`)

- Convert `BSTriShape` and `BSDynamicTriShape` into glTF primitives.
- Map `BSLightingShaderProperty` slots:
  - Diffuse ➔ glTF `baseColorTexture`
  - Normal ➔ glTF `normalTexture`
  - Specular/Environment ➔ glTF `metallicRoughnessTexture`
- Convert `NiSkinInstance` bone weights and transforms directly into glTF `skins` and `joints`.

### 1.4 Texture Compressor (`DDS_TO_KTX2`)

- Transcode DirectDraw Surface files into supercompressed **Basis Universal KTX2**.
- Target runtime transcoding for Desktop (BC7), Mobile (ASTC), and Web (ETC2).

### 1.5 Papyrus Bytecode Transpiler (`PEX_TO_LUA`)

- Decompile `.pex` binary bytecode into an Abstract Syntax Tree (AST).
- Transpile Papyrus Events, Properties, Functions, and State Machines directly into `Luau` modules.

### 1.6 Unified Async Pipeline Orchestration (`tokio`)

- **Async Task Concurrency:** Uses `tokio::spawn` for I/O tasks and `tokio::task::spawn_blocking` for CPU-intensive mesh/texture transcoding (`dds` ➔ `ktx2`, `nif` ➔ `glb`).
- **Async Progress Channel:** Emits real-time progress events over `tokio::sync::mpsc` channels to the Launcher GUI and CLI runners without thread blocking.
- **Deep High-Leverage API:** `AssetPipeline::run_async(config, progress_tx).await` orchestrates all sub-converters concurrently with atomic cache validation.
