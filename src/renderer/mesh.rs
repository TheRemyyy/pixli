//! Mesh: 3D geometry.

use crate::math::{Vec2, Vec3, Color};
use std::sync::atomic::{AtomicU64, Ordering};

static MESH_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Vertex data with tangent for normal mapping.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub uv: Vec2,
    pub color: Color,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::UP,
            tangent: Vec3::RIGHT,
            uv: Vec2::ZERO,
            color: Color::WHITE,
        }
    }
}

/// Mesh component.
#[derive(Clone)]
pub struct Mesh {
    id: u64,
    pub vertices: Vec<Vertex>,
}

impl Mesh {
    /// Create empty mesh.
    pub fn new() -> Self {
        Self {
            id: MESH_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            vertices: Vec::new(),
        }
    }

    /// Create mesh from vertices.
    pub fn from_vertices(vertices: Vec<Vertex>) -> Self {
        Self {
            id: MESH_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            vertices,
        }
    }

    /// Get unique mesh ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Create a cube.
    pub fn cube(size: f32) -> Self {
        let s = size / 2.0;
        let mut vertices = Vec::with_capacity(36);

        // Front face (+Z): tangent is +X.
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let tangent = Vec3::new(1.0, 0.0, 0.0);
        vertices.push(Vertex { position: Vec3::new(-s, -s, s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, -s, s), normal, tangent, uv: Vec2::new(1.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, -s, s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, s, s), normal, tangent, uv: Vec2::new(0.0, 1.0), color: Color::WHITE });

        // Back face (-Z): tangent is -X.
        let normal = Vec3::new(0.0, 0.0, -1.0);
        let tangent = Vec3::new(-1.0, 0.0, 0.0);
        vertices.push(Vertex { position: Vec3::new(s, -s, -s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, -s, -s), normal, tangent, uv: Vec2::new(1.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, s, -s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, -s, -s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, s, -s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, -s), normal, tangent, uv: Vec2::new(0.0, 1.0), color: Color::WHITE });

        // Top face (+Y): tangent is +X.
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let tangent = Vec3::new(1.0, 0.0, 0.0);
        vertices.push(Vertex { position: Vec3::new(-s, s, s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, s), normal, tangent, uv: Vec2::new(1.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, -s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, s, s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, -s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, s, -s), normal, tangent, uv: Vec2::new(0.0, 1.0), color: Color::WHITE });

        // Bottom face (-Y): tangent is +X.
        let normal = Vec3::new(0.0, -1.0, 0.0);
        let tangent = Vec3::new(1.0, 0.0, 0.0);
        vertices.push(Vertex { position: Vec3::new(-s, -s, -s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, -s, -s), normal, tangent, uv: Vec2::new(1.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, -s, s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, -s, -s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, -s, s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, -s, s), normal, tangent, uv: Vec2::new(0.0, 1.0), color: Color::WHITE });

        // Right face (+X): tangent is -Z.
        let normal = Vec3::new(1.0, 0.0, 0.0);
        let tangent = Vec3::new(0.0, 0.0, -1.0);
        vertices.push(Vertex { position: Vec3::new(s, -s, s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, -s, -s), normal, tangent, uv: Vec2::new(1.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, -s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, -s, s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, -s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(s, s, s), normal, tangent, uv: Vec2::new(0.0, 1.0), color: Color::WHITE });

        // Left face (-X): tangent is +Z.
        let normal = Vec3::new(-1.0, 0.0, 0.0);
        let tangent = Vec3::new(0.0, 0.0, 1.0);
        vertices.push(Vertex { position: Vec3::new(-s, -s, -s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, -s, s), normal, tangent, uv: Vec2::new(1.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, s, s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, -s, -s), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, s, s), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE });
        vertices.push(Vertex { position: Vec3::new(-s, s, -s), normal, tangent, uv: Vec2::new(0.0, 1.0), color: Color::WHITE });

        Self::from_vertices(vertices)
    }

    /// Create a sphere.
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();

        for i in 0..=rings {
            let v = i as f32 / rings as f32;
            let phi = v * std::f32::consts::PI;

            for j in 0..=segments {
                let u = j as f32 / segments as f32;
                let theta = u * std::f32::consts::PI * 2.0;

                let x = theta.cos() * phi.sin();
                let y = phi.cos();
                let z = theta.sin() * phi.sin();

                let position = Vec3::new(x * radius, y * radius, z * radius);
                let normal = Vec3::new(x, y, z);
                // Tangent: +U direction (along theta) equals (-sin(theta), 0, cos(theta)), i.e. (-z, 0, x) on equator.
                let tangent = Vec3::new(-z, 0.0, x).normalized();
                let uv = Vec2::new(u, v);

                vertices.push(Vertex { position, normal, tangent, uv, color: Color::WHITE });
            }
        }

        // Create triangles.
        let mut tri_vertices = Vec::new();
        for i in 0..rings {
            for j in 0..segments {
                let first = i * (segments + 1) + j;
                let second = first + segments + 1;

                tri_vertices.push(vertices[first as usize]);
                tri_vertices.push(vertices[second as usize]);
                tri_vertices.push(vertices[(first + 1) as usize]);

                tri_vertices.push(vertices[second as usize]);
                tri_vertices.push(vertices[(second + 1) as usize]);
                tri_vertices.push(vertices[(first + 1) as usize]);
            }
        }

        Self::from_vertices(tri_vertices)
    }

    /// Create a plane.
    pub fn plane(width: f32, depth: f32) -> Self {
        let hw = width / 2.0;
        let hd = depth / 2.0;
        let normal = Vec3::UP;
        let tangent = Vec3::RIGHT;

        let vertices = vec![
            Vertex { position: Vec3::new(-hw, 0.0, -hd), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE },
            Vertex { position: Vec3::new(hw, 0.0, -hd), normal, tangent, uv: Vec2::new(1.0, 0.0), color: Color::WHITE },
            Vertex { position: Vec3::new(hw, 0.0, hd), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE },
            Vertex { position: Vec3::new(-hw, 0.0, -hd), normal, tangent, uv: Vec2::new(0.0, 0.0), color: Color::WHITE },
            Vertex { position: Vec3::new(hw, 0.0, hd), normal, tangent, uv: Vec2::new(1.0, 1.0), color: Color::WHITE },
            Vertex { position: Vec3::new(-hw, 0.0, hd), normal, tangent, uv: Vec2::new(0.0, 1.0), color: Color::WHITE },
        ];

        Self::from_vertices(vertices)
    }

    /// Create a cylinder.
    pub fn cylinder(radius: f32, height: f32, segments: u32) -> Self {
        let mut vertices = Vec::new();
        let half_height = height / 2.0;

        // Side.
        for i in 0..segments {
            let theta0 = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let theta1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 2.0;

            let x0 = theta0.cos() * radius;
            let z0 = theta0.sin() * radius;
            let x1 = theta1.cos() * radius;
            let z1 = theta1.sin() * radius;

            let n0 = Vec3::new(theta0.cos(), 0.0, theta0.sin());
            let n1 = Vec3::new(theta1.cos(), 0.0, theta1.sin());

            let u0 = i as f32 / segments as f32;
            let u1 = (i + 1) as f32 / segments as f32;

            let tangent0 = Vec3::new(-theta0.sin(), 0.0, theta0.cos());
            let tangent1 = Vec3::new(-theta1.sin(), 0.0, theta1.cos());
            // Bottom triangle.
            vertices.push(Vertex { position: Vec3::new(x0, -half_height, z0), normal: n0, tangent: tangent0, uv: Vec2::new(u0, 0.0), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(x1, -half_height, z1), normal: n1, tangent: tangent1, uv: Vec2::new(u1, 0.0), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(x1, half_height, z1), normal: n1, tangent: tangent1, uv: Vec2::new(u1, 1.0), color: Color::WHITE });

            // Top triangle.
            vertices.push(Vertex { position: Vec3::new(x0, -half_height, z0), normal: n0, tangent: tangent0, uv: Vec2::new(u0, 0.0), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(x1, half_height, z1), normal: n1, tangent: tangent1, uv: Vec2::new(u1, 1.0), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(x0, half_height, z0), normal: n0, tangent: tangent0, uv: Vec2::new(u0, 1.0), color: Color::WHITE });
        }

        // Top cap.
        let top_normal = Vec3::UP;
        for i in 0..segments {
            let theta0 = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let theta1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 2.0;

            let tangent = Vec3::new(-theta0.sin(), 0.0, theta0.cos());
            vertices.push(Vertex { position: Vec3::new(0.0, half_height, 0.0), normal: top_normal, tangent, uv: Vec2::new(0.5, 0.5), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(theta0.cos() * radius, half_height, theta0.sin() * radius), normal: top_normal, tangent, uv: Vec2::new(0.5 + theta0.cos() * 0.5, 0.5 + theta0.sin() * 0.5), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(theta1.cos() * radius, half_height, theta1.sin() * radius), normal: top_normal, tangent, uv: Vec2::new(0.5 + theta1.cos() * 0.5, 0.5 + theta1.sin() * 0.5), color: Color::WHITE });
        }

        // Bottom cap.
        let bottom_normal = Vec3::DOWN;
        for i in 0..segments {
            let theta0 = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let theta1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 2.0;

            let tangent = Vec3::new(-theta0.sin(), 0.0, theta0.cos());
            vertices.push(Vertex { position: Vec3::new(0.0, -half_height, 0.0), normal: bottom_normal, tangent, uv: Vec2::new(0.5, 0.5), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(theta1.cos() * radius, -half_height, theta1.sin() * radius), normal: bottom_normal, tangent, uv: Vec2::new(0.5 + theta1.cos() * 0.5, 0.5 + theta1.sin() * 0.5), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(theta0.cos() * radius, -half_height, theta0.sin() * radius), normal: bottom_normal, tangent, uv: Vec2::new(0.5 + theta0.cos() * 0.5, 0.5 + theta0.sin() * 0.5), color: Color::WHITE });
        }

        Self::from_vertices(vertices)
    }

    /// Create a cone.
    pub fn cone(radius: f32, height: f32, segments: u32) -> Self {
        let mut vertices = Vec::new();
        let half_height = height / 2.0;

        // Side.
        for i in 0..segments {
            let theta0 = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let theta1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 2.0;

            let x0 = theta0.cos() * radius;
            let z0 = theta0.sin() * radius;
            let x1 = theta1.cos() * radius;
            let z1 = theta1.sin() * radius;

            // Normal pointing outward and up.
            let n0 = Vec3::new(theta0.cos(), radius / height, theta0.sin()).normalized();
            let n1 = Vec3::new(theta1.cos(), radius / height, theta1.sin()).normalized();
            let n_tip = (n0 + n1).normalized();

            let tangent0 = Vec3::new(-theta0.sin(), 0.0, theta0.cos());
            let tangent1 = Vec3::new(-theta1.sin(), 0.0, theta1.cos());
            vertices.push(Vertex { position: Vec3::new(x0, -half_height, z0), normal: n0, tangent: tangent0, uv: Vec2::new(0.0, 0.0), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(x1, -half_height, z1), normal: n1, tangent: tangent1, uv: Vec2::new(1.0, 0.0), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(0.0, half_height, 0.0), normal: n_tip, tangent: tangent0, uv: Vec2::new(0.5, 1.0), color: Color::WHITE });
        }

        // Bottom cap.
        let bottom_normal = Vec3::DOWN;
        for i in 0..segments {
            let theta0 = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let theta1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 2.0;

            let tangent = Vec3::new(-theta0.sin(), 0.0, theta0.cos());
            vertices.push(Vertex { position: Vec3::new(0.0, -half_height, 0.0), normal: bottom_normal, tangent, uv: Vec2::new(0.5, 0.5), color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(theta1.cos() * radius, -half_height, theta1.sin() * radius), normal: bottom_normal, tangent, uv: Vec2::ZERO, color: Color::WHITE });
            vertices.push(Vertex { position: Vec3::new(theta0.cos() * radius, -half_height, theta0.sin() * radius), normal: bottom_normal, tangent, uv: Vec2::ZERO, color: Color::WHITE });
        }

        Self::from_vertices(vertices)
    }

    /// Create a torus.
    pub fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> Self {
        let mut vertices = Vec::new();

        for i in 0..major_segments {
            let theta0 = (i as f32 / major_segments as f32) * std::f32::consts::PI * 2.0;
            let theta1 = ((i + 1) as f32 / major_segments as f32) * std::f32::consts::PI * 2.0;

            for j in 0..minor_segments {
                let phi0 = (j as f32 / minor_segments as f32) * std::f32::consts::PI * 2.0;
                let phi1 = ((j + 1) as f32 / minor_segments as f32) * std::f32::consts::PI * 2.0;

                let point = |theta: f32, phi: f32| -> (Vec3, Vec3) {
                    let cx = major_radius * theta.cos();
                    let cz = major_radius * theta.sin();
                    let x = (major_radius + minor_radius * phi.cos()) * theta.cos();
                    let y = minor_radius * phi.sin();
                    let z = (major_radius + minor_radius * phi.cos()) * theta.sin();
                    let normal = Vec3::new(x - cx, y, z - cz).normalized();
                    (Vec3::new(x, y, z), normal)
                };

                let (p00, n00) = point(theta0, phi0);
                let (p10, n10) = point(theta1, phi0);
                let (p01, n01) = point(theta0, phi1);
                let (p11, n11) = point(theta1, phi1);

                let t00 = Vec3::new(-theta0.sin(), 0.0, theta0.cos());
                let t10 = Vec3::new(-theta1.sin(), 0.0, theta1.cos());
                vertices.push(Vertex { position: p00, normal: n00, tangent: t00, uv: Vec2::ZERO, color: Color::WHITE });
                vertices.push(Vertex { position: p10, normal: n10, tangent: t10, uv: Vec2::ZERO, color: Color::WHITE });
                vertices.push(Vertex { position: p11, normal: n11, tangent: t10, uv: Vec2::ZERO, color: Color::WHITE });

                vertices.push(Vertex { position: p00, normal: n00, tangent: t00, uv: Vec2::ZERO, color: Color::WHITE });
                vertices.push(Vertex { position: p11, normal: n11, tangent: t10, uv: Vec2::ZERO, color: Color::WHITE });
                vertices.push(Vertex { position: p01, normal: n01, tangent: t00, uv: Vec2::ZERO, color: Color::WHITE });
            }
        }

        Self::from_vertices(vertices)
    }

    /// Create ground plane with checkerboard pattern.
    pub fn ground(size: f32, tiles: u32) -> Self {
        let mut vertices = Vec::new();
        let tile_size = size / tiles as f32;
        let half_size = size / 2.0;
        let normal = Vec3::UP;

        for i in 0..tiles {
            for j in 0..tiles {
                let x = -half_size + i as f32 * tile_size;
                let z = -half_size + j as f32 * tile_size;

                let color = if (i + j) % 2 == 0 {
                    Color::new(0.3, 0.5, 0.3, 1.0)
                } else {
                    Color::new(0.35, 0.55, 0.35, 1.0)
                };

                let tangent = Vec3::RIGHT;
                vertices.push(Vertex { position: Vec3::new(x, 0.0, z), normal, tangent, uv: Vec2::new(0.0, 0.0), color });
                vertices.push(Vertex { position: Vec3::new(x + tile_size, 0.0, z), normal, tangent, uv: Vec2::new(1.0, 0.0), color });
                vertices.push(Vertex { position: Vec3::new(x + tile_size, 0.0, z + tile_size), normal, tangent, uv: Vec2::new(1.0, 1.0), color });

                vertices.push(Vertex { position: Vec3::new(x, 0.0, z), normal, tangent, uv: Vec2::new(0.0, 0.0), color });
                vertices.push(Vertex { position: Vec3::new(x + tile_size, 0.0, z + tile_size), normal, tangent, uv: Vec2::new(1.0, 1.0), color });
                vertices.push(Vertex { position: Vec3::new(x, 0.0, z + tile_size), normal, tangent, uv: Vec2::new(0.0, 1.0), color });
            }
        }

        Self::from_vertices(vertices)
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}
