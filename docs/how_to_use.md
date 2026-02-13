# How to make a game with Pixli

This guide shows how to use Pixli step by step: from a minimal window to ECS, physics, rendering, and input. Use it together with `cargo doc --open` for full API details.

---

## 1. Minimal game

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
    // Spawn a red cube at origin
    world.spawn()
        .with(Transform::from_position(Vec3::ZERO))
        .with(Mesh::cube(1.0))
        .with(Material::color(Color::RED));
    // Camera 5 units back
    renderer.camera.position = Vec3::new(0.0, 2.0, 5.0);
}

fn update(state: &mut GameState) {
    // Runs every frame; put game logic here
}
```

- **`App`** — Builder. Chain `with_title`, `with_size`, `with_fullscreen`, `with_vsync`, then `add_startup_system` / `add_system`, finally `run()`.
- **`add_startup_system(F)`** — Runs once after the window and GPU are ready. Signature: `fn(&mut World, &mut Renderer)`.
- **`add_system(F)`** — Runs every frame. Signature: `fn(&mut GameState)`.
- **`App::run()`** — Returns `Result<(), Error>`. Handle errors in `main` (e.g. no GPU, no window).

---

## 2. What you get each frame: `GameState`

Inside `update(state: &mut GameState)` you have:

| Field       | Type        | What it is |
|------------|-------------|------------|
| `state.world`   | `&mut World`   | All entities and components. Spawn, query, get, despawn. |
| `state.input`   | `&Input`       | Keyboard, mouse, movement axes. Read-only. |
| `state.time`     | `&Time`        | Delta time, elapsed, FPS, fixed timestep. Read-only. |
| `state.renderer` | `&mut Renderer` | Camera, lights, fog, graphics settings. |
| `state.physics`  | `&mut Physics`  | Gravity, collision events; `update` is called by the engine. |
| `state.audio`    | `&Audio`        | Play sounds (real playback via rodio). |
| `state.window`   | `&Window`       | Window handle (e.g. for cursor grab). |

You never create `GameState` yourself; the engine passes it into your systems.

---

## 3. ECS: entities and components

### Spawn an entity

```rust
let entity = world.spawn()
    .with(Transform::from_position(Vec3::new(1.0, 0.0, 0.0)))
    .with(Mesh::cube(1.0))
    .with(Material::blue())
    .with(Collider::sphere(0.5))
    .with(RigidBody::dynamic())
    .build();
```

- **`world.spawn()`** — Returns `EntityBuilder`. Add components with `.with(component)`, then `.build()` to get the `Entity`.
- **Built-in components** you can `.with()`: `Transform`, `Mesh`, `Material`, `Collider`, `RigidBody`. For **unlit** (vertex-colored) meshes use `UnlitMeshRef(mesh_id)` (see Unlit rendering below).

### Query entities

```rust
// All entities that have both Transform and Mesh
for entity in state.world.query::<(&Transform, &Mesh)>().iter() {
    let transform = state.world.get::<Transform>(entity).unwrap();
    // ...
}

// Count
let n = state.world.query::<(&RigidBody, &Collider)>().count();
```

- **`world.query::<(&A, &B)>()`** — Returns a query. Use `.iter()` for entities, `.count()` for count. Only entities that have **all** listed components are included.
- **`world.get::<T>(entity)`** — `Option<&T>`.  
- **`world.get_mut::<T>(entity)`** — `Option<&mut T>`.  
- **`world.has::<T>(entity)`** — `bool`.  
- **`world.despawn(entity)`** — Remove entity and all its components.  
- **`world.add_component(entity, component)`** / **`world.remove_component::<T>(entity)`** — Add or remove a component from an existing entity.

### Component types (quick reference)

| Component     | Use |
|--------------|-----|
| `Transform`  | Position, rotation, scale. Required for rendering and physics. |
| `Mesh`       | Lit mesh (PBR). Use with `Material`. |
| `Material`   | Color, metallic, roughness, emission. For lit pipeline. |
| `UnlitMeshRef(id)` | Reference to an uploaded unlit mesh (see Renderer). |
| `Collider`   | Shape (box, sphere, capsule), offset, trigger. |
| `RigidBody`  | Velocity, mass, kinematic/static/dynamic, gravity. |

---

## 4. Math

Import from prelude: `Vec2`, `Vec3`, `Vec4`, `Mat4`, `Quat`, `Transform`, `Color`.

### Vectors

- **`Vec3::ZERO`**, **`Vec3::ONE`**, **`Vec3::UP`**, **`Vec3::DOWN`**, **`Vec3::LEFT`**, **`Vec3::RIGHT`**, **`Vec3::FORWARD`**, **`Vec3::BACK`**.
- **`Vec3::new(x, y, z)`**, **`v.normalized()`**, **`v.length()`**, **`v.length_squared()`**, **`a.dot(b)`**, **`a.cross(b)`**, **`a + b`**, **`a * s`**.

### Transform

- **`Transform::new()`** — Identity at origin.  
- **`Transform::from_position(pos)`**, **`from_position_rotation`**, **`from_position_rotation_scale`**.  
- **`transform.position`**, **`transform.rotation`** (`Quat`), **`transform.scale`**.  
- **`transform.matrix()`** — World matrix.  
- **`transform.forward()`**, **`right()`**, **`up()`** — Directions from rotation.  
- **`transform.translate(offset)`**, **`look_at(target)`** (takes up vector).

### Color

- **`Color::new(r, g, b, a)`**, **`Color::RED`**, **`GREEN`**, **`BLUE`**, **`WHITE`**, **`BLACK`**, **`GRAY`**, etc.  
- **`c.lerp(other, t)`**.

---

## 5. Physics

- **`Physics::update(world, delta)`** is called by the engine each frame. You don’t call it from your systems.
- **`state.physics.gravity`** — Default `Vec3::new(0, -9.81, 0)`. Change if needed.
- **`state.physics.collision_events`** — Cleared each frame; list of `CollisionEvent { entity_a, entity_b, point, normal, penetration }`. Read in your systems to react to collisions.

### Collider

- **`Collider::box_collider(size: Vec3)`**, **`Collider::sphere(radius)`**, **`Collider::capsule(radius, height)`**.  
- **`.with_offset(offset)`**, **`.as_trigger()`** (no physical response).

### RigidBody

- **`RigidBody::dynamic()`** — Normal physics, gravity.  
- **`RigidBody::kinematic()`** — Moved by code only.  
- **`RigidBody::statik()`** — Immovable (e.g. floor).  
- **`.with_mass(m)`**, **`.with_drag(d)`**, **`.with_velocity(v)`**, **`.without_gravity()`**.  
- **`rb.velocity`** — Read/write.  
- **`rb.add_force(force)`**, **`add_force_at_position(force, position, center_of_mass)`** (optional).

### Raycast

- **`state.physics.raycast(world, origin, direction, max_distance)`** — Returns `Option<(Entity, hit_point, distance, normal)>`.

---

## 6. Renderer: camera, meshes, materials, lights

### Camera

- **`state.renderer.camera`** — Main camera.  
- **`camera.position`**, **`camera.yaw`**, **`camera.pitch`** (radians).  
- **`camera.forward()`**, **`camera.right()`**, **`camera.up()`** — Directions.  
- **`camera.forward_horizontal()`**, **`camera.right_horizontal()`** — For FPS movement (no pitch).  
- **`camera.process_mouse(delta_x, delta_y, sensitivity)`** — Apply mouse delta (e.g. from `state.input.mouse_delta()`).  
- **`camera.view_matrix()`**, **`projection_matrix()`**, **`view_projection_matrix()`**.  
- **`camera.look_at(target)`** — Orient camera toward a point.

### Lit meshes (PBR)

- Add **`Mesh`** + **`Material`** (and **`Transform`**) to an entity. The engine uploads meshes at startup and draws them with the lit pipeline (one directional light, shadows, fog).
- **Mesh**: **`Mesh::cube(size)`**, **`Mesh::sphere(radius, segments)`**, **`Mesh::plane(width, depth)`**, **`Mesh::cylinder(radius, height, segments)`**, **`Mesh::cone(radius, height, segments)`**, **`Mesh::torus(...)`**, **`Mesh::from_vertices(vertices)`**.
- **Material**: **`Material::color(color)`**, **`Material::red()`**, **`Material::green()`**, etc., **`Material::metallic(color, metallic)`**, **`Material::emissive(color, strength)`**, **`.with_roughness(r)`**, **`.with_metallic(m)`**.

### Unlit meshes (vertex color, no lighting)

1. In **startup**: create an **`UnlitMesh`** (e.g. from vertices), then **`let id = renderer.upload_unlit_mesh(&mesh)`**.
2. Spawn entities with **`UnlitMeshRef(id)`** and **`Transform`** (no `Mesh`/`Material`). They are drawn with the unlit pipeline (batched, fog).

### Lights and atmosphere

- **`state.renderer.directional_light = Some(Light::directional(direction, color, intensity))`** — One directional light (shadows supported).  
- **`state.renderer.ambient_light`**, **`clear_color`**, **`fog_color`**, **`fog_start`**, **`fog_end`**.  
- **`state.renderer.graphics`** — `GraphicsSettings`: e.g. MSAA, SSAO, bloom toggles.

---

## 7. Input

- **`state.input.key_pressed(KeyCode::KeyW)`** — Currently held.  
- **`state.input.key_just_pressed(KeyCode::Space)`** — Pressed this frame.  
- **`state.input.key_just_released(KeyCode::Escape)`** — Released this frame.  
- **`state.input.movement_vector_normalized()`** — Vec2 from WASD/arrows, length 0 or 1.  
- **`state.input.axis_horizontal()`**, **`axis_vertical()`** — -1, 0, or 1.  
- **`state.input.mouse_button_pressed(MouseButton::Left)`**, **`mouse_button_just_pressed`**, **`mouse_button_just_released`**.  
- **`state.input.mouse_position()`** — Window coordinates.  
- **`state.input.mouse_delta()`** — Movement since last frame (use when mouse is captured for FPS look).  
- **`state.input.scroll_delta()`**.  
- **`state.input.is_mouse_captured()`** — Whether cursor is captured (e.g. FPS mode).  
- **KeyCode** — From `winit` (e.g. `KeyCode::KeyW`, `KeyCode::Space`, `KeyCode::Escape`).

---

## 8. Time

- **`state.time.delta()`** — Seconds since last frame (scaled by `time_scale`).  
- **`state.time.elapsed()`** — Total seconds since start.  
- **`state.time.frame_count()`**.  
- **`state.time.fps()`**, **`state.time.fps_int()`**.  
- **`state.time.time_scale()`**, **`state.time.set_time_scale(s)`** — Slow motion / pause (0 = pause).  
- **`state.time.fixed_steps()`** — Number of fixed-dt steps to run this frame (e.g. for fixed physics).  
- **`state.time.fixed_delta()`** — Fixed timestep length.  
- **Timer** (in `pixli::time`): **`Timer::once(duration)`**, **`Timer::repeating(duration)`**, **`timer.tick(delta)`** returns `true` when the timer fires.

---

## 9. Audio

- **`AudioSource::load(path)`** — Load from file (WAV, OGG, MP3, FLAC).  
- **`state.audio.play(&source)`** — Returns **`Sound`** (play, stop, set_volume).  
- **`state.audio.play_volume(&source, volume)`**, **`play_music(&source)`** (looping).  
- **`state.audio.set_master_volume(v)`**, **set_music_volume**, **set_sfx_volume**.  
- Note: playback implementation may be placeholder; API is stable.

---

## 10. Error handling

- **`App::run()`** returns **`Result<(), Error>`**. Use in `main`:

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

- **`Error`** variants cover window creation, no GPU adapter, device request, surface errors. See `pixli::Error` in `cargo doc`.

---

## 11. Prelude and API docs

- **`use pixli::prelude::*;`** — Imports the usual types: `App`, `GameState`, `World`, `Transform`, `Mesh`, `Material`, `Vec3`, `Color`, `Input`, `Time`, `Camera`, `Renderer`, `Physics`, `Collider`, `RigidBody`, `Query`, `Entity`, etc.
- Full API: run **`cargo doc --open`** and browse the crate root and modules (`app`, `ecs`, `math`, `physics`, `renderer`, `input`, `time`, `audio`).

---

## 12. Example: shooter

Run the FPS example:

```bash
cargo run --example shooter
```

Controls: WASD move, mouse aim, LMB fire, Space jump, ESC release mouse / quit.  
Code is in **`examples/shooter/`** — good reference for movement, shooting, view model, unlit meshes, and raycast.
