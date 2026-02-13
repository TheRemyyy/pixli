//! Map: floor, cover obstacles, lighting and fog.

use pixli::prelude::*;
use pixli::renderer::{Material, Mesh, UnlitMesh, UnlitMeshRef, UnlitVertex};

use crate::config::ARENA_HALF;

/// Floor: checkerboard grid.
fn create_ground_vertices() -> Vec<UnlitVertex> {
    let grid_size: i32 = 45;
    let tile_size: f32 = 4.0;
    let mut vertices = Vec::new();

    for i in -grid_size..grid_size {
        for j in -grid_size..grid_size {
            let x0 = (i as f32) * tile_size;
            let z0 = (j as f32) * tile_size;
            let x1 = ((i + 1) as f32) * tile_size;
            let z1 = ((j + 1) as f32) * tile_size;

            let color = if (i + j).abs() % 2 == 0 {
                [0.22, 0.28, 0.32]
            } else {
                [0.26, 0.32, 0.36]
            };

            vertices.push(UnlitVertex {
                position: [x0, 0.0, z0],
                color,
            });
            vertices.push(UnlitVertex {
                position: [x1, 0.0, z0],
                color,
            });
            vertices.push(UnlitVertex {
                position: [x0, 0.0, z1],
                color,
            });
            vertices.push(UnlitVertex {
                position: [x1, 0.0, z0],
                color,
            });
            vertices.push(UnlitVertex {
                position: [x1, 0.0, z1],
                color,
            });
            vertices.push(UnlitVertex {
                position: [x0, 0.0, z1],
                color,
            });
        }
    }

    vertices
}

/// Configure renderer: light sky, soft fog, sun, MSAA.
pub fn setup_renderer(renderer: &mut Renderer) {
    renderer.clear_color = Color::new(0.52, 0.68, 0.95, 1.0);
    renderer.fog_color = Color::new(0.72, 0.80, 0.92, 1.0);
    renderer.fog_start = 40.0;
    renderer.fog_end = 150.0;
    renderer.ambient_light = Color::new(0.38, 0.44, 0.54, 1.0);
    renderer.directional_light = Some(Light::directional(
        Vec3::new(0.35, -0.92, 0.15).normalized(),
        Color::new(1.0, 0.97, 0.9, 1.35),
        1.35,
    ));
    renderer.camera.fov = std::f32::consts::FRAC_PI_4;
    renderer.camera.near = 0.1;
    renderer.camera.far = 1000.0;

    renderer.set_msaa(4);
    renderer.graphics.enable_shadows = true;
    renderer.graphics.enable_sky = true;
    renderer.graphics.enable_ssao = false;
    renderer.graphics.enable_bloom = false;
}

/// Spawn map: ground (unlit), walls, crates (lit).
pub fn spawn_map(world: &mut World, renderer: &mut Renderer) {
    let ground_verts = create_ground_vertices();
    let ground_mesh_id = renderer.upload_unlit_mesh(&UnlitMesh::from_vertices(ground_verts));

    let cube_mesh = Mesh::cube(1.0);
    renderer.upload_mesh(&cube_mesh);

    let wall_color = Color::new(0.38, 0.42, 0.48, 1.0);
    let crate_color = Color::new(0.58, 0.38, 0.22, 1.0);

    // Ground: unlit grid and collider
    let ground_size = 45 * 2 * 4;
    world
        .spawn()
        .with(Transform::default())
        .with(UnlitMeshRef(ground_mesh_id))
        .with(
            Collider::box_collider(Vec3::new(ground_size as f32, 0.02, ground_size as f32))
                .with_offset(Vec3::new(0.0, -0.01, 0.0)),
        )
        .with(RigidBody::statik())
        .build();

    // Arena walls: four sides around the map (lit, cast shadows).
    let wall_h = 4.0;
    let wall_thick = 2.0;
    let s = ARENA_HALF + wall_thick;
    let wall_size_z = Vec3::new(s * 2.0 + wall_thick * 2.0, wall_h, wall_thick);
    let wall_size_x = Vec3::new(wall_thick, wall_h, s * 2.0);

    for (pos, scale) in [
        (Vec3::new(0.0, wall_h / 2.0, -s), wall_size_z),
        (Vec3::new(0.0, wall_h / 2.0, s), wall_size_z),
        (Vec3::new(-s, wall_h / 2.0, 0.0), wall_size_x),
        (Vec3::new(s, wall_h / 2.0, 0.0), wall_size_x),
    ] {
        world
            .spawn()
            .with(Transform::from_position_scale(pos, scale))
            .with(cube_mesh.clone())
            .with(Material::color(wall_color))
            .with(Collider::box_collider(scale))
            .with(RigidBody::statik())
            .build();
    }

    // Crates (cover): lit, various sizes.
    let crates: [(f32, f32, f32, f32, f32, f32); 14] = [
        (-25.0, 0.6, 20.0, 1.2, 1.2, 1.2),
        (20.0, 0.5, -18.0, 1.0, 1.0, 1.0),
        (-15.0, 0.8, -10.0, 1.6, 1.6, 1.6),
        (18.0, 0.4, 15.0, 0.8, 0.8, 0.8),
        (-30.0, 0.5, -15.0, 1.0, 1.0, 1.0),
        (10.0, 0.7, 25.0, 1.4, 1.4, 1.4),
        (-20.0, 0.5, 28.0, 1.0, 1.0, 1.0),
        (28.0, 0.6, -12.0, 1.2, 1.2, 1.2),
        (0.0, 0.5, -28.0, 1.0, 1.0, 1.0),
        (-22.0, 0.4, 0.0, 0.8, 0.8, 0.8),
        (22.0, 0.6, 22.0, 1.2, 1.2, 1.2),
        (-10.0, 0.5, -22.0, 1.0, 1.0, 1.0),
        (30.0, 0.5, 5.0, 1.0, 1.0, 1.0),
        (-5.0, 0.6, 30.0, 1.2, 1.2, 1.2),
    ];

    for (x, y, z, sx, sy, sz) in crates {
        let size = Vec3::new(sx, sy, sz);
        world
            .spawn()
            .with(Transform::from_position_scale(Vec3::new(x, y, z), size))
            .with(cube_mesh.clone())
            .with(Material::color(crate_color))
            .with(Collider::box_collider(size))
            .with(RigidBody::statik())
            .build();
    }
}
