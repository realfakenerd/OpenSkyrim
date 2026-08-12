# Data Formats & Specs Reference

This document outlines the binary structures of Bethesda Creation Engine files used by OpenSkyrim.

---

## 1. `.bsa` / `.ba2` Archive Format

Skyrim Special Edition uses two BSA archive formats:

1. **Classic Skyrim BSA (Header Magic: `BSA\0` / Version `105`)**
2. **Special Edition BA2 (Header Magic: `BTDX` / Version `1`)**

### Classic BSA Binary Header (36 bytes)

| Offset | Type      | Field Name                 | Description                                                                          |
| :----- | :-------- | :------------------------- | :----------------------------------------------------------------------------------- |
| `0x00` | `[u8; 4]` | `file_id`                  | Magic identifier: `"BSA\0"` (0x00415342)                                             |
| `0x04` | `u32`     | `version`                  | Format version (`104` for Oblivion, `105` for Skyrim)                                |
| `0x08` | `u32`     | `offset`                   | Offset to first file record header (usually 36)                                      |
| `0x0C` | `u32`     | `archive_flags`            | Bitflags (0x1 = Include Directory Names, 0x2 = Include File Names, 0x4 = Compressed) |
| `0x10` | `u32`     | `folder_count`             | Total number of folder record blocks                                                 |
| `0x14` | `u32`     | `file_count`               | Total number of files across all folders                                             |
| `0x18` | `u32`     | `total_folder_name_length` | Total length of all folder string paths                                              |
| `0x1C` | `u32`     | `total_file_name_length`   | Total length of all file string names                                                |
| `0x20` | `u32`     | `file_flags`               | Flags specifying contained file types (Meshes, Textures, Sound, Scripts, etc.)       |

---

## 2. `.esm` Record Format

Skyrim master files (`Skyrim.esm`, `Dawnguard.esm`) consist of a series of **Records** and **Groups**.

### Record Header Structure (24 bytes)

| Field       | Size          | Description                                                          |
| :---------- | :------------ | :------------------------------------------------------------------- |
| `type`      | 4 bytes ASCII | Record Type Identifier (e.g. `TES4`, `CELL`, `LAND`, `NPC_`, `REFR`) |
| `data_size` | `u32`         | Size of payload data following this header                           |
| `flags`     | `u32`         | Record flags (e.g. Compressed, Deleted, Ignored)                     |
| `form_id`   | `u32`         | Unique 32-bit ID for this game record                                |
| `revision`  | `u32`         | Revision count                                                       |
| `version`   | `u16`         | Record version                                                       |
| `unknown`   | `u16`         | Internal padding/flags                                               |

### Key Record Types for OpenSkyrim

- **`TES4`**: Master file header (contains author, description, master dependencies).
- **`CELL`**: Interior or Exterior cell definition (world coordinates, lighting parameters).
- **`LAND`**: Heightmap, vertex normal, vertex color, and texture layer data for terrain.
- **`REFR`**: World placement reference (links a FormID mesh/object to 3D coordinates `X, Y, Z, RotX, RotY, RotZ`).
- **`NPC_`**: Actor definition (stats, mesh paths, attributes).

---

## 3. `.nif` Model Format

NIF (NetImmerse Format) files store 3D geometry, skeletons, animation channels, and material properties.

### Primary NIF Blocks in Skyrim:

1. **`NiHeader`**: File signature (`NetImmerse File Format Header`), version string, block count.
2. **`BSFadeNode` / `NiNode`**: Scene graph nodes containing transform matrices (`translation`, `rotation`, `scale`).
3. **`BSTriShape`**: Compact geometry buffer introduced in Skyrim SE containing vertex positions (`Vec3`), UV coordinates (`Vec2`), normals (`Vec3`), and index buffers.
4. **`BSLightingShaderProperty`**: Materials containing texture paths (`base_color`, `normal_map`, `specular`).
