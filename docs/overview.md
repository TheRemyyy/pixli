# Introduction to Pixli

**Pixli** is a 3D game engine written in **Rust**, built for clarity and production use. It uses **wgpu** for portable graphics (Vulkan, DirectX 12, Metal) and provides an ECS, physics, and audio in a single crate.

**→ New to Pixli?** See [**How to use / API guide**](how_to_use.md) to learn how to create a game step by step and what you can use (App, GameState, ECS, physics, renderer, input, time, audio).

## Core philosophy

1. **No panics on user paths** — Window and GPU initialization return `Result`; you handle errors in `main`.
2. **Recovery from GPU/surface loss** — Lost/Outdated/Timeout surface errors are handled (reconfigure or skip frame); only OutOfMemory exits.
3. **Small, focused API** — App builder, startup and per-frame systems, and a clear prelude.

## Key features

### Rendering (wgpu)

- **Lit pipeline** — PBR-style (metallic/roughness), one directional light, shadows, fog, optional normal mapping.
- **Unlit pipeline** — Vertex-colored meshes, batching, fog.
- **Post-processing** — SSAO, bloom (extract + blur), tone-mapping composite to swapchain.
- **MSAA** — Configurable 1/2/4/8x; pipelines recreated on change.
- **Sky** — Fullscreen gradient; clear color and fog configurable.

### ECS

- **World** — Spawn/despawn entities, attach components.
- **Queries** — Iterate over entities with given component sets.
- **Components** — Transform, Mesh, Material, Collider, RigidBody, custom (e.g. UnlitMeshRef for unlit batching).

### Physics

- **Colliders** — AABB and sphere; triggers (no physical response).
- **RigidBody** — Velocity, kinematic flag.
- **Collision events** — Per-frame list; game code can react.
- **Raycast** — Hit entity, point, distance, normal.

### Audio

- **Sound** — Load from bytes (e.g. WAV/MP3 via rodio).
- **AudioSource** — Playback control (play, stop, volume).

### Input and time

- **Input** — Key and mouse state, cursor delta (when captured), scroll.
- **Time** — Delta time and frame count.

## Use cases

- **Prototypes and small games** — Get a window, 3D scene, and systems running quickly.
- **Learning** — See how wgpu, ECS, and a game loop fit together in Rust.
- **Production-style apps** — Use `Result` and surface handling for robust behavior across drivers and platforms.
