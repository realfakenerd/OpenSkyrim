# Asset & Data Modernization Pipeline Strategy

This document outlines the conversion pipeline to ingest legacy Skyrim formats (`.bsa`, `.dds`, `.nif`, `.hkx`, `.esm`) and output modern asset standards suitable for modern web & native runtimes (Bevy Engine, glTF 2.0, KTX2, WebGPU).

---

## 1. Pipeline Overview Diagram

```
┌───────────────────────────────┐
│     Legacy Skyrim Data        │
│  (.bsa, .dds, .nif, .hkx)     │
└───────────────┬───────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       OpenSkyrim Converter Pipeline                         │
│                                                                             │
│   ┌────────────────-┐     ┌────────────────┐     ┌──────────────────────┐   │
│   │ Archive Unpacker│ ──► │ Mesh & Texture │ ──► │ Record / World Data  │   │
│   │ (BSA / BA2)     │     │ Converter      │     │ Serializer           │   │
│   └────────────────-┘     └────────────────┘     └──────────────────────┘   │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Modern Asset Output Formats                             │
│  ┌──────────────────────────┬──────────────────────┬─────────────────────┐  │
│  │   glTF 2.0 / glb         │   KTX2 / Basis       │   JSON / BSON / Ron │  │
│  │   (Meshes, PBR, Skeleton)│   (GPU Textures)     │(Cell & Scene Graphs)│  │
│  └──────────────────────────┴──────────────────────┴─────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Format Conversion Matrix

| Legacy Format        | Target Modern Format                         | Purpose                                                                | Conversion Tool / Library          |
| :--- | :--- | :--- | :--- |
| **`.bsa` / `.ba2`** | Directory Hierarchy / VFS | Extracted compressed virtual filesystem | Rust (`flate2`, `lz4_flex`) |
| **`.nif`** | **`glTF 2.0` (`.glb`)** | Standard 3D geometry, mesh hierarchies, PBR materials, skinning | `nif-parser` ➔ `gltf` crate |
| **`.dds` (BC1-BC7)** | **`KTX2` / Supercompressed Basis Universal** | Fast GPU texture streaming & compressed VRAM footprints | `image` crate / `basis-universal` |
| **`.hkx` (Havok)** | **glTF Animations & Rapier 3D Colliders** | Skeletal animation clips & physics collision meshes | Havok XML exporter / custom parser |
| **`.esm` / `.esp`** | **`SQLite3` / Binary (Bincode / rkyv)** | High-performance spatial indexing, memory-mapped zero-copy queries | `rusqlite` / `rkyv` / `zerocopy` |
| **`.pex` / `.psc` (Papyrus)** | **`Lua 5.4` Scripts** | High-performance, sandboxed scripting engine | `mlua` / Papyrus-to-Lua AST Transpiler |

---

## 3. Detailed Conversion Modules

### A. Mesh Converter (`.nif` ➔ `.gltf` / `.glb`)

- **Geometry Conversion:**
  - Extract `BSTriShape` vertex buffers (Positions, Normals, UVs, Tangents).
  - Combine multi-part meshes into single glTF primitives.
- **Material Mapping:**
  - Map Skyrim `BSLightingShaderProperty` texture slots:
    - Diffuse Map ➔ glTF `baseColorTexture`
    - Normal Map ➔ glTF `normalTexture`
    - Specular Map / Environment ➔ glTF `metallicRoughnessTexture`
- **Skinning & Rigging:**
  - Map `NiSkinInstance` & `NiSkinData` bone weights directly to glTF `skins` and `joints`.

### B. Texture Transcoder (`.dds` ➔ `.ktx2`)

- Converts DirectDraw Surface files into supercompressed **Basis Universal KTX2** textures.
- Supports runtime transcoding into WebGPU/Vulkan native compressed formats (BC1/BC7 for Desktop, ASTC/ETC2 for Mobile/Web).

### C. World & Scene Database (`.esm` ➔ `SQLite3` + `rkyv` Zero-Copy Cache)
* **SQLite 3 Database (`skyrim_world.db`):**
  * **Spatial R-Tree Indexing (`rtree` module):** Query cell objects instantly using 3D Bounding Boxes (`X, Y, Z` coordinates) based on player camera position.
  * **Relational FormID Lookup:** Fast `O(1)` index table mapping 32-bit `FormID` ➔ Record payload.
  * **Tables:** `cells` (Grid coords, lighting, worldspace), `references` (Placed glTF model URIs, transforms, flags), `npcs` (Stats, race, dialogue trees).

* **Zero-Copy Hot Storage (`rkyv` / `zerocopy`):**
  * Frequently requested terrain meshes and cell data can be serialized into binary files mapped directly into RAM with `mmap`.
  * Eliminates deserialization CPU overhead during fast player movement across Skyrim's terrain.

### D. Scripting Engine (`.pex` Papyrus Bytecode ➔ `Lua 5.4` Scripts)
* **Papyrus Decompiler / Transpiler:**
  * Parse `.pex` binary bytecode or decompiled `.psc` source code into an Abstract Syntax Tree (AST).
  * Translate Papyrus features (Events, States, Properties, Native functions) directly to Lua equivalent tables and functions.
* **Lua Runtime Integration (`mlua` crate):**
  * Lua 5.4 / Luau provides extreme execution speed (near C/Rust speed with JIT or optimized bytecode).
  * Expose game engine API bindings (e.g. `Game.getPlayer()`, `Actor.addItem()`, `ObjectReference.enable()`) to Lua via `mlua`.
* **State Preservation & Save Games:**
  * Modern Lua state serialization enables light, fast save-game state snapshots without Papyrus VM thread corruption.

