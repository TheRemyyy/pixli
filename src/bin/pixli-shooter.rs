#[path = "../../examples/shooter/components.rs"]
mod components;
#[path = "../../examples/shooter/config.rs"]
mod config;
#[path = "../../examples/shooter/map.rs"]
mod map;
#[path = "../../examples/shooter/meshes.rs"]
mod meshes;
#[path = "../../examples/shooter/run.rs"]
mod run;
#[path = "../../examples/shooter/sound.rs"]
mod sound;
#[path = "../../examples/shooter/systems.rs"]
mod systems;

fn main() -> pixli::Result<()> {
    run::run()
}
