# OpenSkyrim Launcher & Setup Workflow

This document details the user journey, automatic game directory detection, asset transformation pipeline trigger, and game launch process.

---

## 1. User Experience & Workflow Diagram

```
                 ┌───────────────────────────────┐
                 │    Gamer Runs OpenSkyrim      │
                 │          Launcher             │
                 └───────────────┬───────────────┘
                                 │
                                 ▼
                 ┌───────────────────────────────┐
                 │  Auto-Detect Skyrim Folder    │
                 │ (Steam, GOG, Custom Registry) │
                 └───────────────┬───────────────┘
                                 │
                    ┌────────────┴────────────┐
                    │ Found game directory?   │
                    └─────┬─────────────┬─────┘
                     YES  │             │  NO
                          │             ▼
                          │   ┌───────────────────┐
                          │   │ File Picker Dialog│
                          │   │ "Where is Skyrim?"│
                          │   └─────────┬─────────┘
                          │             │
                          └───────┬─────┘
                                  │
                                  ▼
                 ┌───────────────────────────────┐
                 │  Check if Assets Modernized?  │
                 └───────────────┬───────────────┘
                                 │
                    ┌────────────┴────────────┐
                    │ Transformed files exist?│
                    └─────┬─────────────┬─────┘
                     YES  │             │  NO
                          │             ▼
                          │   ┌───────────────────┐
                          │   │ Run Converter     │
                          │   │ Pipeline (.bsa,   │
                          │   │ .nif, .dds, .pex) │
                          │   └─────────┬─────────┘
                          │             │
                          └───────┬─────┘
                                  │
                                  ▼
                 ┌───────────────────────────────┐
                 │   Enable "PLAY" Button in UI  │
                 │  Launch OpenSkyrim Bevy Engine│
                 └───────────────────────────────┘
```

---

## 2. Step-by-Step Launcher Specifications

### Step 1: Automated Game Detection

When the launcher opens, it checks common installation paths on Windows:

- **GOG Registry / Install Paths:** `C:\GOG Games\The Elder Scrolls V Skyrim Special Edition`
- **Steam Common Paths:** `C:\Program Files (x86)\Steam\steamapps\common\Skyrim Special Edition`
- **Local Workspace Fallback:** `./skyrim_game/` or `./game_data/`

If non-existent or invalid, it opens a Native File Dialog:

> _"Skyrim game directory not automatically found. Please select your Skyrim Special Edition installation folder."_

---

### Step 2: Validation & Modernization Check

The launcher validates the selected folder for signature files:

- Required: `Data/Skyrim.esm`, `Data/Skyrim - Textures0.bsa`, `Data/Skyrim - Meshes0.bsa`

It checks if the target `modern_assets/` output directory already contains valid converted data:

- `modern_assets/skyrim_world.db` (SQLite database)
- `modern_assets/meshes/` (glTF 2.0 `.glb` models)
- `modern_assets/textures/` (KTX2 compressed textures)
- `modern_assets/scripts/` (Transpiled Luau scripts)

---

### Step 3: Transformation Progress Bar (Library Call)

The launcher imports `converter` directly as a Rust crate dependency. It invokes the conversion functions in a background Rust thread while updating the GUI progress bar:

```rust
// Inside launcher
use converter::{NifConverter, TextureConverter, EsmConverter};

pub fn run_conversion_job(game_dir: PathBuf, progress_tx: Sender<ProgressUpdate>) {
    std::thread::spawn(move || {
        // Stage 1: Convert Textures
        TextureConverter::convert_all(&game_dir, &progress_tx);
        // Stage 2: Convert 3D Meshes
        NifConverter::convert_all(&game_dir, &progress_tx);
        // Stage 3: Parse ESM to libSQL
        EsmConverter::convert_all(&game_dir, &progress_tx);
    });
}
```

---

### Step 4: Ready to Play & Spawning the Engine

When conversion finishes (or on subsequent launches):

1. The launcher saves the path configuration to `config.json`.
2. Clicking **"PLAY OPENSKYRIM"** spawns the game engine binary (`engine`):
   ```rust
   use std::process::Command;

   pub fn launch_game_engine() {
       Command::new("./engine")
           .arg("--config")
           .arg("config.json")
           .spawn()
           .expect("Failed to launch OpenSkyrim engine binary!");

       // Optionally close the launcher
       std::process::exit(0);
   }
   ```
