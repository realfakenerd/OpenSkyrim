# Plan: Setup Project Scaffolding

## Phase 1: Monorepo & Turbo Repo Initialization [checkpoint: c575feb]

- [x] Task: Initialize Bun Workspace (5207e9e)
    - [x] Sub-task: Create root `package.json` with `workspaces` configuration.
    - [x] Sub-task: Initialize `bun.lockb` via `bun install`.
    - [x] Sub-task: Install `turbo` as a dev dependency.

- [x] Task: Configure Turbo Repo (56056af)
    - [x] Sub-task: Create `turbo.json` with basic pipelines for `build`, `dev`, and `test`.

- [x] Task: Conductor - User Manual Verification 'Monorepo & Turbo Repo Initialization' (a6c0a1c)

## Phase 2: Backend & Frontend Scaffolding (Tauri + Svelte 5)

- [x] Task: Scaffold Tauri App with Svelte 5
    - [x] Sub-task: Use `pnpm create tauri-app` to scaffold the structure.
    - [x] Sub-task: Select Svelte (TypeScript) as the frontend framework.
    - [x] Sub-task: Rearrange folders to match project structure (`src-tauri`, `src`).

- [x] Task: Configure Tauri & Vite
    - [x] Sub-task: Ensure `tauri.conf.json` points to the correct frontend build output.
    - [x] Sub-task: Update `vite.config.ts` for Tauri compatibility.

- [~] Task: Add Bevy Dependency
    - [ ] Sub-task: Add `bevy` to `src-tauri/Cargo.toml`.
    - [ ] Sub-task: Create a basic Bevy "Hello World" system in `src-tauri/src/main.rs` to verify it compiles.

- [ ] Task: Conductor - User Manual Verification 'Backend & Frontend Scaffolding' (Protocol in workflow.md)

## Phase 3: Documentation Scaffolding (Astro + Starlight)

- [x] Task: Initialize Astro Starlight
    - [x] Sub-task: Run `pnpm create astro@latest --template starlight` in the `docs` directory.
    - [x] Sub-task: Ensure it is added to the pnpm workspace.

- [ ] Task: Integrate Docs into Turbo
    - [ ] Sub-task: Update `turbo.json` to include the docs build/dev tasks.

- [ ] Task: Conductor - User Manual Verification 'Documentation Scaffolding' (Protocol in workflow.md)

## Phase 4: Development Environment (Docker & Dev Container)

- [x] Task: Create Dockerfile (634ddd9)
    - [x] Sub-task: Set up base image with Rust and Node.js.
    - [x] Sub-task: Install system dependencies for Tauri and Bevy.

- [x] Task: Configure devcontainer.json (6883a6b)
    - [x] Sub-task: Create `.devcontainer/devcontainer.json` for editor integration.
    - [x] Sub-task: Map volumes and configure workspace permissions.

- [~] Task: Conductor - User Manual Verification 'Development Environment' (Protocol in workflow.md)
