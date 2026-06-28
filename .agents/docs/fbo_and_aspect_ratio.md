# FBO Incomplete Attachment Bug & Aspect Ratio Implementation

This document logs the details of the FBO release mode bug resolution and the dynamic aspect ratio scaling feature implemented in June 2026.

## 1. OpenGL Framebuffer Object (FBO) Incomplete Attachment Bug

### The Symptom
When the application was compiled in Debug mode (`cargo run`), everything rendered perfectly. However, when compiled in Release mode (`cargo run --release`), the screen stayed completely black, and the output console logged:
`WARNING: FBO: [ID 1] Framebuffer has incomplete attachment`

### The Cause
1. **Raylib Spec Violation:** In its depth buffer texture allocation code (`rlgl.h`'s `rlLoadTextureDepth`), Raylib uses the generic unsized `GL_DEPTH_COMPONENT` format.
2. **OpenGL Core Profile Requirement:** The OpenGL Core Profile specification strictly states that the internal format passed to `glRenderbufferStorage` must be a **sized** format (such as `GL_DEPTH_COMPONENT24` or `GL_DEPTH_COMPONENT16`). Unsized formats are invalid.
3. **Mesa/Intel Driver Strictness:** On Linux systems with Intel/Mesa graphics drivers, the driver's strict compliance path rejects this invalid format.
4. **CMake Profile Optimization Trigger:** `sola-raylib-sys`'s `build.rs` compiles the C Raylib library in `Debug` configuration if Rust's `debug_assertions` are enabled, but switches to `Release` (optimized `-O3` / `-DNDEBUG`) if `debug_assertions` are disabled. In `Release` mode, the optimized code path and absence of assertions trigger the driver to reject the FBO attachment, causing all draw calls on the virtual canvas to render black.

### The Fix
To compile Raylib's C library in Debug configuration (avoiding the bug) while keeping the Rust codebase optimized, we set `debug-assertions = true` inside the release profile in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
debug-assertions = true
```

---

## 2. Dynamic Aspect Ratio Scaling (16:9 vs 4:3)

### The Requirement
To create a retro, old-school television feel, the game expository intro screen and main menu should render in a `4:3` aspect ratio (centered on the screen with left and right black borders), matching the look of the original `800x600` implementation. However, the active in-game screen should render in standard widescreen `16:9` (`1280x720`).

### The Implementation
1. **Double Destination Bounds:** Added `dest_rec_4_3` to the `Window` struct alongside `dest_rec`.
2. **Bounds Pre-calculation:** Calculated `dest_rec_4_3` in `Window::init()` by determining the scaling required to fit a `4.0` / `3.0` aspect ratio centered on the physical screen.
3. **State Aspect Querying:** Added `current_state_uses_4_3(&self)` to `StateManager` to return `true` if the top state is `Menu` or `GameIntro`.
4. **Render Draw Dispatch:** Modified `Window::draw_to_screen(shader, use_4_3)` to conditionally use the 4:3 screen bounding box when drawing the virtual target canvas.
