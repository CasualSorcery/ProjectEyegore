mod core;
mod utils;
mod world;

use crate::core::engine::Engine;
use winit::event_loop::{ControlFlow, EventLoop};

/// Wrapper that sets up the window event loop and game ray-casting logic
/// as per .ron in the file path
///
/// # Arguments
///
/// * `config_path` - A string slice pointing to the `.ron` config file desired
pub fn run_game(config_path: &str) {
    // create the event loop
    let event_loop = EventLoop::new()
        .expect("Failed to create event loop");

    // set to Poll so the game loop runs constantly without waiting for key presses
    event_loop.set_control_flow(ControlFlow::Poll);

    // initialize the engine
    let mut game_engine = Engine::new(config_path);

    // hand over the control to run_app
    let _ = event_loop.run_app(&mut game_engine);
}