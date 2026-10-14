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
- **`camera.rs`** — 2D viewport camera (`GameCamera`) managing virtual coordinates (320x180), panning, and screen projection.
- **`systems/`** — ECS systems: `physics`, `collision`, `behaviour` (with `gobbler` submodule), `input`, `hook`.
- **`prelude.rs`** — Re-exports all public modules and types.

## Conventions

- `main.rs` uses a fixed 1280×720 virtual resolution with render-texture scaling. Platform-specific fullscreen: macOS uses `toggle_fullscreen()`, other platforms set window to monitor dimensions.
- Dependencies: only `sola-raylib` (not the `raylib` crate — API differs).
- Entity classification: `is_enemy()` → Gobbler, EvilEye, Fireworm. `is_weapon()` → Bullet, Fireball, Bomb.

## Release Build & OpenGL FBO Workaround

- In `Cargo.toml`, we use `[profile.release.build-override]` with `opt-level = 3` and `debug-assertions = false` to ensure raylib is built in optimized Release mode for 60 FPS performance.
- On Linux Intel/Mesa drivers, raylib's default `LoadRenderTexture` encounters two driver/compiler issues under release optimizations:
  1. Raylib's `rlLoadTexture(NULL, ...)` is miscompiled under GCC `-O3` to call `glCompressedTexImage2D` with format `GL_RGBA8`, which Mesa rejects with `GL_INVALID_ENUM`, leaving the color texture unallocated.
  2. `rlLoadTextureDepth` passes unsized `GL_DEPTH_COMPONENT` to `glRenderbufferStorage`, failing depth buffer allocation.
- This causes `glCheckFramebufferStatus` to return `GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT`, rendering a black screen except for procedural shaders.
- **Resolution**: In `src/window.rs`, `Window::init` checks if `target.id` has an incomplete FBO (`!rlFramebufferComplete(target.id)`). If so, it cleanly detaches/unloads the unneeded depth renderbuffer (Rustahn is a 2D game) and re-allocates a valid color texture with initialized pixel memory via `rlLoadTexture`. This achieves full release-mode performance (60 FPS) without any black-screen issues on Linux/Mesa or Windows.

## Rendering and Aspect Ratio Conventions

- The virtual game loop targets a fixed resolution of 1280x720 (16:9).
- Inside `StateManager` and `Window`, we support dynamic aspect ratio scaling:
  - **Menu and GameIntro screens** are drawn in a centered `4:3` bounding box (leaving left and right black borders) to simulate a retro CRT television feel.
  - **Game, Pause, and Options screens** are scaled to the full `16:9` resolution.
  - This aspect ratio selection is automatically queried from `StateManager::current_state_uses_4_3` and passed to `Window::draw_to_screen`.

## Audio Management & Lifecycles

- **Audio Device Context**: Owned by `main()` in `main.rs` as a `RaylibAudio` instance, outliving the application.
- **Audio Resources**: `music_tracks: HashMap<MusicID, Music<'aud>>` and `sound_effects: HashMap<SoundID, Sound<'aud>>` are owned by `Application<'aud>`.
- **`AudioManager` Controller**: A lightweight view-struct `AudioManager<'a, 'aud>` constructed on the fly each frame in `Application::update()` and passed to screen updates (`audio: &mut AudioManager<'_, '_>`). This avoids self-referential compiler constraints entirely.
- **Drop Ordering Safety**: In `Application<'aud>`, the resource maps (`music_tracks`, `sound_effects`) must be declared *before* the `window` field. This ensures that all audio assets are dropped and unloaded (calling `UnloadMusicStream` and `UnloadSound` in C) *before* the window's Raylib context is dropped, preventing memory access violations/segfaults on shutdown.
