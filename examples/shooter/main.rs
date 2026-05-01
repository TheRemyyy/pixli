//! Shooter: first person shooter example.
//!
//! Run with: `cargo run --example shooter`
//!
//! Controls: WASD move, mouse aim, LMB fire, Space jump, ESC release mouse or quit.

mod components;
mod config;
mod map;
mod meshes;
mod run;
mod sound;
mod systems;

fn main() -> pixli::Result<()> {
    run::run()
}
