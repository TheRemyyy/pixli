<div align="center">

# Pixli

**A simple yet powerful 3D game engine in Rust**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

*ECS, wgpu rendering, physics, and audio — production-ready error handling and surface recovery*

[Overview](#overview) • [Requirements](#requirements) • [Installation](#installation) • [Usage](#usage) • [Project Structure](#project-structure) • [Documentation](#documentation) • [Contributing](#contributing)

</div>

---

## <a id="overview"></a>Overview

Pixli is a 3D game engine built in Rust with **wgpu** for cross-platform graphics (Vulkan, DirectX 12, Metal). It provides an entity-component system (ECS), PBR lighting with shadows, SSAO, bloom, unlit/lit pipelines, physics (AABB/sphere colliders, rigid bodies), and audio via rodio.

### Key Features

- **Rendering** — Lit and unlit pipelines, directional shadows, SSAO, bloom, MSAA, sky gradient, fog
- **ECS** — Entity/component world, queries, spawn/despawn
- **Physics** — Box and sphere colliders, rigid bodies, collision events, raycasting
- **Audio** — Sound loading and playback (rodio)
- **Input** — Keyboard, mouse, cursor capture
- **Production-ready** — `Result`-based API, no unwraps on user paths, GPU/surface loss handling (Lost, Outdated, Timeout, OutOfMemory)

## <a id="requirements"></a>Requirements

- **Rust** 1.75 or later
- **GPU** with Vulkan 1.2, DirectX 12, or Metal support
- **Windows / Linux / macOS** (wgpu backends)

## <a id="installation"></a>Installation

```bash
git clone https://github.com/your-username/pixli.git
cd pixli
cargo build --release
```

## <a id="usage"></a>Usage

### Quick start

```rust
use pixli::prelude::*;

fn main() -> pixli::Result<()> {
    App::new()
        .with_title("My Game")
        .with_size(1280, 720)
        .add_startup_system(setup)
        .add_system(update)
        .run()
}

fn setup(world: &mut World, renderer: &mut Renderer) {
    world.spawn()
        .with(Transform::from_position(Vec3::ZERO))
        .with(Mesh::cube(1.0))
        .with(Material::color(Color::RED));
    renderer.camera.position = Vec3::new(0.0, 2.0, 5.0);
}

fn update(state: &mut GameState) {
    let (world, input, time, renderer, ..) = (
        state.world,
        state.input,
        state.time,
        state.renderer,
        (),
    );
    // Game logic
}
```

### Shooter example

```bash
cargo run --example shooter
```

Controls: WASD move, mouse aim, LMB fire, Space jump, ESC release mouse or quit.

### Error handling

`App::run()` returns `Result<(), Error>`. Handle initialization failures (no GPU, window creation, etc.) in `main`:

```rust
fn main() -> pixli::Result<()> {
    App::new().with_title("Game").add_system(update).run()
}
// or
fn main() {
    if let Err(e) = App::new().with_title("Game").add_system(update).run() {
        eprintln!("Fatal: {}", e);
        std::process::exit(1);
    }
}
```

## <a id="project-structure"></a>Project Structure

```
pixli/
├── src/
│   ├── lib.rs           # Library root, prelude
│   ├── app.rs           # App builder, event loop, surface/device init
│   ├── error.rs         # Error type and Result
│   ├── ecs/             # Entity, World, Query, components
│   ├── math/            # Vec2/3/4, Mat4, Quat, Transform, Color
│   ├── physics/         # Collider, RigidBody, Physics, raycast
│   ├── renderer/        # wgpu pipelines, meshes, materials, camera, light
│   ├── audio.rs         # Audio, Sound, AudioSource
│   ├── input.rs         # Input, KeyCode, MouseButton
│   ├── time.rs          # Time, delta, frame count
│   └── window.rs        # Window config
├── examples/
│   └── shooter/         # FPS example (config, meshes, systems)
├── docs/                # Documentation
└── README.md
```

## <a id="documentation"></a>Documentation

- [**Overview**](docs/overview.md) — Philosophy and features
- [**System architecture**](docs/architecture/system_overview.md) — App loop, render pipeline, ECS
- [**Changelog**](CHANGELOG.md) — Version history

Generate API docs:

```bash
cargo doc --open
```

## <a id="contributing"></a>Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for coding standards, testing, and the PR process.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

<div align="center">
<sub>Built with Rust and wgpu</sub>
</div>
