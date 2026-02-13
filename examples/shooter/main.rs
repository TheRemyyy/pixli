//! Shooter: first person shooter example.
//!
//! Run with: `cargo run --example shooter`
//!
//! Controls: WASD move, mouse aim, LMB fire, Space jump, ESC release mouse or quit.

use pixli::prelude::*;
use pixli::renderer::UnlitMeshRef;

mod components;
mod config;
mod map;
mod meshes;
mod sound;
mod systems;

use components::*;
use config::*;
use map::*;
use meshes::*;
use sound::*;
use systems::*;

fn main() -> pixli::Result<()> {
    App::new()
        .with_title("Shooter")
        .with_size(1280, 720)
        .with_vsync(false)
        .add_startup_system(setup)
        .add_system(player_controller_system)
        .add_system(shooting_system)
        .add_system(tracer_system)
        .add_system(viewmodel_system)
        .add_system(crosshair_system)
        .add_system(player_bounds_system)
        .run()
}

fn setup(world: &mut World, renderer: &mut Renderer) {
    setup_renderer(renderer);
    spawn_map(world, renderer);

    // Enemy mesh (red)
    let enemy_color: [f32; 3] = [0.85, 0.2, 0.2];
    let enemy_mesh_id = renderer.upload_unlit_mesh(&unlit_cube(1.0, enemy_color));

    // Player
    let player_pos = Vec3::new(0.0, config::CAMERA_HEIGHT, 8.0);
    let player_entity = world
        .spawn()
        .with(PlayerController {
            velocity_y: 0.0,
            is_grounded: true,
        })
        .with(Transform::from_position(player_pos))
        .with(RigidBody::new().without_gravity())
        .with(
            Collider::box_collider(Vec3::new(0.8, config::CAMERA_HEIGHT, 0.8))
                .with_offset(Vec3::new(0.0, -config::CAMERA_HEIGHT * 0.5, 0.0)),
        )
        .with(WeaponState::new(0.12)) // ~8 rounds/s
        .build();

    renderer.camera.position = player_pos;
    renderer.camera.yaw = -90.0_f32.to_radians();
    renderer.camera.pitch = 0.0;

    // Singleton: reference to player entity for raycast exclusion.
    world.spawn().with(PlayerEntity(player_entity)).build();

    // ViewModel: weapon in hand, correct initial position.
    let pistol_mesh_id = renderer.upload_unlit_mesh(&create_pistol_mesh());
    let muzzle_mesh_id = renderer.upload_unlit_mesh(&create_muzzle_flash_mesh());
    let cam = &renderer.camera;
    let fwd = cam.forward();
    let vm_pos = cam.position + fwd * 0.38 + cam.right() * 0.14 - cam.up() * 0.18;
    let vm_rot = Quat::from_rotation_axes(cam.right(), cam.up(), cam.forward());

    world
        .spawn()
        .with(ViewModel)
        .with(RaycastIgnore)
        .with(RecoilAmount(0.0))
        .with(Transform::from_position_rotation(vm_pos, vm_rot))
        .with(UnlitMeshRef(pistol_mesh_id))
        .build();

    world
        .spawn()
        .with(MuzzleFlash(0.0))
        .with(RaycastIgnore)
        .with(Transform::default())
        .with(UnlitMeshRef(muzzle_mesh_id))
        .build();

    // Tracer beam mesh for shot visualization.
    let tracer_mesh_id = renderer.upload_unlit_mesh(&create_tracer_mesh());
    world.spawn().with(TracerMeshId(tracer_mesh_id)).build();

    // Shoot SFX (minimal WAV, no external assets).
    let shoot_source = AudioSource::from_bytes(shoot_wav_bytes());
    world.spawn().with(ShootSound(shoot_source)).build();

    // Crosshair
    let crosshair_mesh_id = renderer.upload_unlit_mesh(&create_crosshair_mesh());

    world
        .spawn()
        .with(Crosshair)
        .with(RaycastIgnore)
        .with(Transform::default())
        .with(UnlitMeshRef(crosshair_mesh_id))
        .build();

    // Enemies: standing targets in the arena.
    let enemy_half_y = ENEMY_HEIGHT * 0.5;
    let enemy_scale = enemy_scale();

    for (x, _y, z) in ENEMY_POSITIONS {
        let center_y = 0.6 + enemy_half_y;
        world
            .spawn()
            .with(Transform::from_position_scale(
                Vec3::new(x, center_y, z),
                enemy_scale,
            ))
            .with(UnlitMeshRef(enemy_mesh_id))
            .with(Enemy)
            .with(Health::new(100))
            .with(Collider::box_collider(enemy_scale))
            .with(RigidBody::statik())
            .build();
    }
}
