# AGENTS.md

## Build & Run

- `cargo check` / `cargo build` / `cargo run` — standard Rust toolchain, no extra steps
- No tests, lints, or formatting config exist yet
- Edition 2024 — requires a recent Rust toolchain (1.85+)

## Architecture

Single crate (not a workspace). Binary (`main.rs`) + library (`lib.rs`).

- **`ecs.rs`** — Custom ECS using SoA (struct-of-arrays) layout. Entities are `usize` indices into parallel `Vec`s on `World`. Optional components use `Vec<Option<T>>` sparse arrays.
- **`components.rs`** — Component structs (`MovementData`, `AnimationData`, `Mortality`, `WeaponSet`, `PhysicsData`) and stat constants.
- **`types.rs`** — `EntityType` enum (tags), `SpriteID` enum, `EntityID` wrapper.
- **`systems/`** — ECS systems: `physics`, `collision`, `behaviour` (with `gobbler` submodule), `input`. Most are stubs.
- **`prelude.rs`** — Re-exports all public modules and types.

## Conventions

- `main.rs` uses a fixed 1280×720 virtual resolution with render-texture scaling. Platform-specific fullscreen: macOS uses `toggle_fullscreen()`, other platforms set window to monitor dimensions.
- Dependencies: only `sola-raylib` (not the `raylib` crate — API differs).
- Entity classification: `is_enemy()` → Gobbler, EvilEye, Fireworm. `is_weapon()` → Bullet, Fireball, Bomb.
