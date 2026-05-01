use pixli::prelude::*;

const GRID_WIDTH: i32 = 16;
const GRID_DEPTH: i32 = 16;
const SPACING: f32 = 2.6;

fn main() -> pixli::Result<()> {
    let max_frames = std::env::var("PIXLI_STRESS_FRAMES")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());

    let mut app = App::new()
        .with_title("Pixli Stress")
        .with_app_id("io.github.pixli.stress")
        .with_size(1600, 900)
        .with_vsync(false)
        .add_startup_system(setup)
        .add_system(orbit_camera);

    if let Some(max_frames) = max_frames {
        app = app.with_max_frames(max_frames);
    }

    app.run()
}

fn setup(world: &mut World, renderer: &mut Renderer) {
    renderer.clear_color = Color::new(0.035, 0.045, 0.06, 1.0);
    renderer.fog_color = Color::new(0.05, 0.06, 0.075, 1.0);
    renderer.fog_start = 45.0;
    renderer.fog_end = 190.0;
    renderer.ambient_light = Color::new(0.18, 0.20, 0.24, 1.0);
    renderer.directional_light = Some(Light::directional(
        Vec3::new(-0.35, -0.86, -0.28).normalized(),
        Color::new(1.0, 0.94, 0.82, 1.0),
        2.2,
    ));
    renderer.camera.position = Vec3::new(0.0, 24.0, 48.0);
    renderer.camera.yaw = -90.0_f32.to_radians();
    renderer.camera.pitch = -24.0_f32.to_radians();
    renderer.camera.fov = 55.0_f32.to_radians();
    renderer.camera.far = 500.0;

    renderer.set_msaa(4);
    renderer.graphics.enable_sky = true;
    renderer.graphics.enable_fog = true;
    renderer.graphics.enable_shadows = true;
    renderer.graphics.enable_ssao = true;
    renderer.graphics.enable_bloom = true;
    renderer.graphics.shadow = ShadowSettings::high();
    renderer.graphics.ssao = SsaoSettings::high();
    renderer.graphics.bloom = BloomSettings::high();

    let cube = Mesh::cube(1.0);
    renderer.upload_mesh(&cube);

    for z in 0..GRID_DEPTH {
        for x in 0..GRID_WIDTH {
            let xf = (x - GRID_WIDTH / 2) as f32 * SPACING;
            let zf = (z - GRID_DEPTH / 2) as f32 * SPACING;
            let height = 0.8 + ((x * 13 + z * 7) % 9) as f32 * 0.35;
            let color = Color::new(
                0.25 + x as f32 / GRID_WIDTH as f32 * 0.55,
                0.28 + z as f32 / GRID_DEPTH as f32 * 0.42,
                0.55,
                1.0,
            );
            world
                .spawn()
                .with(Transform::from_position_scale(
                    Vec3::new(xf, height * 0.5, zf),
                    Vec3::new(1.0, height, 1.0),
                ))
                .with(cube.clone())
                .with(Material::color(color).with_roughness(0.38))
                .build();
        }
    }

    let emissive = Material::emissive(Color::new(0.25, 0.8, 1.0, 1.0), 4.0);
    for i in 0..16 {
        let angle = i as f32 / 16.0 * std::f32::consts::TAU;
        world
            .spawn()
            .with(Transform::from_position_scale(
                Vec3::new(angle.cos() * 28.0, 5.0, angle.sin() * 28.0),
                Vec3::splat(0.7),
            ))
            .with(cube.clone())
            .with(emissive)
            .build();
    }
}

fn orbit_camera(state: &mut GameState) {
    let t = state.time.elapsed();
    let radius = 54.0;
    state.renderer.camera.position = Vec3::new(t.cos() * radius, 26.0, t.sin() * radius);
    state.renderer.camera.yaw = t + std::f32::consts::PI;
    state.renderer.camera.pitch = -25.0_f32.to_radians();
}
