# OpenSkyrim Modding Architecture & Compatibility Strategy

Skyrim has the largest modding ecosystem in PC gaming. This document outlines how OpenSkyrim handles legacy Skyrim mods (`.esp`, `.esm`, `.esl`, `.bsa`, mesh/texture overrides, Papyrus scripts) while introducing modern, native Lua modding.

---

## 1. Dual-Tier Modding Architecture

```
                               ┌───────────────────────────────┐
                               │     Modder / User Setup       │
                               │  (VORTEX / MO2 / Data Dir)    │
                               └───────────────┬───────────────┘
                                               │
                                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    OpenSkyrim Mod Ingestion & Sandbox                       │
│                                                                             │
│   ┌─────────────────────────────────┐   ┌───────────────────────────────┐   │
│   │  Tier 1: Legacy Compatibility   │   │  Tier 2: Native Modern Mods   │   │
│   │  - Unpack `.esp`/`.esl` to DB   │   │  - Direct glTF / KTX2 Assets  │   │
│   │  - On-the-fly .nif/.dds convert │   │  - Native Lua 5.4 Scripts     │   │
│   │  - Papyrus ➔ Lua Transpilation  │   │  - Hot-reloading & Sandboxing │   │
│   └─────────────────────────────────┘   └───────────────────────────────┘   │
└──────────────────────────────────────────────┬──────────────────────────────┘
                                               │
                                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Unified OpenSkyrim Engine Runtime                        │
│                 (Virtual Filesystem + SQLite Mod Layering)                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Handling Legacy Skyrim Mods (`.esp`, `.esl`, `.bsa`, `.nif`, `.dds`)

### A. Mod Plugin Load Order & Database Layering (`.esp` / `.esl`)
In original Skyrim, `.esp` files override records from `Skyrim.esm` based on load order (e.g. `plugins.txt`).
* **SQLite Mod Layering:**
  * OpenSkyrim stores base game data in `skyrim_world.db`.
  * When a user adds an `.esp` mod, the converter imports the plugin's records into a mod table with a higher **load order priority weight**.
  * Database query:
    ```sql
    SELECT * FROM refr_objects WHERE form_id = ? ORDER BY load_order DESC LIMIT 1;
    ```
  * **Benefit:** Instant record resolution with zero engine hacking required!

### B. Dynamic On-the-Fly Asset Conversion (Loose Files & BSA Mods)
* Mods containing loose `.nif` or `.dds` files are detected by the Virtual Filesystem (VFS).
* The converter transpiles mod `.nif` files to `.glb` and `.dds` textures to `.ktx2` in a local cache directory (`mod_cache/`).

### C. Legacy Papyrus Scripts (`.pex` Mods)
* `.pex` bytecode contained within mod `.bsa` archives or `Scripts/` folders is run through the **Papyrus-to-Lua AST Transpiler**.
* Mod functions call the unified OpenSkyrim Lua API bindings seamlessly.

## 3. Integrated Launcher Mod Manager & Storage Structure

Instead of relying solely on external tools, OpenSkyrim includes a **Built-in Mod Manager** inside the Launcher.

### Storage Hierarchy (`OpenSkyrim/`)
```
OpenSkyrim/
├── game_data/               # Unpacked base game & DLC assets
├── transformed_data/        # Transformed base game (glTF, KTX2, SQLite)
├── mods/                    # User installed raw mod folders (installed via Launcher)
│   ├── ModA_HighResArmor/
│   └── ModB_CustomQuest/
├── transformed_mods/        # Transformed mod cache (glTF, KTX2, Lua)
└── openskyrim_config.json   # Mod load order & active plugin list
```

### Built-in Mod Manager Features:
* **Drag-and-Drop Installation:** Drop `.zip` / `.7z` / `.rar` mod archives directly into the Launcher.
* **Visual Load Order Manager:** Drag and drop `.esp` / `.esl` load order priorities with conflict detection.
* **One-Click Transformation:** Clicking "Enable Mod" automatically runs the asset converter for that mod into `transformed_mods/`.

---

## 4. UI Modernization (Flash / Scaleform `.gfx` ➔ Web & Bevy UI)

### The Skyrim UI Problem:
Original Skyrim uses **Autodesk Scaleform**, which executes Adobe Flash (`.swf` / `.gfx` files with ActionScript 2.0). Flash is obsolete, insecure, and heavily limits UI customization.

### The OpenSkyrim Flash-to-HTML UI Integration with Bevy:

```
┌───────────────────────────┐
│ Original Skyrim Flash UI  │
│  (.swf / .gfx Scaleform)  │
└─────────────┬─────────────┘
              │
              ▼ (Offline Transpiler)
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Converted HTML5 / CSS3 / JS                           │
│  ┌──────────────────────────┬──────────────────────┬─────────────────────┐  │
│  │   HTML Templates (.html) │   CSS Stylesheets    │   JS Event Scripts  │  │
│  │   (DOM Structure)        │   (Themes & Fonts)   │   (UI Logic)        │  │
│  └──────────────────────────┴──────────────────────┴─────────────────────┘  │
└─────────────────────────────┬───────────────────────────────────────────────┘
                              │
                              ▼ (Render Integration)
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Bevy Engine Render Pipeline                         │
│                                                                             │
│   Option A: Ultralight / Blitz (Embedded Lightweight HTML Renderer)          │
│   Option B: bevy_hui / Bevy Native UI Node Tree (Pure Native GPU Execution)│
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │   Bevy ECS Bridge: UI Events ◄─── Bi-Directional IPC ───► Lua / Engine │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### How it works step-by-step:

1. **Flash Extraction & AST Transpilation (Offline Phase):**
   * **Shapes & Vector Art:** Vector graphics inside `.swf`/`.gfx` are exported to SVG / CSS vector shapes.
   * **ActionScript 2.0 ➔ JavaScript / Lua:** UI scripts (like opening inventory, equipping items, displaying health bars) are translated into modern JS or Lua UI functions.

2. **Rendering inside Bevy (Runtime Phase):**
   * **Approach A: Bevy 0.19 `bsn!` (Bevy Scene Notation) Native Nodes — *Recommended Native Path*:**
     * Transpile Flash layout structures directly into **Bevy 0.19 `bsn!` declarative macros**.
     * `bsn!` provides clean, patchable UI widget trees with automatic asset dependency management directly in Rust/Bevy without needing any browser runtimes or webview overhead!
   * **Approach B: Lightweight Embedded HTML Engine (`Ultralight` / `Blitz`):**
     * Ultralight renders HTML/CSS into a GPU texture buffer, which Bevy draws as an overlay quad directly on top of the 3D scene at 60+ FPS.

3. **Two-Way Event Bridge (UI ↔ Bevy ECS ↔ Lua):**
   * Clicking a menu button sends a lightweight event across the Rust bridge to Bevy systems.
   * Player health or inventory updates in Rust instantly trigger DOM / UI updates in real time.
