# Specification: Setup Project Scaffolding

## 1. Overview
This track focuses on initializing the foundational structure of the OpenSkyrim project. We will set up a monorepo using `pnpm` and `Turbo Repo` to manage the distinct parts of the application: the Rust/Bevy backend, the Svelte 5 frontend (via Tauri), and the Astro/Starlight documentation.

## 2. Goals
- Initialize a git repository (already done, but verified).
- Configure a `pnpm` monorepo with `Turbo Repo`.
- specific workspaces:
    - `src-tauri`: The Rust backend with Tauri and Bevy.
    - `src`: The Svelte 5 frontend.
    - `docs`: The Astro/Starlight documentation site.
- Ensure all parts can be built and run via Turbo Repo commands.

## 3. Technical Implementation
### 3.1 Monorepo Structure
- Root `package.json` with `pnpm` workspace configuration.
- `turbo.json` configuring the build pipeline (build, test, dev).

### 3.2 Backend (Rust + Tauri)
- Initialize a new Tauri app.
- Add `bevy` (0.18+ or latest) as a dependency in `src-tauri/Cargo.toml`.
- Configure `tauri.conf.json` to point to the frontend build.

### 3.3 Frontend (Svelte 5)
- Initialize a Svelte 5 project (using Vite) in the `src` folder (or moving the Tauri-created one).
- Configure Vite to work with Tauri.

### 3.4 Development Environment
- **Dockerfile:** A custom image containing Rust, Node.js, pnpm, and all system libraries required by Tauri (GTK, WebKit2Gtk) and Bevy (ALSA, X11/Wayland libs).
- **.devcontainer/devcontainer.json:** Configuration for VS Code and Zed to automatically build and enter the container, with pre-installed extensions for Rust, Svelte, and Lua.

### 3.4 Documentation
- Initialize a new Astro project with the Starlight template in the `docs` folder.

## 4. Verification
- Running `pnpm dev` at the root should start the Tauri app (Rust + Svelte) and potentially the docs server (or `pnpm docs` for that).
- `pnpm build` should successfully build all packages.
