//! Camera: view and projection.

use crate::math::{Vec3, Mat4, Quat};

/// Camera.
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,      // Field of view in radians.
    pub aspect: f32,   // Aspect ratio.
    pub near: f32,     // Near clipping plane.
    pub far: f32,      // Far clipping plane.
    
    // Euler angles for easier control.
    pub yaw: f32,      // Rotation around Y axis.
    pub pitch: f32,    // Rotation around X axis.
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, 5.0),
            rotation: Quat::IDENTITY,
            fov: std::f32::consts::FRAC_PI_4, // 45 degrees.
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            yaw: -std::f32::consts::FRAC_PI_2, // Looking at negative Z.
            pitch: 0.0,
        }
    }

    /// Create camera at position.
    pub fn at(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Get forward direction.
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalized()
    }

    /// Get right direction (fallback only when looking straight up or down).
    pub fn right(&self) -> Vec3 {
        let fwd = self.forward();
        let r = fwd.cross(Vec3::UP);
        let len_sq = r.length_squared();
        // Fallback only when forward is nearly (0, ±1, 0).
        if len_sq < 1e-10 {
            // Singularity: looking straight up or down; use axis not parallel to forward.
            fwd.cross(Vec3::RIGHT).normalized()
        } else {
            r.normalized()
        }
    }

    /// Get up direction (always perpendicular to forward and right).
    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalized()
    }

    /// Get horizontal forward (for walking).
    pub fn forward_horizontal(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalized()
    }

    /// Get horizontal right (for walking).
    pub fn right_horizontal(&self) -> Vec3 {
        Vec3::new(
            (self.yaw + std::f32::consts::FRAC_PI_2).cos(),
            0.0,
            (self.yaw + std::f32::consts::FRAC_PI_2).sin(),
        ).normalized()
    }

    /// Get view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        let forward = self.forward();
        let target = self.position + forward;
        Mat4::look_at(self.position, target, Vec3::UP)
    }

    /// Get projection matrix.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov, self.aspect, self.near, self.far)
    }

    /// Get view projection matrix.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Look at a target position.
    pub fn look_at(&mut self, target: Vec3) {
        let direction = (target - self.position).normalized();
        self.pitch = direction.y.asin();
        self.yaw = direction.z.atan2(direction.x);
    }

    /// Process mouse movement.
    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        self.yaw += delta_x * sensitivity;
        self.pitch -= delta_y * sensitivity;
        
        // Clamp pitch to avoid flipping.
        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
    }

    /// Move camera with FPS style controls.
    pub fn fly_move(&mut self, forward: f32, right: f32, up: f32, speed: f32) {
        let movement = self.forward() * forward + self.right() * right + Vec3::UP * up;
        self.position += movement * speed;
    }

    /// Move camera with walking controls (locked to XZ plane).
    pub fn walk_move(&mut self, forward: f32, right: f32, speed: f32) {
        let movement = self.forward_horizontal() * forward + self.right_horizontal() * right;
        self.position.x += movement.x * speed;
        self.position.z += movement.z * speed;
    }

    /// Set position.
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Set field of view in degrees.
    pub fn set_fov_degrees(&mut self, degrees: f32) {
        self.fov = degrees.to_radians();
    }

    /// Set aspect ratio from dimensions.
    pub fn set_aspect_ratio(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height.max(1) as f32;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// First person camera controller.
#[allow(dead_code)]
pub struct FpsCameraController {
    pub speed: f32,
    pub sprint_multiplier: f32,
    pub sensitivity: f32,
    pub height: f32,
    pub velocity_y: f32,
    pub is_grounded: bool,
}

#[allow(dead_code)]
impl FpsCameraController {
    pub fn new() -> Self {
        Self {
            speed: 5.0,
            sprint_multiplier: 2.0,
            sensitivity: 0.001,
            height: 1.7,
            velocity_y: 0.0,
            is_grounded: true,
        }
    }

    pub fn update(&mut self, camera: &mut Camera, input: &crate::input::Input, delta: f32) {
        use crate::input::KeyCode;

        // Mouse look.
        if input.is_mouse_captured() {
            let delta_mouse = input.mouse_delta();
            camera.process_mouse(delta_mouse.x, delta_mouse.y, self.sensitivity);
        }

        // Movement.
        let sprint = input.key_pressed(KeyCode::ShiftLeft) || input.key_pressed(KeyCode::ShiftRight);
        let speed = self.speed * if sprint { self.sprint_multiplier } else { 1.0 } * delta;

        let move_vec = input.movement_vector_normalized();
        camera.walk_move(move_vec.y, move_vec.x, speed);

        // Jumping.
        if input.key_just_pressed(KeyCode::Space) && self.is_grounded {
            self.velocity_y = 8.0;
            self.is_grounded = false;
        }

        // Gravity.
        self.velocity_y -= 20.0 * delta;
        camera.position.y += self.velocity_y * delta;

        // Ground collision.
        if camera.position.y <= self.height {
            camera.position.y = self.height;
            self.velocity_y = 0.0;
            self.is_grounded = true;
        }
    }
}

impl Default for FpsCameraController {
    fn default() -> Self {
        Self::new()
    }
}
