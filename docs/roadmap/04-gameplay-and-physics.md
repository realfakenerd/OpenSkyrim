# Phase 4: Gameplay Mechanics, Physics & Save Persistence

> **Goal:** Implement core RPG game loops, character physics, AI behaviors, and sub-second save state persistence.

---

## 🎯 Key Deliverables & Specifications

### 4.1 Physics & Collision Integration (`Rapier 3D`)

- Convert Havok `.hkx` collision meshes into Rapier 3D colliders.
- Character Controller physics (walking, jumping, swimming, slope sliding, step climbing).

### 4.2 Skeletal Animation System

- Playback of glTF skinning clips; animation state blending tree (idle, walk, run, sprint, attack).
- Dynamic weapon attachment nodes (`WeaponRight`, `WeaponLeft`, `Shield`, `Quiver`).

### 4.3 Combat, Magic & Quest State Machine

- Melee hit registration, stamina/magicka resource management, spellcasting trajectories.
- Quest stage state machine synchronized with Luau events and SQLite database record updates.

### 4.4 Sub-Second Save / Quick-Load Architecture

- Luau runtime table state snapshots.
- Delta state tracking written to SQLite database (`skyrim_world.db`).
- Instant sub-second save creation and loading without thread corruption.
