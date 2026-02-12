//! Game components: player, enemies, health, weapon.

use pixli::ecs::Entity;

/// Player: FPS camera state (gravity, jump).
pub struct PlayerController {
    pub velocity_y: f32,
    pub is_grounded: bool,
}

/// Enemy (target to shoot).
pub struct Enemy;

/// Entity health (enemies take damage on hit).
#[allow(dead_code)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

/// Weapon state: fire rate and last shot time.
pub struct WeaponState {
    pub last_shot_time: f32,
    pub fire_rate: f32,  // Seconds between shots
}

impl WeaponState {
    pub fn new(fire_rate: f32) -> Self {
        Self {
            last_shot_time: -100.0,
            fire_rate,
        }
    }

    pub fn can_fire(&self, current_time: f32) -> bool {
        current_time - self.last_shot_time >= self.fire_rate
    }
}

/// Reference to the player entity (for raycast exclusion).
pub struct PlayerEntity(pub Entity);

/// ViewModel: weapon in hand (follows camera).
pub struct ViewModel;

/// Recoil: current weapon pullback amount (decayed each frame).
pub struct RecoilAmount(pub f32);

/// Muzzle flash: visibility timer (set by shooting_system on fire).
pub struct MuzzleFlash(pub f32);

/// Crosshair: screen center reticle (entity in front of camera).
pub struct Crosshair;

/// Tracer beam: short lived ray from muzzle to hit (remaining time).
pub struct Tracer(pub f32);

/// Singleton: mesh ID for tracer (thin box scaled to ray length).
pub struct TracerMeshId(pub u64);

/// Entity ignored by raycast (weapons, crosshair, tracer).
pub struct RaycastIgnore;
