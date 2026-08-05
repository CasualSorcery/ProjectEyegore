mod config;
mod engine;
mod player;
mod map;
mod entity;

use crate::engine::Engine;

pub fn run_game(config_path: &str) {

    // instantiate the engine
    let mut game_engine = Engine::new(config_path);

    // start main game loop
    game_engine.run();

    // The game_engine and window will drop and close when this returns
    println!("Game exited cleanly.");
}
