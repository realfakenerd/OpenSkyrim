# OpenSkyrim Estimated System Requirements

Comparing official **Skyrim Special Edition (2016)** baseline requirements against **OpenSkyrim's Rust/Bevy modernized runtime**.

---

## 1. Requirement Comparison Table

| Hardware Component      | Skyrim SE Official Minimum                         | **OpenSkyrim Target Minimum**                                              | Why It Goes Down / Changes                                                                                                 |
| :---------------------- | :------------------------------------------------- | :------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------- |
| **CPU (Processor)**     | Intel Core i5-750 / AMD Phenom II X4-945 (4 Cores) | **Dual-Core 64-bit CPU** (Intel i3 / Celeron / ARM64)                      | **Rust + Bevy ECS** provides true parallel multi-core scheduling. Lua 5.4 scripting consumes < 5% CPU compared to Papyrus. |
| **RAM (System Memory)** | 8 GB RAM                                           | **4 GB RAM**                                                               | **Zero-Copy `rkyv` / `mmap`** stream data straight from disk on demand, eliminating double-buffering & memory spikes.      |
| **GPU (Graphics)**      | NVIDIA GTX 470 (1GB) / AMD HD 7870 (2GB)           | **Vulkan / WebGPU Capable iGPU** (Intel HD 5000+ / Vega 3 / Apple Silicon) | **KTX2 Basis Textures** reduce VRAM footprint by 75%. **glTF 2.0** uses GPU instancing for static world meshes.            |
| **Storage Space**       | 12 GB                                              | **~6 - 8 GB** (Converted Assets)                                           | KTX2 supercompression and deduplicated glTF models significantly compress game installation footprint.                     |
| **Operating System**    | Windows 7/8.1/10 (64-bit)                          | **Windows, Linux, macOS, Android & WebAssembly (Browser)**                 | Bevy & WebGPU allow OpenSkyrim to run natively cross-platform!                                                             |

---

## 2. Key Performance Boosters in OpenSkyrim

1. **Lower CPU Bottlenecks:**
   - Creation Engine's original main thread handled rendering setup, Papyrus scripts, and cell streaming sequentially on 1-2 cores.
   - Bevy's ECS automatically splits rendering, audio, AI, and cell queries across all available CPU threads seamlessly.

2. **Lower RAM & VRAM Footprint:**
   - KTX2 textures allow low-spec GPUs (like integrated Intel UHD graphics or mobile chips) to load full high-res textures without running out of VRAM.

3. **Cross-Platform & Mobile Readiness:**
   - Because of WebGPU and Rust compilation, OpenSkyrim could theoretically run smoothly on low-power devices like a Raspberry Pi 5, Android tablets, or inside web browsers!
