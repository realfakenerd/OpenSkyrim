# OpenSkyrim Technology Stack

## 1. Core & Engine (Rust)
- **Engine:** [Bevy 0.18+](https://bevyengine.org/) - Utilizing the ECS (Entity Component System) for performance and modularity.
- **Physics:** [Rapier 3D](https://rapier.rs/) - High-performance physics engine for Rust.
- **Data Parsing:** [esplugin](https://crates.io/crates/esplugin) (or similar) - To handle original Bethesda .esm, .esp, and .esl files.
- **Asset Pipeline:** [nifly](https://crates.io/crates/nifly) - For parsing original NIF (Gamebryo) assets and converting them to glTF 2.0.
- **Version Management:** Bleeding Edge - Prioritizing the latest Bevy development features.

## 2. UI Layer (Tauri + Svelte 5)
- **Framework:** [Svelte 5](https://svelte.dev/) - Utilizing Runes ($state, $derived, $effect) for optimized reactivity.
- **App Shell:** [Tauri 2.0](https://tauri.app/) - Providing the bridge between Rust and the Svelte frontend.
- **Build Tool:** [Vite](https://vitejs.dev/) - For fast development and HMR.
- **Styling:** Modern CSS (Grid, Flexbox, Blur effects) to implement a skeuomorphic UI.

## 3. Scripting & Logic
- **Scripting Language:** LuaJIT via [mlua](https://github.com/mlua-rs/mlua).
- **Transpiler:** Custom Papyrus-to-Lua transpiler using [pest](https://pest.rs/) or [nom](https://github.com/rust-bakery/nom).
- **Async Runtime:** [Tokio](https://tokio.rs/) - For handling network I/O and async tasks within the Lua bridge.

## 4. Configuration & Serialization
- **System Configs:** **JSON** - Standardized format for engine configurations and Tauri bridge communication.
- **User Settings:** **TOML** - Human-readable format for user-facing options and mod configurations.
- **Binary Formats:** Original Skyrim formats (.esm, .esp, .nif) parsed directly into the engine's internal state.

## 5. Build & Infrastructure
- **Package Management:** Monorepo structure using `bun`.
- **Build System:** [Turbo Repo](https://turbo.build/) - For high-performance build caching and task orchestration across the monorepo.
- **CI/CD:** [GitHub Actions](https://github.com/features/actions) for automated testing, linting (Clippy), and builds.
- **Cross-Compilation:** [Docker](https://www.docker.com/) for reproducible environments, specifically for Android targets.

## 6. Project Structure
- `/src-tauri`: Core Backend (Rust, Bevy, Lua Bridge, ESM Parser).
- `/src`: Frontend UI (Svelte 5, HUD, Menus).
- `/docs`: Project Documentation (Astro with Starlight template).
- `/assets`: Converted assets (glTF) and audio files.
- `/scripts`: Transpiled Lua scripts.
