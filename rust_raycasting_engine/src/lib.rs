mod config;
mod engine;
mod player;
mod map;
mod entity;

use crate::engine::Engine;

pub fn run_game(config_path: &str) {
    // Note: Make sure this path matches where your RON file actually lives
    // relative to the terminal running the command (e.g., "assets/config.ron")

    // Instantiate the engine (which loads the config, textures, and window)
    let mut game_engine = Engine::new(config_path);

    // Kick off the main game loop
    game_engine.run();

    // The game_engine and window will safely drop and close when this returns
    println!("Game exited cleanly.");
}
