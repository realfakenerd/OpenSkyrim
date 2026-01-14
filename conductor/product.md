# OpenSkyrim (Base Engine)

## Initial Concept

**Project:** OpenSkyrim (Base Engine)

**Goal:** Create an engine capable of running original Skyrim data but with modern technologies.

**Architecture:**

1.  **Core & Engine (Rust + Bevy)**
    *   **Engine:** Bevy 0.18+. Use ECS pattern for all entity management.
    *   **Asset Pipeline:** Implement NIF (Gamebryo) to glTF 2.0 conversion (using `nifly` crate). Bevy loads processed glTF.
    *   **Data Parsing:** Use `esplugin` or similar to read .esm, .esp, .esl files. Extract Cell, Worldspace, and Reference to populate ECS.
    *   **Physics:** Integrate `Rapier` for 3D physics (replacing Havok).

2.  **UI Layer (Tauri + Svelte 5)**
    *   **Framework:** Svelte 5 (Runes: $state, $derived, $effect).
    *   **Bridge:** Tauri 2.0 for bidirectional communication between Rust (game state) and Svelte (HUD/Menus).
    *   **Features:** UI must support SVG, WebP, and modern CSS (Blur, Grid, Flexbox) for a Fluent Design (Windows 11) experience.

3.  **Scripting & Logic (Lua + Papyrus Transpiler)**
    *   **Language:** LuaJIT (via `mlua`).
    *   **Transpilation:** System to read .pex/.psc (Papyrus) and map calls to Lua functions exposed by Rust Core.
    *   **Extensibility:** Lua system accesses network APIs (HTTP) for real-time data mods or external device integration (e.g., Poco F5).

4.  **Suggested Folder Structure:**
    *   `/src-tauri`: Backend Rust (Core, Bevy, Lua Bridge, ESM Parser).
    *   `/src`: Frontend Svelte 5 (HUD, Inventory, Menus).
    *   `/assets`: Converted glTF files and sounds.
    *   `/scripts`: Lua scripts generated from Papyrus.

## Core Vision & Goals
- **Modern Performance & Stability:** Prioritizing a crash-resistant engine that achieves high framerate stability and low latency input.
- **Advanced Modding & Extensibility:** Empowering modders with LuaJIT scripting and real-time external data integration (HTTP).
- **Cross-Platform Accessibility:** Native support for Windows, Linux, and Android.
- **Seamless Exploration:** Architected for efficient data streaming to enable seamless cell transitions, aiming to minimize or eliminate traditional loading screens.

## Target Audience
- **Hardcore Modders:** Providing a modern engine to push the limits of Skyrim's original data.
- **Stability-Seeking Players:** Delivering a performance-optimized and stable way to experience the classic game.
- **RPG Developers:** Offering a modern, ECS-based foundation for new RPG projects.

## Minimum Viable Product (MVP)
The initial milestone focuses on achieving a playable baseline:
1.  **Data Parsing:** Successful loading of master files (.esm) to populate the Bevy ECS.
2.  **Asset Pipeline:** Rendering static world scenes from converted glTF assets.
3.  **Basic Gameplay:** A functional character controller with 3D physics (collision) moving within a loaded game cell.

## Input & Interaction
- **Primary Control:** Keyboard and Mouse for Desktop platforms.
- **Gamepad Support:** Unified gamepad integration for both Desktop and Android.
- **Future Roadmap:** Implementation of touch-friendly UI overlays for mobile platforms.