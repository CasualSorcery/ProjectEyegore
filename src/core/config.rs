#![allow(dead_code)]
#![allow(unused_variables)]
use crate::world::entity::Entity;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GameConfig {
    pub name: String,
    pub scr_width: usize,
    pub scr_height: usize,
    pub target_fps: usize, // TODO: Add FPS clamping
    pub fullscreen: bool,
    pub levels: Vec<Level>,
}
#[derive(Deserialize)]
pub struct Level {
    pub name: String,
    pub map_width: usize,
    pub map_height: usize,
    pub wall_map: Vec<u8>,
    pub floor_map: Vec<u8>,
    pub ceil_map: Vec<u8>,
    pub player_start_x: f64,
    pub player_start_y: f64,
    pub max_fog_distance: f64,
    pub textures: Vec<String>,
    pub entities: Vec<Entity>,
}
// TODO: Ponder Tiled Support for map making
// TODO: Add dedicated json parser for Tiled MapMaking support.
pub fn load_config(filepath: &str) -> GameConfig {
    // tries to read the ron file
    let file_content = std::fs::read_to_string(filepath).expect("Failed to read file");
    // tries to serialize the ron file into the GameConfig Struct
    ron::from_str(&file_content).expect("Failed to parse RON") // returning the result as GameConfig
}