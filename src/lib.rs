mod core;
mod utils;
mod world;

use crate::core::engine::Engine;

pub fn run_game(config_path: &str) {
    // instantiate the utils
    let mut game_engine = Engine::new(config_path);

    // start main game loop
    game_engine.run();
}
