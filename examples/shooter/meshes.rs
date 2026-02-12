//! Unlit mesh creation for shooter: pistol, crosshair, tracer, enemies.

use pixli::prelude::*;
use pixli::renderer::{UnlitMesh, UnlitVertex};

/// Unlit cube with uniform color.
pub fn unlit_cube(size: f32, color: [f32; 3]) -> UnlitMesh {
    let mesh = Mesh::cube(size);
    let vertices: Vec<UnlitVertex> = mesh
        .vertices
        .iter()
        .map(|v| UnlitVertex {
            position: v.position.to_array(),
            color,
        })
        .collect();
    UnlitMesh::from_vertices(vertices)
}

/// Pistol mesh: readable shapes, two colors, barrel along negative Z.
pub fn create_pistol_mesh() -> UnlitMesh {
    let body: [f32; 3] = [0.25, 0.25, 0.28];
    let accent: [f32; 3] = [0.38, 0.38, 0.42];
    let mut vertices = Vec::new();
    vertices.extend(box_verts((0.0, -0.08, 0.12), (0.055, 0.08, 0.055), body));
    vertices.extend(box_verts((0.0, 0.0, 0.02), (0.07, 0.045, 0.08), body));
    vertices.extend(box_verts((0.0, 0.0, -0.2), (0.03, 0.03, 0.12), accent));
    vertices.extend(box_verts((0.0, 0.0, -0.34), (0.025, 0.025, 0.02), accent));
    UnlitMesh::from_vertices(vertices)
}

/// Small cube for muzzle flash.
pub fn create_muzzle_flash_mesh() -> UnlitMesh {
    let bright: [f32; 3] = [1.0, 0.9, 0.4];
    unlit_cube(0.08, bright)
}

/// Box vertices at center and half extents with color.
pub fn box_verts(
    center: (f32, f32, f32),
    half: (f32, f32, f32),
    color: [f32; 3],
) -> Vec<UnlitVertex> {
    let (cx, cy, cz) = center;
    let (hx, hy, hz) = half;
    let mesh = Mesh::cube(1.0);
    mesh.vertices
        .iter()
        .map(|v| UnlitVertex {
            position: [
                cx + v.position.x * hx * 2.0,
                cy + v.position.y * hy * 2.0,
                cz + v.position.z * hz * 2.0,
            ],
            color,
        })
        .collect()
}

/// Crosshair: small plus shape.
pub fn create_crosshair_mesh() -> UnlitMesh {
    let white: [f32; 3] = [1.0, 1.0, 1.0];
    let mut vertices = Vec::new();
    vertices.extend(box_verts((0.0, 0.0, 0.0), (0.028, 0.003, 0.003), white));
    vertices.extend(box_verts((0.0, 0.0, 0.0), (0.003, 0.028, 0.003), white));
    UnlitMesh::from_vertices(vertices)
}

/// Tracer beam: unit cube scaled to ray length, bright red.
pub fn create_tracer_mesh() -> UnlitMesh {
    let color: [f32; 3] = [1.0, 0.25, 0.2];
    unlit_cube(1.0, color)
}
