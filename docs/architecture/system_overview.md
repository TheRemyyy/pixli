# System architecture

This document describes the main loops and data flow in Pixli.

## Application loop

The engine uses **winit** for the event loop and window. When the app is **resumed** (e.g. first frame or after suspend):

1. **Window** — Created via `event_loop.create_window(...)`. On failure, an `Error::Window` is stored and the loop exits; `App::run()` then returns `Err`.
2. **wgpu** — Instance, surface (from window), adapter request, device + queue. Any failure is mapped to `Error` (NoAdapter, DeviceRequest, Surface), stored, and the loop exits.
3. **Surface config** — Format (srgb preferred), size, vsync/immediate, alpha mode.
4. **Renderer init** — `renderer.init(device, queue, format, width, height)` creates pipelines, textures, and buffers.
5. **Startup systems** — Run once; typically spawn entities and upload meshes.
6. **Mesh upload** — All entities with `Mesh` are uploaded to the GPU (mesh cache).

Each frame:

1. **Time** — Delta time and frame count updated.
2. **Physics** — `Physics::update(world, delta)` (velocity integration, collision detection, resolution).
3. **Systems** — User `GameState` systems run (input, gameplay, camera sync).
4. **Render** — `renderer.render(world, &swapchain_view)` (see below).
5. **Present** — Swapchain present; on `SurfaceError`, handle Lost/Outdated (reconfigure), Timeout (skip frame), OutOfMemory (exit).
6. **Input** — State cleared for next frame.

## Render pipeline

Rendering is split into helper passes; `render()` only orchestrates and submits.

1. **View/proj and light** — Camera matrices, directional light direction/color/intensity.
2. **Unlit batching** — `build_unlit_batches()`: query unlit entities, sort by mesh, fill instance scratch; upload to GPU. Optional upload of batch offsets.
3. **Depth pre-pass (SSAO)** — When SSAO enabled: write 1x depth to a texture (unlit + lit); used later for SSAO sample.
4. **Shadow pass** — Render lit geometry from light view into shadow map; or clear if shadows disabled.
5. **Main pass** — Render to scene texture (with MSAA resolve if needed): sky, unlit batches, lit entities (uniforms, shadow bind group).
6. **Bloom** — Extract bright, blur H, blur V (or clear bloom texture when disabled).
7. **SSAO** — Compute AO from depth, blur; or clear to white when disabled.
8. **Post pass** — Composite scene + bloom + SSAO to swapchain (tone mapping).

All passes use the same `CommandEncoder`; a single `queue.submit(encoder.finish())` is used at the end.

## ECS

- **World** — Stores entities and component storages (per-type). Entities are indices; components are stored in contiguous arrays keyed by entity.
- **Spawn** — Returns an `EntityBuilder`; add components with `.with(...)`, then `.build()` to get the entity.
- **Query** — `world.query::<(&Transform, &Mesh)>()` returns an iterator over entities that have all listed components.
- **Get** — `world.get::<T>(entity)` and `world.get_mut::<T>(entity)` for single-component access.

## Error handling

- **App::run()** — Returns `Result<(), Error>`. Event loop creation, window creation, surface, adapter, and device request failures are converted to `Error` and returned instead of panicking.
- **Renderer** — When device/queue or required resources are missing, `render()` returns early (no draw). Pipeline recreation on MSAA change uses `if let Some(device)` so missing device does not panic.
- **Physics** — No unwraps; missing `RigidBody` or `Transform` is skipped (e.g. `if let Some(rb) = world.get::<RigidBody>(entity)`).
