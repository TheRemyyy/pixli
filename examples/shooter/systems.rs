//! Game systems: player movement, shooting, damage.

use pixli::ecs::Entity;
use pixli::math::{Quat, Transform, Vec3};
use pixli::prelude::*;

use crate::components::{
    Crosshair, Health, MuzzleFlash, PlayerController, PlayerEntity, RaycastIgnore, RecoilAmount,
    Tracer, TracerMeshId, ViewModel, WeaponState,
};
use crate::config::{ARENA_HALF, CAMERA_HEIGHT};

const GRAVITY: f32 = 20.0;
const JUMP_FORCE: f32 = 8.0;
const WALK_SPEED: f32 = 6.0;
const MOUSE_SENSITIVITY: f32 = 0.001;
const GUN_DAMAGE: i32 = 34;
const RAYCAST_MAX_DIST: f32 = 600.0;
const RAYCAST_ORIGIN_OFFSET: f32 = 0.3;

const GUN_OFFSET_FWD: f32 = 0.38;
const GUN_OFFSET_RIGHT: f32 = 0.14;
const GUN_OFFSET_DOWN: f32 = 0.18;
const RECOIL_IMPULSE: f32 = 0.12;
const RECOIL_DECAY: f32 = 0.72;
const MUZZLE_FLASH_DURATION: f32 = 0.06;
const BARREL_LENGTH: f32 = 0.38;
const CROSSHAIR_DIST: f32 = 0.5;
const VIEWMODEL_SMOOTH: f32 = 1.0;
const TRACER_THICKNESS: f32 = 0.012;
const TRACER_DURATION: f32 = 0.07;

/// Raycast excluding player and RaycastIgnore entities; returns closest hit.
fn raycast_exclude_player(
    world: &World,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    player_entity: Entity,
) -> Option<(Entity, Vec3, f32, Vec3)> {
    let direction = direction.normalized();
    let mut closest: Option<(Entity, Vec3, f32, Vec3)> = None;

    for entity in world.query::<(&Transform, &Collider)>().iter() {
        if entity.id() == player_entity.id() && entity.generation() == player_entity.generation() {
            continue;
        }
        if world.has::<crate::components::RaycastIgnore>(entity) {
            continue;
        }
        let transform = world.get::<Transform>(entity)?;
        let collider = world.get::<Collider>(entity)?;

        if let Some((hit_point, distance, normal)) =
            collider.raycast(origin, direction, transform.position)
        {
            if distance <= max_distance && distance > 0.001 {
                if closest.is_none() || distance < closest.unwrap().2 {
                    closest = Some((entity, hit_point, distance, normal));
                }
            }
        }
    }

    closest
}

/// Player movement and aim; sync camera to player.
pub fn player_controller_system(state: &mut GameState) {
    let delta = state.time.delta();

    let player_entity = match state
        .world
        .query::<(&PlayerController, &Transform, &RigidBody)>()
        .iter()
        .next()
    {
        Some(e) => e,
        None => return,
    };

    let (velocity_y, is_grounded) = {
        let c = state.world.get::<PlayerController>(player_entity).unwrap();
        (c.velocity_y, c.is_grounded)
    };

    if state.input.is_mouse_captured() {
        let delta_mouse = state.input.mouse_delta();
        state
            .renderer
            .camera
            .process_mouse(delta_mouse.x, delta_mouse.y, MOUSE_SENSITIVITY);
    }

    let move_vec = state.input.movement_vector_normalized();
    let cam = &state.renderer.camera;
    let movement = cam.forward_horizontal() * move_vec.y + cam.right_horizontal() * move_vec.x;

    let mut new_velocity_y = velocity_y;
    let mut new_grounded = is_grounded;

    if state.input.key_just_pressed(KeyCode::Space) && is_grounded {
        new_velocity_y = JUMP_FORCE;
        new_grounded = false;
    }
    new_velocity_y -= GRAVITY * delta;

    if let Some(rb) = state.world.get_mut::<RigidBody>(player_entity) {
        rb.velocity.x = movement.x * WALK_SPEED;
        rb.velocity.z = movement.z * WALK_SPEED;
        rb.velocity.y = new_velocity_y;
    }

    let pos = state
        .world
        .get::<Transform>(player_entity)
        .unwrap()
        .position;
    state.renderer.camera.position = pos;

    if pos.y <= CAMERA_HEIGHT {
        if let Some(t) = state.world.get_mut::<Transform>(player_entity) {
            t.position.y = CAMERA_HEIGHT;
        }
        if let Some(rb) = state.world.get_mut::<RigidBody>(player_entity) {
            rb.velocity.y = 0.0;
        }
        new_grounded = true;
    }

    if let Some(c) = state.world.get_mut::<PlayerController>(player_entity) {
        c.velocity_y = new_velocity_y;
        c.is_grounded = new_grounded;
    }
}

/// Shooting: LMB, raycast, Health damage, enemy death.
pub fn shooting_system(state: &mut GameState) {
    let player_entity = match state.world.query::<(&PlayerEntity,)>().iter().next() {
        Some(e) => {
            let pe = state.world.get::<PlayerEntity>(e).unwrap();
            pe.0
        }
        None => return,
    };

    let can_fire = state
        .world
        .get::<WeaponState>(player_entity)
        .map(|ws| ws.can_fire(state.time.elapsed()))
        .unwrap_or(false);

    if !state.input.mouse_button_just_pressed(MouseButton::Left) || !can_fire {
        return;
    }

    let cam = &state.renderer.camera;
    let origin = cam.position + cam.forward() * RAYCAST_ORIGIN_OFFSET;
    let direction = cam.forward();

    let hit = raycast_exclude_player(
        state.world,
        origin,
        direction,
        RAYCAST_MAX_DIST,
        player_entity,
    );

    if let Some((entity, _hit_point, _dist, _hit_normal)) = hit {
        if let Some(health) = state.world.get_mut::<Health>(entity) {
            health.take_damage(GUN_DAMAGE);
            if health.is_dead() {
                state.world.despawn(entity);
            }
        }
    }

    if let Some(ws) = state.world.get_mut::<WeaponState>(player_entity) {
        ws.last_shot_time = state.time.elapsed();
    }

    // Tracer: short lived ray from muzzle to hit (or max distance).
    let end_point = hit
        .map(|(_, hit_point, _, _)| hit_point)
        .unwrap_or_else(|| origin + direction * RAYCAST_MAX_DIST);
    let length = (end_point - origin).length();
    let midpoint = (origin + end_point) * 0.5;
    let rot = Quat::from_rotation_axes(cam.right(), cam.up(), direction);
    let scale = Vec3::new(TRACER_THICKNESS, TRACER_THICKNESS, length);

    let tracer_entity = state.world.query::<(&TracerMeshId,)>().iter().next();
    let tracer_mesh_id =
        tracer_entity.and_then(|e| state.world.get::<TracerMeshId>(e).map(|t| t.0));
    if let Some(mesh_id) = tracer_mesh_id {
        state
            .world
            .spawn()
            .with(Tracer(TRACER_DURATION))
            .with(RaycastIgnore)
            .with(Transform::from_position_rotation_scale(
                midpoint, rot, scale,
            ))
            .with(UnlitMeshRef(mesh_id))
            .build();
    }

    // Recoil and muzzle flash (we just fired).
    let viewmodel_entities: Vec<Entity> = state
        .world
        .query::<(&ViewModel, &RecoilAmount)>()
        .iter()
        .collect();
    for e in viewmodel_entities {
        if let Some(ra) = state.world.get_mut::<RecoilAmount>(e) {
            ra.0 += RECOIL_IMPULSE;
        }
    }
    let muzzle_entities: Vec<Entity> = state.world.query::<(&MuzzleFlash,)>().iter().collect();
    for e in muzzle_entities {
        if let Some(mf) = state.world.get_mut::<MuzzleFlash>(e) {
            mf.0 = MUZZLE_FLASH_DURATION;
        }
    }
}

/// Tracer: decay timer and despawn when expired (laser disappears).
pub fn tracer_system(state: &mut GameState) {
    let delta = state.time.delta();
    for entity in state.world.query::<(&Tracer,)>().iter().collect::<Vec<_>>() {
        if let Some(t) = state.world.get_mut::<Tracer>(entity) {
            t.0 -= delta;
        }
    }
    let to_despawn: Vec<Entity> = state
        .world
        .query::<(&Tracer,)>()
        .iter()
        .filter(|e| state.world.get::<Tracer>(*e).map_or(false, |t| t.0 <= 0.0))
        .collect();
    for entity in to_despawn {
        state.world.despawn(entity);
    }
}

/// Rotation aligned to camera axes so object points where you look.
fn camera_rotation(cam: &pixli::renderer::Camera) -> Quat {
    Quat::from_rotation_axes(cam.right(), cam.up(), cam.forward())
}

/// ViewModel: smooth camera follow, recoil and muzzle flash.
pub fn viewmodel_system(state: &mut GameState) {
    let delta = state.time.delta();
    let cam = &state.renderer.camera;

    let fwd = cam.forward();
    let base_pos = cam.position + fwd * GUN_OFFSET_FWD + cam.right() * GUN_OFFSET_RIGHT
        - cam.up() * GUN_OFFSET_DOWN;
    let target_rot = camera_rotation(cam);

    let viewmodel_entities: Vec<Entity> = state
        .world
        .query::<(&ViewModel, &RecoilAmount, &Transform)>()
        .iter()
        .collect();
    for entity in viewmodel_entities {
        let recoil = state.world.get::<RecoilAmount>(entity).unwrap().0;
        let target_pos = base_pos + cam.forward() * (-recoil);

        if let Some(t) = state.world.get_mut::<Transform>(entity) {
            t.position = t.position.lerp(target_pos, VIEWMODEL_SMOOTH);
            t.rotation = t.rotation.slerp(target_rot, VIEWMODEL_SMOOTH);
            t.scale = Vec3::new(1.0, 1.0, 1.0);
        }
        if let Some(ra) = state.world.get_mut::<RecoilAmount>(entity) {
            ra.0 *= RECOIL_DECAY;
            if ra.0 < 0.002 {
                ra.0 = 0.0;
            }
        }
    }

    let muzzle_pos = base_pos + cam.forward() * (-BARREL_LENGTH);
    let muzzle_entities: Vec<Entity> = state
        .world
        .query::<(&MuzzleFlash, &Transform)>()
        .iter()
        .collect();
    for entity in muzzle_entities {
        let mf = state.world.get::<MuzzleFlash>(entity).unwrap().0;
        if let Some(t) = state.world.get_mut::<Transform>(entity) {
            t.position = muzzle_pos;
            t.rotation = target_rot;
            let s = if mf > 0.0 { 0.1 } else { 0.0 };
            t.scale = Vec3::splat(s);
        }
        if let Some(mf_comp) = state.world.get_mut::<MuzzleFlash>(entity) {
            mf_comp.0 = (mf_comp.0 - delta).max(0.0);
        }
    }
}

/// Crosshair: always in front of camera.
pub fn crosshair_system(state: &mut GameState) {
    let cam = &state.renderer.camera;
    let pos = cam.position + cam.forward() * CROSSHAIR_DIST;
    let rot = camera_rotation(cam);
    let scale = Vec3::splat(0.12);

    let entities: Vec<Entity> = state
        .world
        .query::<(&Crosshair, &Transform)>()
        .iter()
        .collect();
    for e in entities {
        if let Some(t) = state.world.get_mut::<Transform>(e) {
            t.position = pos;
            t.rotation = rot;
            t.scale = scale;
        }
    }
}

/// Clamp player to arena bounds.
pub fn player_bounds_system(state: &mut GameState) {
    let p = &mut state.renderer.camera.position;
    p.x = p.x.clamp(-ARENA_HALF, ARENA_HALF);
    p.z = p.z.clamp(-ARENA_HALF, ARENA_HALF);
    if p.y < CAMERA_HEIGHT {
        p.y = CAMERA_HEIGHT;
    }
}
