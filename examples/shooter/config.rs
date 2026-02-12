//! Shooter example constants and configuration.

use pixli::math::Vec3;

/// Arena half size (from center).
pub const ARENA_HALF: f32 = 40.0;
/// Camera and player eye height.
pub const CAMERA_HEIGHT: f32 = 1.7;

/// Enemy spawn positions (x, _y, z).
pub const ENEMY_POSITIONS: [(f32, f32, f32); 12] = [
    (-20.0, 0.6, 25.0),
    (22.0, 0.6, -20.0),
    (-15.0, 0.6, -15.0),
    (18.0, 0.6, 18.0),
    (-28.0, 0.6, 0.0),
    (25.0, 0.6, 10.0),
    (0.0, 0.6, 28.0),
    (-25.0, 0.6, -25.0),
    (30.0, 0.6, -15.0),
    (-10.0, 0.6, 30.0),
    (12.0, 0.6, -28.0),
    (-32.0, 0.6, 18.0),
];

/// Enemy box dimensions (width, height, depth).
pub const ENEMY_WIDTH: f32 = 0.6;
pub const ENEMY_HEIGHT: f32 = 1.2;
pub const ENEMY_DEPTH: f32 = 0.6;

/// Enemy scale vector for Transform.
pub fn enemy_scale() -> Vec3 {
    Vec3::new(ENEMY_WIDTH, ENEMY_HEIGHT, ENEMY_DEPTH)
}
