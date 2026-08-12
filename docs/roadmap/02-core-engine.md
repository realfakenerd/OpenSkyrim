# Phase 2: Core Engine Runtime & Vercidium Renderer (`engine`)

> **Goal:** Build the core Bevy 0.19+ engine runtime to achieve zero-loading-screen spatial streaming and high-efficiency GPU instanced rendering for massive Skyrim render distances.

---

## 🎯 Key Deliverables & Specifications

### 2.1 Bevy ECS Architecture & World Initialization

- 4-crate workspace design (`launcher`, `converter`, `scripting`, `engine`).
- Bevy 0.19 ECS components (`FormId`, `CellRef`, `WorldTransform`, `MeshHandle`, `MaterialHandle`).

### 2.2 Multi-Threaded Cell & Spatial Streaming

- Asynchronous background frustum queries against `skyrim_world.db` using the **Hybrid Spatial Query Module** (normalized exterior R-Tree + $O(1)$ interior `cell_id` lookups).
- Zero-CPU heightmap loading via `mmap` zero-copy `rkyv` buffers (`cell_cache.rkyv`).
- Dynamic sub-millisecond cell load/unload pipeline across interior/exterior boundaries with zero loading screens.

### 2.3 Vercidium-Style GPU Instanced Indirect Renderer

- Multi-draw indirect rendering pipeline (`DrawMeshInstancedIndirect`) via `wgpu`.
- Batching hundreds of thousands of static world instances (foliage, trees, rocks, architecture) into GPU buffers.
- Occlusion & frustum GPU culling compute shaders (HZB culling) to maintain 60+ FPS on integrated GPUs.

### 2.4 Terrain & Water Shader Pipeline

- Multi-layer PBR terrain shader (up to 6 splat texture layers per land cell).
- Dynamic water surface rendering with planar reflections and flow maps.
