# DDS to KTX2 / Basis Universal Texture Transformation Specification

This document details the technical specification for converting Skyrim DirectDraw Surface (`.dds`) textures into supercompressed, GPU-ready **KTX2 (Basis Universal)** textures for OpenSkyrim.

---

## 1. Overview & Objectives

- **Input:** Skyrim `.dds` files extracted from BSA archives or loose mod files (encoded in BC1/DXT1, BC3/DXT5, or BC7 compression formats).
- **Output:** Supercompressed **KTX2** binary files (`.ktx2`) using Basis Universal compression.
- **Goal:**
  1. Reduce texture VRAM memory footprint by up to **75%**.
  2. Provide zero-CPU transcoding at runtime directly into GPU-supported compressed formats (BC1/BC7 for Desktop, ASTC/ETC2 for Mobile/WebGPU).

---

## 2. Format & Compression Mapping

| Skyrim DDS Format     | Texture Type                             | KTX2 Target Encoding   | Basis Universal Mode                                |
| :-------------------- | :--------------------------------------- | :--------------------- | :-------------------------------------------------- |
| **BC1 / DXT1 (RGB)**  | Diffuse / Albedo Map (no alpha)          | `KTX2 (ETC1S / UASTC)` | `BasisTextureType::Texture2D`                       |
| **BC3 / DXT5 (RGBA)** | Diffuse with Alpha Mask / Transparency   | `KTX2 (ETC1S / UASTC)` | `BasisTextureType::Texture2D` (With Alpha)          |
| **BC5 / ATI2 (RG)**   | Normal Maps                              | `KTX2 (UASTC)`         | High-quality UASTC (Preserves Normal map precision) |
| **BC7 (BPTC)**        | High-quality PBR Textures / SE Overhauls | `KTX2 (UASTC)`         | `BasisTextureType::Texture2D`                       |

---

## 3. Transformation Pipeline Diagram

```
┌──────────────────────┐
│  Skyrim DDS File     │
│  (BC1 / BC3 / BC7)   │
└──────────┬───────────┘
           │
           ▼ (Rust `image` / `ddsfile` crate)
┌─────────────────────────────────────────────────────────────────────────────┐
│ 1. Decode & Parse DDS Buffer                                                │
│    - Read Mipmaps, Header Flags, Width, Height, and Pixel Formats           │
│    - Decompress / Re-encode pixel buffers into RGBA8 raw image slices       │
└──────────┬──────────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 2. Compress via Basis Universal (`basis-universal` crate)                   │
│    - Normal Maps   > Transcode to UASTC mode (high precision vectors)       │
│    - Color Textures> Transcode to ETC1S mode (supercompressed payload)      │
│    - Generate KTX2 Mipmap Pyramid Chain                                     │
└──────────┬──────────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 3. Output KTX2 Texture (.ktx2)                                              │
│    - Write KTX2 Container Header & Supercompressed Bitstream                │
│    - Store in `transformed_data/textures/` for instant Bevy WebGPU loading  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Runtime Transcoding in Bevy Engine

When OpenSkyrim loads a `.ktx2` texture into Bevy's AssetServer:

```
                      ┌──────────────────────┐
                      │    .ktx2 File        │
                      │  (Supercompressed)   │
                      └──────────┬───────────┘
                                 │
                   ┌─────────────┴─────────────┐
                   │    Detect Target GPU      │
                   └──────┬─────────────┬──────┘
                          │             │
        ┌─────────────────┘             └─────────────────┐
        ▼ (Desktop Vulkan / DirectX12)                    ▼ (Mobile / WebGPU)
┌───────────────────────────────┐               ┌───────────────────────────────┐
│ Transcode to BC1 / BC7 (VRAM) │               │ Transcode to ASTC / ETC2      │
└───────────────────────────────┘               └───────────────────────────────┘
```

---

## 5. Precise Rust Implementation (`ddsfile` + `basis-universal` Crates)

```rust
use ddsfile::{Dds, DxgiFormat, D3dfmt};
use basis_universal::{CompressorParams, Compressor, BasisFormat, ColorSpace};

pub struct DdsToKtx2Converter;

impl DdsToKtx2Converter {
    /// Converts a raw Skyrim DDS byte buffer into a supercompressed KTX2 byte buffer
    pub fn convert(dds_bytes: &[u8], is_normal_map: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 1. Read DDS Header using ddsfile
        let dds = Dds::read(dds_bytes)?;
        let width = dds.get_width();
        let height = dds.get_height();

        // Extract raw pixel data for array layer 0
        let dds_data = dds.get_data(0)?;

        // 2. Setup Basis Universal Compressor Parameters
        let mut params = CompressorParams::new();
        params.set_uastc(is_normal_map); // Use UASTC for normal maps, ETC1S for diffuse
        params.set_generate_mipmaps(true);
        params.set_color_space(if is_normal_map { ColorSpace::Linear } else { ColorSpace::Srgb });

        // 3. Pass Raw RGBA Image Slices to Basis Compressor
        let source_image = params.source_image_mut(0);
        source_image.init(dds_data, width, height, 4); // 4 channels (RGBA)

        // 4. Run Compressor & Export KTX2
        let mut compressor = Compressor::new(params);
        compressor.process()?;

        // Retrieve KTX2 byte buffer
        let ktx2_bytes = compressor.output_ktx2();
        Ok(ktx2_bytes.to_vec())
    }
}
```
