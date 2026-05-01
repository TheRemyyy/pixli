# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Error handling** — `pixli::Error` and `pixli::Result`; `App::run()` returns `Result<(), Error>` instead of panicking.
- **Window/GPU init** — Window creation, surface creation, adapter request, and device request failures are reported as `Error` and exit the event loop; `run()` then returns `Err`.
- **Surface error handling** — `SurfaceError::Lost` and `Outdated` trigger surface reconfigure; `Timeout` skips frame with trace log; `OutOfMemory` logs and exits.
- **Renderer** — No unwraps in resize or pipeline recreation; optional SSAO/blur resources use `if let Some(...)`.
- **Physics** — No unwraps; missing `RigidBody` or invalid raycast state handled with `Option`/early continue.
- **Documentation** — README (overview, requirements, installation, usage, structure, docs links), `docs/overview.md`, `docs/architecture/system_overview.md`, CONTRIBUTING.md, CHANGELOG.md.
- **CI** — GitHub Actions workflow for `cargo test` and `cargo build --release` (Windows, Linux, macOS).
- **Cargo** — Edition set to 2021; `thiserror` dependency for `Error` derive.
- **Linux desktop support** — Explicit X11/Wayland winit features, Linux app id / WM_CLASS metadata, Vulkan-only backend selection, and surface-capability based present mode fallback for GNOME, KDE Plasma, wlroots, X11, and XWayland sessions.
- **Release game binary** — `cargo build --release` now builds `pixli-shooter`, and `cargo run --release` launches it by default.
- **Profiler** — Built-in CPU frame profiler controlled by `PIXLI_PROFILE=1` with timings for physics, systems, surface acquire, render, present, and total frame time.
- **GPU profiling** — Vulkan timestamp queries report GPU frame time when the adapter supports timestamp queries.
- **Stress scene** — `pixli-stress` binary renders a dense lighting/post-processing scene for repeatable performance checks.

### Changed

- **Cargo.toml** — `edition` from 2024 to 2021 for stable toolchain compatibility.
- **Examples** — Shooter example `main` returns `pixli::Result<()>` and uses `run()` without unwrap.
- **Audio** — Rodio output is behind the `audio` feature so Linux library and release-game builds do not require ALSA development headers unless sound is enabled.
- **Graphics** — Non-Vulkan wgpu backends are rejected at startup; Pixli now requires a Vulkan adapter.
- **Quality presets** — Shadows, SSAO, and bloom now use explicit quality settings; shooter enables SSAO and bloom by default.
- **Renderer CPU overhead** — Bloom render target views and lit entity collection now reuse cached allocations instead of recreating per frame.

## [0.1.0] — Initial release

- ECS (World, Entity, Query, components).
- Renderer (wgpu): lit/unlit pipelines, shadows, SSAO, bloom, MSAA, sky, fog.
- Physics: Collider (box, sphere), RigidBody, collision events, raycast.
- Audio (rodio), Input, Time, App loop (winit).
- Shooter example (FPS with config and meshes modules).
